"""WebSocket live mode: stream JobEvent frames for a job."""

from __future__ import annotations

import json
from typing import AsyncIterator


def ws_url(base_url: str, job_id: str) -> str:
    url = base_url.rstrip("/")
    if url.startswith("https://"):
        url = "wss://" + url[len("https://") :]
    elif url.startswith("http://"):
        url = "ws://" + url[len("http://") :]
    return f"{url}/api/v1/jobs/{job_id}/ws"


async def stream_events(base_url: str, job_id: str) -> AsyncIterator[dict]:
    """Yield JobEvent dicts (canonical JSON) until a terminal status arrives.

    The server pushes the current status immediately on connect, then every
    change (queue position, progress, terminal state).
    """
    import websockets

    async with websockets.connect(ws_url(base_url, job_id)) as socket:
        async for frame in socket:
            if isinstance(frame, bytes):
                continue  # binary frames only appear with ?encoding=proto
            event = json.loads(frame)
            yield event

            status = event.get("status")
            if status is not None:
                from ..engine._spec import JobState

                if JobState.parse(status.get("state", "")).is_terminal:
                    return
