"""httpx transports with JSON <-> protobuf content negotiation.

All payloads cross this layer as canonical-JSON dicts. With ``proto=True``
the dicts are converted via ``google.protobuf.json_format`` to the generated
messages and sent/received as ``application/x-protobuf``.
"""

from __future__ import annotations

import httpx

from ._errors import RemoteJobError

PROTOBUF = "application/x-protobuf"
JSON = "application/json"


def _pb2():
    from .._proto import job_pb2

    return job_pb2


def _json_format():
    from google.protobuf import json_format

    return json_format


def encode_submit(body: dict, proto: bool) -> tuple[bytes, str]:
    if not proto:
        import json

        return json.dumps(body).encode(), JSON
    message = _json_format().ParseDict(body, _pb2().SubmitJobRequest())
    return message.SerializeToString(), PROTOBUF


def decode_response(content: bytes, content_type: str, proto_message_factory) -> dict:
    if content_type.split(";")[0].strip() in (PROTOBUF, "application/protobuf"):
        message = proto_message_factory()
        message.ParseFromString(content)
        return _json_format().MessageToDict(message)
    import json

    return json.loads(content)


def check_error(response: httpx.Response) -> None:
    if response.status_code < 400:
        return
    try:
        body = response.json()
    except Exception:  # noqa: BLE001
        body = {"code": "INTERNAL", "message": response.text}
    raise RemoteJobError.from_dict(body)


class Transport:
    """Synchronous REST transport."""

    def __init__(
        self,
        base_url: str,
        *,
        proto: bool = True,
        timeout: float = 30.0,
        api_key: str | None = None,
        client: httpx.Client | None = None,
    ) -> None:
        self.proto = proto
        headers = {}
        if api_key:
            headers["Authorization"] = f"Bearer {api_key}"
        if proto:
            headers["Accept"] = PROTOBUF
        self._client = client or httpx.Client(
            base_url=base_url, timeout=timeout, headers=headers
        )
        if client is not None:
            # injected client (tests): still apply negotiation headers
            self._client.headers.update(headers)
        self.base_url = base_url

    def close(self) -> None:
        self._client.close()

    def submit(self, body: dict) -> dict:
        payload, content_type = encode_submit(body, self.proto)
        response = self._client.post(
            "/api/v1/jobs", content=payload, headers={"Content-Type": content_type}
        )
        check_error(response)
        return decode_response(
            response.content,
            response.headers.get("content-type", JSON),
            lambda: _pb2().SubmitJobResponse(),
        )

    def status(self, job_id: str) -> dict:
        response = self._client.get(f"/api/v1/jobs/{job_id}")
        check_error(response)
        return decode_response(
            response.content,
            response.headers.get("content-type", JSON),
            lambda: _pb2().JobStatus(),
        )

    def result(self, job_id: str) -> dict:
        response = self._client.get(f"/api/v1/jobs/{job_id}/result")
        check_error(response)
        return decode_response(
            response.content,
            response.headers.get("content-type", JSON),
            lambda: _pb2().JobResult(),
        )

    def cancel(self, job_id: str) -> dict:
        response = self._client.delete(f"/api/v1/jobs/{job_id}")
        check_error(response)
        return decode_response(
            response.content,
            response.headers.get("content-type", JSON),
            lambda: _pb2().JobStatus(),
        )


class AsyncTransport:
    """Asynchronous REST transport (mirror of :class:`Transport`)."""

    def __init__(
        self,
        base_url: str,
        *,
        proto: bool = True,
        timeout: float = 30.0,
        api_key: str | None = None,
        client: httpx.AsyncClient | None = None,
    ) -> None:
        self.proto = proto
        headers = {}
        if api_key:
            headers["Authorization"] = f"Bearer {api_key}"
        if proto:
            headers["Accept"] = PROTOBUF
        self._client = client or httpx.AsyncClient(
            base_url=base_url, timeout=timeout, headers=headers
        )
        if client is not None:
            self._client.headers.update(headers)
        self.base_url = base_url

    async def aclose(self) -> None:
        await self._client.aclose()

    async def submit(self, body: dict) -> dict:
        payload, content_type = encode_submit(body, self.proto)
        response = await self._client.post(
            "/api/v1/jobs", content=payload, headers={"Content-Type": content_type}
        )
        check_error(response)
        return decode_response(
            response.content,
            response.headers.get("content-type", JSON),
            lambda: _pb2().SubmitJobResponse(),
        )

    async def status(self, job_id: str) -> dict:
        response = await self._client.get(f"/api/v1/jobs/{job_id}")
        check_error(response)
        return decode_response(
            response.content,
            response.headers.get("content-type", JSON),
            lambda: _pb2().JobStatus(),
        )

    async def result(self, job_id: str) -> dict:
        response = await self._client.get(f"/api/v1/jobs/{job_id}/result")
        check_error(response)
        return decode_response(
            response.content,
            response.headers.get("content-type", JSON),
            lambda: _pb2().JobResult(),
        )

    async def cancel(self, job_id: str) -> dict:
        response = await self._client.delete(f"/api/v1/jobs/{job_id}")
        check_error(response)
        return decode_response(
            response.content,
            response.headers.get("content-type", JSON),
            lambda: _pb2().JobStatus(),
        )
