"""Submit the example calculation to a pyoe2-craftpath backend.

Requires the client extra (pip install 'pyoe2-craftpath[client]') and a
running backend; set CRAFTPATH_BACKEND_URL, e.g.:

    CRAFTPATH_BACKEND_URL=http://localhost:8080 python example_engine_remote.py
    CRAFTPATH_BACKEND_URL=http://localhost:8080 python example_engine_remote.py --live
"""

import os
import sys

import pyoe2_craftpath as pc

BACKEND_URL = os.environ.get("CRAFTPATH_BACKEND_URL")
if not BACKEND_URL:
    print("CRAFTPATH_BACKEND_URL not set — skipping remote example.")
    sys.exit(0)

POE2_LEAGUE = "Standard"

# item parsing is local; the backend fetches its own league data
local = pc.LocalEngine(cache_dir="./cache")
item_provider, _ = local.providers(POE2_LEAGUE)
start = pc.CraftOfExileEmulatorItemImport.parse_itemsnapshot_from_string(
    open("example_items/start_item_magic_1_affix_bow.json").read(), item_provider
)
target = pc.CraftOfExileEmulatorItemImport.parse_itemsnapshot_from_string(
    open("example_items/expensive_target_item_rare_6_affix_bow.json").read(), item_provider
)

spec = pc.JobSpec(
    start=start,
    target=target,
    league=POE2_LEAGUE,
    path_analyzers=[pc.StatisticAnalyzerPathPreset.UniquePathChance],
    max_routes=5,
    max_ram_in_bytes=1_000_000_000,
)

engine = pc.RemoteEngine(BACKEND_URL)
job = engine.submit(spec)
print(f"submitted job {job.job_id}, queue position: {job.queue_position()}")

if "--live" in sys.argv:
    # WebSocket live mode: push updates instead of polling
    def on_event(event: dict) -> None:
        status = event.get("status", {})
        progress = status.get("progress") or {}
        print(
            f"[live] {status.get('state')} "
            f"pos={status.get('queuePosition')} "
            f"{progress.get('phase', '')} {progress.get('percent', 0)}%"
        )

    result = job.stream(on_event)
else:
    result = job.wait(
        poll_interval=2.0,
        on_status=lambda s: print(
            f"[poll] {s.get('state')} pos={s.get('queuePosition')} "
            f"{(s.get('progress') or {}).get('phase', '')}"
        ),
    )

print(f"matrix size: {result.matrix_size}")
print(result.pretty_text or "(no pretty text requested)")
