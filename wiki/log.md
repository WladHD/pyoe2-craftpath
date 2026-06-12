# Wiki Log

Append-only chronological record of operations on the wiki. Each entry begins with `## [YYYY-MM-DD] <op> | <description>` so it's parseable with `grep "^## \[" log.md | tail -N`.

Operations:
- `ingest` - a source was processed into the wiki.
- `query` - a question was answered against the wiki (typically only logged when the answer was filed back as synthesis).
- `lint` - a health check was run.
- `schema` - the schema was modified.
- `shard` - an index was sharded.

---

- 2026-06-11: initialized wiki; seeded concepts from the README split (how-to-run, development-strategy incl. contribution section, commit-conventions moved from repo root) plus a new architecture deep-dive reflecting the 2026-06 engine rework.
- 2026-06-12: ingest | chat use-case requirements (raw/2026-06-11-mcp-chat-usecases-request.md) -> source mcp-chat-usecases-request, concepts mcp-chat-usecases + mcp-capability-roadmap, entity pathofbuilding-poe2.
- 2026-06-12: query | implemented the Phase-1 capability drop in backend/crates (10 new MCP tools, features/inspect + features/craftspec in core, PoB2 import, curated meta catalog); statuses updated in mcp-chat-usecases + mcp-capability-roadmap.
