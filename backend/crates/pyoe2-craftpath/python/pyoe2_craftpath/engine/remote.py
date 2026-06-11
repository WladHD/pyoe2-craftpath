"""RemoteEngine: submit calculations to a pyoe2-craftpath backend.

Requires the ``client`` extra (httpx + websockets + protobuf):
``pip install pyoe2-craftpath[client]``.
"""

from __future__ import annotations

import asyncio
import time
from typing import AsyncIterator, Callable

from ..client._errors import RemoteJobError
from ._convert import job_spec_to_dict, result_dict_to_views
from ._result import CraftResult
from ._spec import JobSpec, JobState


def _require_client_deps() -> None:
    try:
        import httpx  # noqa: F401
    except ImportError as exc:  # pragma: no cover
        raise ImportError(
            "RemoteEngine requires the client extra: pip install 'pyoe2-craftpath[client]'"
        ) from exc


class RemoteEngine:
    """Synchronous client for the backend REST API.

    By default requests/responses travel as binary protobuf
    (``application/x-protobuf``); pass ``json=True`` to use JSON instead
    (handy for debugging with a proxy).
    """

    def __init__(
        self,
        base_url: str,
        *,
        api_key: str | None = None,
        timeout: float = 30.0,
        json: bool = False,
        _client=None,
    ) -> None:
        _require_client_deps()
        from ..client._http import Transport

        self.base_url = base_url.rstrip("/")
        self._transport = Transport(
            self.base_url,
            proto=not json,
            timeout=timeout,
            api_key=api_key,
            client=_client,
        )

    def close(self) -> None:
        self._transport.close()

    def __enter__(self) -> "RemoteEngine":
        return self

    def __exit__(self, *exc) -> None:
        self.close()

    def submit(self, spec: JobSpec) -> "RemoteJob":
        response = self._transport.submit(job_spec_to_dict(spec))
        return RemoteJob(self, response["jobId"], response.get("status"))

    def run(
        self,
        spec: JobSpec,
        *,
        poll_interval: float = 2.0,
        timeout: float | None = None,
    ) -> CraftResult:
        return self.submit(spec).wait(poll_interval=poll_interval, timeout=timeout)


class RemoteJob:
    """Handle for a submitted job: status, queue position, result, live mode."""

    def __init__(self, engine: RemoteEngine, job_id: str, status: dict | None = None) -> None:
        self._engine = engine
        self.job_id = job_id
        self._last_status: dict = status or {}

    # -- inspection ---------------------------------------------------------

    def status(self) -> dict:
        self._last_status = self._engine._transport.status(self.job_id)
        return self._last_status

    def state(self) -> JobState:
        return JobState.parse(self.status().get("state", ""))

    def queue_position(self) -> int | None:
        return self.status().get("queuePosition")

    # -- control ------------------------------------------------------------

    def cancel(self) -> dict:
        return self._engine._transport.cancel(self.job_id)

    # -- completion ---------------------------------------------------------

    def result(self) -> CraftResult:
        return result_dict_to_views(self._engine._transport.result(self.job_id))

    def wait(
        self,
        *,
        poll_interval: float = 2.0,
        timeout: float | None = None,
        on_status: Callable[[dict], None] | None = None,
    ) -> CraftResult:
        deadline = None if timeout is None else time.monotonic() + timeout
        while True:
            status = self.status()
            if on_status is not None:
                on_status(status)
            state = JobState.parse(status.get("state", ""))
            if state == JobState.SUCCEEDED:
                return self.result()
            if state.is_terminal:
                RemoteJobError.from_dict(
                    status.get("error")
                    or {"code": state.name, "message": f"job ended as {state.value}"}
                ).raise_as_native()
            if deadline is not None and time.monotonic() > deadline:
                raise TimeoutError(f"job {self.job_id} did not finish within {timeout}s")
            time.sleep(poll_interval)

    def stream(self, on_event: Callable[[dict], None]) -> CraftResult:
        """Live mode: follow WebSocket events until terminal, then return the
        result. Runs the asyncio machinery internally for sync callers."""

        async def _run() -> CraftResult:
            from ..client._ws import stream_events

            last_state = JobState.UNSPECIFIED
            async for event in stream_events(self._engine.base_url, self.job_id):
                on_event(event)
                status = event.get("status")
                if status is not None:
                    last_state = JobState.parse(status.get("state", ""))
            if last_state == JobState.SUCCEEDED:
                return self.result()
            status = self.status()
            RemoteJobError.from_dict(
                status.get("error")
                or {"code": last_state.name, "message": f"job ended as {last_state.value}"}
            ).raise_as_native()
            raise AssertionError("unreachable")

        return asyncio.run(_run())


