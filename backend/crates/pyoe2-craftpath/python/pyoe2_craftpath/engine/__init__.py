"""Engine layer: one interface, two execution backends.

* :class:`LocalEngine` — runs the calculation in-process via the native
  Rust module (exactly the classic flow).
* :class:`RemoteEngine` / :class:`AsyncRemoteEngine` — submits the job to a
  pyoe2-craftpath backend over REST, with queue-position polling and a
  WebSocket live mode. Requires ``pip install pyoe2-craftpath[client]``.
"""

from ._spec import JobSpec, JobState
from ._result import CraftResult, GroupResultView, GroupView, PathResultView, RouteView
from .local import LocalEngine, LocalJob


def __getattr__(name: str):
    # RemoteEngine pulls in httpx/websockets/protobuf; import lazily so the
    # base install works without the [client] extra.
    if name in ("RemoteEngine", "AsyncRemoteEngine", "RemoteJob", "AsyncRemoteJob"):
        from . import remote

        return getattr(remote, name)
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")


__all__ = [
    "JobSpec",
    "JobState",
    "CraftResult",
    "PathResultView",
    "GroupResultView",
    "RouteView",
    "GroupView",
    "LocalEngine",
    "LocalJob",
    "RemoteEngine",
    "AsyncRemoteEngine",
]
