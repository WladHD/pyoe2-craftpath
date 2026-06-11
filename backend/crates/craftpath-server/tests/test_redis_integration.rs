//! Integration tests against a real Redis instance.
//!
//! Skipped unless `REDIS_TEST_URL` is set, e.g.:
//!   docker run -d --rm -p 6399:6379 redis:7-alpine
//!   REDIS_TEST_URL=redis://127.0.0.1:6399 cargo test -p craftpath-server

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use prost::Message;
use tower::util::ServiceExt;

use craftpath_proto::v1;
use craftpath_server::config::Config;
use craftpath_server::jobs::JobsClient;
use craftpath_server::rest::{AppState, router};

fn test_redis_url() -> Option<String> {
    std::env::var("REDIS_TEST_URL").ok()
}

fn test_config(redis_url: &str) -> Config {
    Config {
        redis_url: redis_url.to_string(),
        bind_addr: "127.0.0.1:0".into(),
        worker_health_addr: "127.0.0.1:0".into(),
        job_ttl_secs: 60,
        job_timeout_secs: 60,
        default_league: "Standard".into(),
        cache_dir: "./cache".into(),
        coe_cache_ttl_secs: 86_400,
        economy_cache_ttl_secs: 3_600,
        max_ram_limit_bytes: 2_000_000_000,
    }
}

fn sample_request() -> v1::SubmitJobRequest {
    let start = v1::ItemSnapshot {
        item_level: 81,
        rarity: v1::ItemRarity::Normal as i32,
        base_id: 20,
        affixes: vec![],
        corrupted: false,
        allowed_sockets: 0,
        sockets: vec![],
    };
    let mut target = start.clone();
    target.rarity = v1::ItemRarity::Rare as i32;
    target.affixes = vec![v1::AffixSpecifier {
        affix_id: 5119,
        fractured: false,
        tier: Some(v1::AffixTierConstraints {
            tier: 3,
            bounds: v1::AffixTierLevelBounds::Minimum as i32,
        }),
    }];

    v1::SubmitJobRequest {
        league: "Standard".into(),
        start: Some(start),
        target: Some(target),
        matrix_builder: v1::MatrixBuilderPreset::HappyPath as i32,
        path_analyzers: vec![v1::StatisticAnalyzerPathPreset::UniquePathChance as i32],
        group_analyzers: vec![],
        limits: Some(v1::Limits {
            max_routes: 3,
            max_ram_in_bytes: 500_000_000,
            timeout_seconds: None,
        }),
        result_options: Some(v1::ResultOptions {
            include_pretty_strings: false,
            include_route_snapshots: false,
            top_n_pretty: None,
        }),
    }
}

#[tokio::test]
async fn test_queue_lifecycle() -> anyhow::Result<()> {
    let Some(url) = test_redis_url() else {
        eprintln!("REDIS_TEST_URL not set — skipping");
        return Ok(());
    };

    let jobs = JobsClient::connect(&url, 60).await?;
    jobs.ensure_group().await?;

    // submit two jobs -> positions 1 and 2
    let status_a = jobs.submit(&sample_request()).await?;
    let status_b = jobs.submit(&sample_request()).await?;
    assert_eq!(status_a.state, v1::JobState::Queued as i32);
    assert_eq!(status_a.queue_position, Some(1));
    assert_eq!(status_b.queue_position, Some(2));

    // worker claims job A
    let (entry_id, job_id, delivery) = jobs
        .claim_next("test-worker", 1_000)
        .await?
        .expect("expected a queued job");
    assert_eq!(job_id, status_a.job_id);
    assert_eq!(delivery, 1);

    let request = jobs.load_request(&job_id).await?.expect("request stored");
    assert_eq!(request.limits.as_ref().unwrap().max_routes, 3);

    jobs.mark_running(&job_id).await?;
    let running = jobs.status(&job_id).await?.unwrap();
    assert_eq!(running.state, v1::JobState::Running as i32);
    assert_eq!(running.queue_position, None);

    // job B moves up to position 1
    let b = jobs.status(&status_b.job_id).await?.unwrap();
    assert_eq!(b.queue_position, Some(1));

    jobs.update_progress(&job_id, "analyzing", 42, 1337).await?;
    let progressed = jobs.status(&job_id).await?.unwrap();
    let progress = progressed.progress.unwrap();
    assert_eq!(progress.percent, 42);
    assert_eq!(progress.routes_found, Some(1337));

    // finish with a result
    let result = v1::JobResult {
        matrix_size: 99,
        path_results: vec![],
        group_results: vec![],
        pretty_text: String::new(),
    };
    jobs.mark_succeeded(&job_id, &result).await?;
    jobs.ack(&entry_id).await?;

    let done = jobs.status(&job_id).await?.unwrap();
    assert_eq!(done.state, v1::JobState::Succeeded as i32);
    let bytes = jobs.result_bytes(&job_id).await?.expect("result stored");
    assert_eq!(v1::JobResult::decode(bytes.as_slice())?.matrix_size, 99);

    // cancel queued job B
    let cancelled = jobs.cancel(&status_b.job_id).await?.unwrap();
    assert_eq!(cancelled.state, v1::JobState::Cancelled as i32);

    // the worker still sees B in the stream but must skip it (cancel flag)
    if let Some((entry_b, job_b, _)) = jobs.claim_next("test-worker", 200).await? {
        assert_eq!(job_b, status_b.job_id);
        assert!(jobs.is_cancel_requested(&job_b).await?);
        jobs.ack(&entry_b).await?;
    }

    Ok(())
}

