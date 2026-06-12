---
type: source
title: "Requirements: chat-driven crafting assistant over MCP"
tags: [mcp, product]
authors: [WladHD]
url:
raw: raw/2026-06-11-mcp-chat-usecases-request.md
ingested: 2026-06-12
created: 2026-06-12
updated: 2026-06-12
---

# Requirements: chat-driven crafting assistant over MCP

Requirement messages from WladHD (2026-06-11 to 2026-06-12) asking that the MCP
server and backend be extended so players can use CraftPath conversationally
through a chat AI, plus a use-case wiki listing consolidating the needed
functionality.

## Key points

- Four seed use cases: "what is currently good?" (meta data per class and
  level, retrieved/cached/scraped), "what enchant can I get with 2 divines on
  average?" (budget expected outcome), "I want a bow with EPPSSA" (compact
  craft-spec notation: E essence, P prefix, S suffix, A abyss), and "I play
  Amazon, what BIS item can I craft with 1 divine?" (class-aware budget BIS).
- Extended to a persona-tiered catalog: at least 5 beginner, 5 regular and
  5 pro questions, including affix recommendations for an existing item,
  inventory-based budgets ("two exalted orbs"), desecration outcome preview,
  route graphs, step-ordering questions (desecration, quality), and
  danger/irreversibility warnings.
- Job UX requirements: queue position, follow/await a running calculation,
  live calculation status.
- PathOfBuilding-PoE2 should be cross-referenced as integration discussion
  points (see [[pathofbuilding-poe2]]).
- Client AI question: prefer mapping onto existing chat AIs (ChatGPT, Claude,
  local models) over training or running a dedicated second model.

## Where this fits

- [[mcp-chat-usecases]] - the persona-tiered use-case catalog distilled from
  these messages.
- [[mcp-capability-roadmap]] - the MCP tool surface, engine gaps and phasing
  the catalog implies.
- [[pathofbuilding-poe2]] - the requested PoB2 integration discussion.
