# Wiki Index

The catalog of all pages in this wiki. Each entry: a wikilink to the page and a one-line summary. The LLM reads this first when answering queries to identify candidate pages.

Keep summaries tight - one line each. The index is engineered to be cheap to read; a fat index defeats its purpose.

When this file exceeds ~300 lines or the wiki passes ~150 pages, shard into `wiki/indexes/<type>.md` and replace this file with a directory of shards. See the `scaling-playbook.md` reference in the `llm-wiki` skill for the migration procedure.

---

## Sources

- [[mcp-chat-usecases-request]] - WladHD's 2026-06 requirement messages for the chat-driven crafting assistant over MCP

## Entities

- [[pathofbuilding-poe2]] - PoB2 build planner: capabilities, and five CraftPath integration discussion points (build import, stat weights, DPS oracle, item round-trip, mod-DB validation)

## Concepts

- [[architecture]] - end-to-end walkthrough: data flow, matrix building, the K-best route engine, the four surfaces (Rust/Python/REST/MCP)
- [[development-strategy]] - the happy-path constraint, algorithm caveats, contribution/extension points
- [[how-to-run]] - running CraftPath as Python library, CLI, backend services or Rust crate
- [[commit-conventions]] - conventional-commits + gitmoji format, body rules, trailers, granularity
- [[mcp-chat-usecases]] - persona-tiered catalog (8 beginner / 14 regular / 6 pro) of chat questions, each mapped to pipeline, status and required capability
- [[mcp-capability-roadmap]] - MCP tool surface, engine/server gaps, EPPSSA craft-spec DSL, meta/price data sources, client-AI strategy, 5-phase roadmap

## Synthesis

(populated as query answers are filed back)
