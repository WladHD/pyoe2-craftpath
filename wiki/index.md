# Wiki Index

The catalog of all pages in this wiki. Each entry: a wikilink to the page and a one-line summary. The LLM reads this first when answering queries to identify candidate pages.

Keep summaries tight - one line each. The index is engineered to be cheap to read; a fat index defeats its purpose.

When this file exceeds ~300 lines or the wiki passes ~150 pages, shard into `wiki/indexes/<type>.md` and replace this file with a directory of shards. See the `scaling-playbook.md` reference in the `llm-wiki` skill for the migration procedure.

---

## Sources

(populated as sources are ingested)

## Entities

(populated as entity pages are created)

## Concepts

- [[architecture]] - end-to-end walkthrough: data flow, matrix building, the K-best route engine, the four surfaces (Rust/Python/REST/MCP)
- [[development-strategy]] - the happy-path constraint, algorithm caveats, contribution/extension points
- [[how-to-run]] - running CraftPath as Python library, CLI, backend services or Rust crate
- [[commit-conventions]] - conventional-commits + gitmoji format, body rules, trailers, granularity

## Synthesis

(populated as query answers are filed back)
