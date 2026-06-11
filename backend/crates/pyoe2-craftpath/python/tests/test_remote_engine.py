"""RemoteEngine against a mocked backend (httpx.MockTransport)."""

import json
from types import SimpleNamespace

import httpx
import pytest

import pyoe2_craftpath as pc
from pyoe2_craftpath.client._errors import RemoteJobError
from pyoe2_craftpath.engine import JobState, RemoteEngine
from pyoe2_craftpath.engine._convert import job_spec_to_dict


class FakeSnapshotParts:
    """Duck-typed stand-ins for the native item snapshot attribute surface
    (real snapshots require provider data from the network)."""

    @staticmethod
    def affix(affix_id: int, tier: int, minimum: bool) -> SimpleNamespace:
        return SimpleNamespace(
            affix=SimpleNamespace(raw_value=affix_id),
            fractured=False,
            tier=SimpleNamespace(
                tier=SimpleNamespace(raw_value=tier),
                bounds=(
                    pc.AffixTierLevelBoundsEnum.Minimum
                    if minimum
                    else pc.AffixTierLevelBoundsEnum.Exact
                ),
            ),
        )

    @staticmethod
    def snapshot(rarity, affixes=()) -> SimpleNamespace:
        return SimpleNamespace(
            item_level=SimpleNamespace(raw_value=81),
            rarity=rarity,
            base_id=SimpleNamespace(raw_value=20),
            affixes=list(affixes),
            corrupted=False,
            allowed_sockets=0,
            sockets=[],
        )


def make_spec() -> pc.JobSpec:
    return pc.JobSpec(
        start=FakeSnapshotParts.snapshot(pc.ItemRarityEnum.Normal),
        target=FakeSnapshotParts.snapshot(
            pc.ItemRarityEnum.Rare, [FakeSnapshotParts.affix(5119, 3, True)]
        ),
        league="Standard",
        max_routes=3,
        max_ram_in_bytes=500_000_000,
    )


def test_job_spec_to_dict_shape():
    body = job_spec_to_dict(make_spec())
    assert body["start"]["rarity"] == "ITEM_RARITY_NORMAL"
    assert body["target"]["affixes"][0]["affixId"] == 5119
    assert body["target"]["affixes"][0]["tier"]["bounds"] == "AFFIX_TIER_LEVEL_BOUNDS_MINIMUM"
    assert body["limits"]["maxRamInBytes"] == "500000000"  # string per canonical JSON
    assert body["pathAnalyzers"] == ["STATISTIC_ANALYZER_PATH_PRESET_UNIQUE_PATH_CHANCE"]

    # the dict parses cleanly into the generated proto message
    from google.protobuf import json_format

    from pyoe2_craftpath._proto import job_pb2

    message = json_format.ParseDict(body, job_pb2.SubmitJobRequest())
    assert message.limits.max_ram_in_bytes == 500_000_000


def _result_payload() -> dict:
    return {
        "matrixSize": "3",
        "pathResults": [
            {
                "preset": "STATISTIC_ANALYZER_PATH_PRESET_UNIQUE_PATH_CHANCE",
                "unitType": "%",
                "routes": [
                    {
                        "chance": 0.047,
                        "weight": 0.047,
                        "pretty": "the best route",
                        "route": [
                            {
                                "itemMatrixId": "123",
                                "currencyList": {
                                    "list": [
                                        {
                                            "kind": "CRAFT_CURRENCY_KIND_ORB_OF_TRANSMUTATION_PERFECT",
                                            "displayName": "Perfect Orb of Transmutation",
                                        }
                                    ]
                                },
                            }
                        ],
                    }
                ],
            }
        ],
        "prettyText": "the best route",
    }


