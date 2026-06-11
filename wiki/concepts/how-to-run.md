---
type: concept
title: "How to run CraftPath"
tags: [usage]
sources: []
created: 2026-06-11
updated: 2026-06-11
---

# How to run CraftPath

CraftPath can be used four ways: as a Python library (in-process or against a
backend), as the interactive CLI, as a self-hosted REST/MCP backend, or as a
Rust crate. This page replaces the old README "How To Run" section and
reflects the `backend/` workspace layout.

## Python library

```bash
pip install pyoe2-craftpath            # native engine only
pip install 'pyoe2-craftpath[client]'  # + remote backend client
```

```python
import pyoe2_craftpath as pc

engine = pc.LocalEngine()                       # in-process (classic)
# engine = pc.RemoteEngine("http://backend")    # or: submit to a backend

result = engine.run(pc.JobSpec(start=..., target=..., league="Standard"))
print(result.pretty_text)
```

All classic classes (`Calculator`, `ItemSnapshot`, providers, presets) remain
importable directly - see the commented examples in
`backend/python_examples/` (`example_engine_local.py`,
`example_engine_remote.py`, and the Jupyter notebook).

Start/target items come from the [CraftOfExile](https://www.craftofexile.com/?game=poe2)
Emulator → Export, parsed via
`CraftOfExileEmulatorItemImport.parse_itemsnapshot_from_string`.

## CLI

The classic interactive CLI is a subcommand of the backend binary
(`pyoe2-backend cli`); the Windows executable from
[Releases](https://github.com/WladHD/pyoe2-craftpath/releases) wraps it.
There is a [video walkthrough](https://www.youtube.com/watch?v=27J1Kjs8q5E)
of the workflow.

Options (all optional):

| Option | Default | Meaning |
|---|---|---|
| `--start_item_path <json>` | `pyoe2-craftpath/startitem.json` | CoE Emulator export of the starting item |
| `--target_item_path <json>` | `pyoe2-craftpath/targetitem.json` | CoE Emulator export of the target item |
| `--cache_path <dir>` | `pyoe2-craftpath` | cache dir for CoE/poe.ninja data (must exist) |
| `--poe2_league <league>` | (current league) | poe.ninja economy league |
| `--amount_routes <n>` | `5` | routes printed per statistic category |
| `--no_updates` | off | skip the GitHub version check |
| `--no_groups` | off | skip the memory-hungry currency-group analysis |
| `--max-ram <n[GB\|MB\|KB]>` | `1GB` | RAM budget for path collection |

## Backend services (REST / worker / MCP)

One binary, selected by subcommand, deployable via the Helm chart in
`deploy/helm/craftpath`:

```bash
docker run -d --rm -p 6379:6379 redis:7-alpine
cargo run -p craftpath-server -- worker &
cargo run -p craftpath-server -- rest      # REST + WebSocket on :8080
cargo run -p craftpath-server -- mcp       # MCP (streamable HTTP or --transport stdio)
```

Submit jobs as JSON or protobuf to `POST /api/v1/jobs`, poll
`GET /api/v1/jobs/{id}` (includes queue position), stream live progress on
`GET /api/v1/jobs/{id}/ws`, fetch `GET /api/v1/jobs/{id}/result`. The wire
contract lives in `/proto/craftpath/v1`. See `backend/README.md` for
endpoints and configuration env vars.

## Rust crate

`craftpath-core` on crates.io; start from the `prelude` and the preset enums
(`MatrixBuilderPreset`, `StatisticAnalyzerPathPreset`,
`StatisticAnalyzerCurrencyGroupPreset`) or the `CraftSession` facade. See
[[development-strategy]] for extension points.

## Where this fits

- [[architecture]] - how a request flows through the system
- [[development-strategy]] - extension points and algorithm caveats