class AsyncRemoteEngine:
    """Asynchronous mirror of :class:`RemoteEngine`."""

    def __init__(
        self,
        base_url: str,
        *,
        api_key: str | None = None,
        timeout: float = 30.0,
        json: bool = False,
        _client=None,
    ) -> None:
        _require_client_deps()
        from ..client._http import AsyncTransport

        self.base_url = base_url.rstrip("/")
        self._transport = AsyncTransport(
            self.base_url,
            proto=not json,
            timeout=timeout,
            api_key=api_key,
            client=_client,
        )

    async def aclose(self) -> None:
        await self._transport.aclose()

    async def __aenter__(self) -> "AsyncRemoteEngine":
        return self

    async def __aexit__(self, *exc) -> None:
        await self.aclose()

    async def submit(self, spec: JobSpec) -> "AsyncRemoteJob":
        response = await self._transport.submit(job_spec_to_dict(spec))
        return AsyncRemoteJob(self, response["jobId"], response.get("status"))

    async def run(
        self,
        spec: JobSpec,
        *,
        poll_interval: float = 2.0,
        timeout: float | None = None,
    ) -> CraftResult:
        job = await self.submit(spec)
        return await job.wait(poll_interval=poll_interval, timeout=timeout)


class AsyncRemoteJob:
    def __init__(
        self, engine: AsyncRemoteEngine, job_id: str, status: dict | None = None
    ) -> None:
        self._engine = engine
        self.job_id = job_id
        self._last_status: dict = status or {}

    async def status(self) -> dict:
        self._last_status = await self._engine._transport.status(self.job_id)
        return self._last_status

    async def state(self) -> JobState:
        return JobState.parse((await self.status()).get("state", ""))

    async def queue_position(self) -> int | None:
        return (await self.status()).get("queuePosition")

    async def cancel(self) -> dict:
        return await self._engine._transport.cancel(self.job_id)

    async def result(self) -> CraftResult:
        return result_dict_to_views(await self._engine._transport.result(self.job_id))

    async def wait(
        self,
        *,
        poll_interval: float = 2.0,
        timeout: float | None = None,
        on_status: Callable[[dict], None] | None = None,
    ) -> CraftResult:
        deadline = None if timeout is None else time.monotonic() + timeout
        while True:
            status = await self.status()
            if on_status is not None:
                on_status(status)
            state = JobState.parse(status.get("state", ""))
            if state == JobState.SUCCEEDED:
                return await self.result()
            if state.is_terminal:
                RemoteJobError.from_dict(
                    status.get("error")
                    or {"code": state.name, "message": f"job ended as {state.value}"}
                ).raise_as_native()
            if deadline is not None and time.monotonic() > deadline:
                raise TimeoutError(f"job {self.job_id} did not finish within {timeout}s")
            await asyncio.sleep(poll_interval)

    async def events(self) -> AsyncIterator[dict]:
        """Live mode: yield JobEvent dicts until the job is terminal."""
        from ..client._ws import stream_events

        async for event in stream_events(self._engine.base_url, self.job_id):
            yield event