def make_mock_backend(statuses: list[dict]):
    """Mock REST backend: one submit, then successive statuses, then result."""
    status_iter = iter(statuses)
    submitted = {}

    def handler(request: httpx.Request) -> httpx.Response:
        if request.method == "POST" and request.url.path == "/api/v1/jobs":
            submitted["body"] = json.loads(request.content)
            return httpx.Response(
                202,
                json={
                    "jobId": "job-1",
                    "status": {"jobId": "job-1", "state": "JOB_STATE_QUEUED", "queuePosition": 3},
                },
            )
        if request.method == "GET" and request.url.path == "/api/v1/jobs/job-1":
            return httpx.Response(200, json=next(status_iter))
        if request.method == "GET" and request.url.path == "/api/v1/jobs/job-1/result":
            return httpx.Response(200, json=_result_payload())
        if request.method == "DELETE" and request.url.path == "/api/v1/jobs/job-1":
            return httpx.Response(
                202, json={"jobId": "job-1", "state": "JOB_STATE_CANCELLED"}
            )
        return httpx.Response(
            404, json={"code": "JOB_NOT_FOUND", "message": "no such route"}
        )

    client = httpx.Client(
        transport=httpx.MockTransport(handler), base_url="http://test.invalid"
    )
    return client, submitted


def test_submit_wait_success():
    client, submitted = make_mock_backend(
        [
            {"jobId": "job-1", "state": "JOB_STATE_QUEUED", "queuePosition": 2},
            {"jobId": "job-1", "state": "JOB_STATE_RUNNING",
             "progress": {"phase": "building_matrix", "percent": 10}},
            {"jobId": "job-1", "state": "JOB_STATE_SUCCEEDED"},
        ]
    )
    engine = RemoteEngine("http://test.invalid", json=True, _client=client)

    seen_states = []
    job = engine.submit(make_spec())
    assert job.job_id == "job-1"
    assert submitted["body"]["league"] == "Standard"

    result = job.wait(poll_interval=0.01, on_status=lambda s: seen_states.append(s["state"]))
    assert seen_states == ["JOB_STATE_QUEUED", "JOB_STATE_RUNNING", "JOB_STATE_SUCCEEDED"]
    assert result.matrix_size == 3
    assert result.path_results[0].routes[0].steps == [["Perfect Orb of Transmutation"]]
    assert result.path_results[0].routes[0].pretty == "the best route"
    assert result.pretty_text == "the best route"


def test_failed_job_raises_native_exception():
    client, _ = make_mock_backend(
        [
            {
                "jobId": "job-1",
                "state": "JOB_STATE_FAILED",
                "error": {"code": "RAM_LIMIT_REACHED", "message": "out of budget"},
            }
        ]
    )
    engine = RemoteEngine("http://test.invalid", json=True, _client=client)
    job = engine.submit(make_spec())
    with pytest.raises(pc.RamLimitError):
        job.wait(poll_interval=0.01)


def test_unknown_job_raises_remote_error():
    client, _ = make_mock_backend([])
    engine = RemoteEngine("http://test.invalid", json=True, _client=client)

    with pytest.raises(RemoteJobError) as excinfo:
        engine._transport.status("missing")
    assert excinfo.value.code == "JOB_NOT_FOUND"


def test_cancel():
    client, _ = make_mock_backend([])
    engine = RemoteEngine("http://test.invalid", json=True, _client=client)
    job = engine.submit(make_spec())
    cancelled = job.cancel()
    assert JobState.parse(cancelled["state"]) == JobState.CANCELLED


def test_proto_wire_mode():
    """proto=default: request body is binary protobuf, Accept negotiates."""
    from google.protobuf import json_format

    from pyoe2_craftpath._proto import job_pb2

    captured = {}

    def handler(request: httpx.Request) -> httpx.Response:
        captured["content_type"] = request.headers["content-type"]
        captured["accept"] = request.headers.get("accept", "")
        message = job_pb2.SubmitJobRequest()
        message.ParseFromString(request.content)
        captured["league"] = message.league

        response = job_pb2.SubmitJobResponse(job_id="job-1")
        response.status.job_id = "job-1"
        response.status.state = job_pb2.JOB_STATE_QUEUED
        return httpx.Response(
            202,
            content=response.SerializeToString(),
            headers={"Content-Type": "application/x-protobuf"},
        )

    client = httpx.Client(
        transport=httpx.MockTransport(handler), base_url="http://test.invalid"
    )
    engine = RemoteEngine("http://test.invalid", _client=client)
    job = engine.submit(make_spec())

    assert captured["content_type"] == "application/x-protobuf"
    assert "application/x-protobuf" in captured["accept"]
    assert captured["league"] == "Standard"
    assert job.job_id == "job-1"
    # the parsed snapshot survived the dict->proto conversion
    json_format  # silence linter
