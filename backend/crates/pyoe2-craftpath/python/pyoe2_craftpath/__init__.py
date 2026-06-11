"""pyoe2-craftpath: Path of Exile 2 crafting-path calculator.

Two ways to use it:

* The native classes (``Calculator``, ``ItemSnapshot``, providers, presets...)
  re-exported here unchanged - the in-process Rust engine.
* The engine layer (:class:`LocalEngine` / :class:`RemoteEngine`) - one
  ``JobSpec``-based interface that either computes in-process or submits the
  job to a pyoe2-craftpath backend (``pip install pyoe2-craftpath[client]``).
"""

from ._native import *  # noqa: F401,F403 - full backward-compatible surface
from ._native import __doc__ as _native_doc  # noqa: F401

from .engine import (  # noqa: F401
    AsyncRemoteEngine,
    CraftResult,
    JobSpec,
    JobState,
    LocalEngine,
    RemoteEngine,
)
