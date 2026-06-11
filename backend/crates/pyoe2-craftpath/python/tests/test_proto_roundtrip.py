"""Cross-language parity: the canonical JSON produced/consumed here must match
what the Rust pbjson side guards in
backend/crates/craftpath-proto/tests/test_convert_roundtrip.rs."""

import json

from google.protobuf import json_format

from pyoe2_craftpath._proto import currency_pb2, item_pb2, job_pb2, presets_pb2


def sample_request_dict() -> dict:
    return {
        "league": "Standard",
        "start": {
            "itemLevel": 81,
            "rarity": "ITEM_RARITY_NORMAL",
            "baseId": 20,
            "affixes": [],
        },
        "target": {
            "itemLevel": 81,
            "rarity": "ITEM_RARITY_RARE",
            "baseId": 20,
            "affixes": [
                {
                    "affixId": 5119,
                    "fractured": False,
                    "tier": {"tier": 3, "bounds": "AFFIX_TIER_LEVEL_BOUNDS_MINIMUM"},
                }
            ],
        },
        "matrixBuilder": "MATRIX_BUILDER_PRESET_HAPPY_PATH",
        "pathAnalyzers": ["STATISTIC_ANALYZER_PATH_PRESET_UNIQUE_PATH_CHANCE"],
        "limits": {"maxRoutes": 3, "maxRamInBytes": "500000000"},
        "resultOptions": {"includePrettyStrings": True},
    }


def test_submit_request_roundtrip_binary_and_json():
    message = json_format.ParseDict(sample_request_dict(), job_pb2.SubmitJobRequest())
    assert message.limits.max_ram_in_bytes == 500_000_000
    assert message.start.item_level == 81

    decoded = job_pb2.SubmitJobRequest()
    decoded.ParseFromString(message.SerializeToString())
    assert decoded == message

    back = json_format.MessageToDict(decoded)
    # 64-bit ints stay strings in canonical JSON
    assert back["limits"]["maxRamInBytes"] == "500000000"
    assert back["matrixBuilder"] == "MATRIX_BUILDER_PRESET_HAPPY_PATH"


def test_all_enum_values_parse():
    for value in presets_pb2.StatisticAnalyzerPathPreset.values():
        assert presets_pb2.StatisticAnalyzerPathPreset.Name(value)
    for value in currency_pb2.CraftCurrencyKind.values():
        assert currency_pb2.CraftCurrencyKind.Name(value)
    assert len(currency_pb2.CraftCurrencyKind.values()) == 40  # 39 + UNSPECIFIED
    assert item_pb2.ItemRarity.Value("ITEM_RARITY_UNIQUE") == 4


def test_job_event_oneof():
    event = job_pb2.JobEvent(
        job_id="x",
        status=job_pb2.JobStatus(job_id="x", state=job_pb2.JOB_STATE_QUEUED),
    )
    as_dict = json_format.MessageToDict(event)
    assert as_dict["status"]["state"] == "JOB_STATE_QUEUED"
    parsed = json_format.ParseDict(json.loads(json.dumps(as_dict)), job_pb2.JobEvent())
    assert parsed.WhichOneof("event") == "status"
