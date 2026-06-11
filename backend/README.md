# pyoe2-craftpath backend

Cargo workspace containing the calculation engine, the wire contract, the
backend services and the Python bindings.

## Crates

| Crate | What it is |
|---|---|
| `crates/craftpath-core` | Pure calculation engine (item matrix propagation + route statistics). Feature `python` (non-default) adds the PyO3 class attributes. |
| `crates/craftpath-proto` | `prost`/`pbjson` types generated from [`/proto`](../proto) plus conversions to/from the domain types. One generated type serves both the binary protobuf and the canonical JSON encoding. |
| `crates/craftpath-server` | The `pyoe2-backend` binary: `rest` \| `worker` \| `mcp` \| `cli` modes. |
| `crates/pyoe2-craftpath` | Python package (maturin): native extension `pyoe2_craftpath._native` + pure-Python engine/client layer. |

## Services

```
client ── REST (JSON or protobuf) ──> pyoe2-backend rest ──> Redis queue ──> pyoe2-backend worker
   │                                        │                                   │
   └── WebSocket live mode <── pub/sub ─────┘          league data (CoE + poe.ninja, cached)
                LLMs ──> pyoe2-backend mcp (MCP tools) ──> same Redis queue
```

- `pyoe2-backend rest` — API under `/api/v1`: `POST /jobs`, `GET /jobs/{id}`,
  `GET /jobs/{id}/result`, `DELETE /jobs/{id}`, `GET /jobs/{id}/ws` (live mode),
  `GET /presets`, plus `/healthz` `/readyz`. Bodies are negotiated between
  `application/json` and `application/x-protobuf` (see `/proto/craftpath/v1`).
- `pyoe2-backend worker` — claims jobs from the Redis stream (consumer group
  with crash recovery), one calculation per pod, progress/heartbeat flushed
  every 500ms, cooperative cancellation and wall-clock timeouts.
- `pyoe2-backend mcp` — Model Context Protocol server (streamable HTTP or
  stdio) with tools `submit_calculation`, `get_job_status`, `get_job_result`,
  `cancel_job`, `list_presets`.
- `pyoe2-backend cli` — the classic interactive CLI, unchanged.

### Run locally

```bash
docker run -d --rm -p 6379:6379 redis:7-alpine
cargo run -p craftpath-server -- worker &
cargo run -p craftpath-server -- rest
# submit a job:
curl -s -X POST localhost:8080/api/v1/jobs -H 'Content-Type: application/json' -d @job.json
```

Configuration is environment-driven: `REDIS_URL`, `BIND_ADDR`,
`POE2_LEAGUE_DEFAULT`, `JOB_TTL_SECS`, `JOB_TIMEOUT_SECS`,
`MAX_RAM_LIMIT_BYTES`, `CACHE_DIR`, `WORKER_HEALTH_ADDR`.

### Deploy

`docker build -f backend/Dockerfile .` from the repo root; Helm chart in
[`/deploy/helm/craftpath`](../deploy/helm/craftpath) (REST deployment +
ingress, CPU-autoscaled workers, optional MCP, single-node Redis or BYO).

## Python package

```python
import pyoe2_craftpath as pc

engine = pc.LocalEngine()                      # in-process (classic behavior)
engine = pc.RemoteEngine("http://backend")     # or: submit to the backend

result = engine.run(pc.JobSpec(start=..., target=..., league="Standard"))
print(result.pretty_text)
```

All classic classes (`Calculator`, `ItemSnapshot`, providers, presets, ...)
remain importable from `pyoe2_craftpath` unchanged. The remote client needs
`pip install 'pyoe2-craftpath[client]'`.

Tests: `cargo test --workspace` (Rust; set `REDIS_TEST_URL` for the queue
integration tests) and `python -m pytest python/tests` in
`crates/pyoe2-craftpath` (Python). Regenerate the committed proto code with
`scripts/gen_proto.sh`, the `.pyi` stub with `cargo run -p pyoe2-craftpath
--bin stub_gen`.
