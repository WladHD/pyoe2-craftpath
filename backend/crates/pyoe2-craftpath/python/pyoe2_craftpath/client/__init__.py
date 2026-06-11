"""Low-level HTTP/WebSocket client for the pyoe2-craftpath backend.

Requires the ``client`` extra: ``pip install pyoe2-craftpath[client]``.
"""

from ._errors import RemoteJobError

__all__ = ["RemoteJobError"]
