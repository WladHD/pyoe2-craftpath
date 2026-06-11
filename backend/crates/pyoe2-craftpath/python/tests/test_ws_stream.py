"""WebSocket live mode against an in-process websockets server."""

import asyncio
import json

import pytest
import websockets

from pyoe2_craftpath.client._ws import stream_events, ws_url


def test_ws_url_scheme_mapping():
    assert (
        ws_url("http://backend:8080", "j1") == "ws://backend:8080/api/v1/jobs/j1/ws"
    )
    assert (
        ws_url("https://backend/", "j1") == "wss://backend/api/v1/jobs/j1/ws"
    )


SCRIPT = [
    {"jobId": "j1", "status": {"jobId": "j1", "state": "JOB_STATE_QUEUED", "queuePosition": 2}},
    {"jobId": "j1", "status": {"jobId": "j1", "state": "JOB_STATE_RUNNING",
                               "progress": {"phase": "building_matrix", "percent": 12}}},
    {"jobId": "j1", "status": {"jobId": "j1", "state": "JOB_STATE_SUCCEEDED"}},
    # must never be delivered: the client stops at the terminal event
    {"jobId": "j1", "status": {"jobId": "j1", "state": "JOB_STATE_QUEUED"}},
]


@pytest.mark.asyncio
async def test_stream_events_until_terminal():
    async def server_handler(socket):
        for event in SCRIPT:
            await socket.send(json.dumps(event))
        await asyncio.sleep(0.2)

    async with websockets.serve(server_handler, "127.0.0.1", 0) as server:
        port = server.sockets[0].getsockname()[1]

        received = []
        async for event in stream_events(f"http://127.0.0.1:{port}", "j1"):
            received.append(event["status"]["state"])

    assert received == ["JOB_STATE_QUEUED", "JOB_STATE_RUNNING", "JOB_STATE_SUCCEEDED"]