#[tokio::test]
async fn test_rest_api_json_and_proto() -> anyhow::Result<()> {
    let Some(url) = test_redis_url() else {
        eprintln!("REDIS_TEST_URL not set — skipping");
        return Ok(());
    };

    let config = test_config(&url);
    let jobs = JobsClient::connect(&url, 60).await?;
    jobs.ensure_group().await?;
    let state = AppState {
        jobs,
        redis: redis::Client::open(url.as_str())?,
        config,
    };
    let app = router(state);

    // ---- submit via JSON
    let json_body = serde_json::to_vec(&sample_request())?;
    let response = app
        .clone()
        .oneshot(
            Request::post("/api/v1/jobs")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json_body))?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::ACCEPTED);
    let body = response.into_body().collect().await?.to_bytes();
    let submit: v1::SubmitJobResponse = serde_json::from_slice(&body)?;
    assert!(!submit.job_id.is_empty());

    // ---- submit via protobuf, response negotiated to protobuf
    let proto_body = sample_request().encode_to_vec();
    let response = app
        .clone()
        .oneshot(
            Request::post("/api/v1/jobs")
                .header(header::CONTENT_TYPE, "application/x-protobuf")
                .header(header::ACCEPT, "application/x-protobuf")
                .body(Body::from(proto_body))?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::ACCEPTED);
    assert_eq!(
        response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok()),
        Some("application/x-protobuf")
    );
    let body = response.into_body().collect().await?.to_bytes();
    let submit_proto = v1::SubmitJobResponse::decode(body.as_ref())?;
    assert!(!submit_proto.job_id.is_empty());

    // ---- status of the JSON-submitted job
    let response = app
        .clone()
        .oneshot(Request::get(format!("/api/v1/jobs/{}", submit.job_id)).body(Body::empty())?)
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await?.to_bytes();
    let status: v1::JobStatus = serde_json::from_slice(&body)?;
    assert_eq!(status.state, v1::JobState::Queued as i32);
    assert!(status.queue_position.is_some());

    // ---- result before finishing -> 409
    let response = app
        .clone()
        .oneshot(
            Request::get(format!("/api/v1/jobs/{}/result", submit.job_id)).body(Body::empty())?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::CONFLICT);

    // ---- unknown job -> 404
    let response = app
        .clone()
        .oneshot(Request::get("/api/v1/jobs/does-not-exist").body(Body::empty())?)
        .await?;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // ---- invalid body -> 400
    let response = app
        .clone()
        .oneshot(
            Request::post("/api/v1/jobs")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from("{\"start\": 42}"))?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // ---- unsupported content type -> 415
    let response = app
        .clone()
        .oneshot(
            Request::post("/api/v1/jobs")
                .header(header::CONTENT_TYPE, "text/plain")
                .body(Body::from("hi"))?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);

    // ---- cancel both queued jobs
    for id in [&submit.job_id, &submit_proto.job_id] {
        let response = app
            .clone()
            .oneshot(
                Request::delete(format!("/api/v1/jobs/{id}"))
                    .body(Body::empty())?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    // ---- presets endpoint
    let response = app
        .clone()
        .oneshot(Request::get("/api/v1/presets").body(Body::empty())?)
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await?.to_bytes();
    let presets: v1::PresetList = serde_json::from_slice(&body)?;
    assert_eq!(presets.path_analyzers.len(), 4);
    assert_eq!(presets.group_analyzers.len(), 2);

    // ---- health endpoints
    let response = app
        .clone()
        .oneshot(Request::get("/readyz").body(Body::empty())?)
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}
