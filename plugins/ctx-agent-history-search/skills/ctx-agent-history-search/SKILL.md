---
name: ctx-agent-history-search
description: Use ctx to search local coding-agent history before acting. Use when prior agent sessions may contain relevant commands, attempts, decisions, source citations, or transcript context.
---

# ctx Agent History Search

Use ctx as a local retrieval tool before repeating investigation work. Treat ctx
output as cited source material from local transcripts, not as generated
analysis.

Use this skill in two modes:

- retrieval before work, when prior sessions may contain decisions, commands,
  failures, or source citations that affect the current task;
- history research reports, when the user asks an agent or read-only subagent to
  research a historical topic across prior local agent sessions.

## Prerequisites

- Require the `ctx` CLI to be installed and set up.
- Start with `ctx status --json`.
- If `ctx` is missing or not set up, tell the user the local history index is
  unavailable and do not invent results.

## Workflow

1. Check health:

   ```bash
   ctx status --json
   ```

2. Inspect available provider sources:

   ```bash
   ctx sources --json
   ```

3. Re-import only when recent local history matters or search misses something
   the user says should exist:

   ```bash
   ctx import --all --json
   ctx import --resume --json
   ```

   Treat `--resume` as an idempotent rescan marker, not a guarantee that every
   provider has native cursor resume.

4. Search with normal language first, then add tight filters when useful:

   ```bash
   ctx search "<query>" --json
   ctx search "<query>" --refresh off --json
   ctx search "<query>" --provider codex --json
   ctx search "<query>" --repo <repo> --json
   ctx search "<query>" --file <path> --json
   ctx search "<query>" --since 30d --json
   ctx search "<query>" --session <ctx-session-id> --json
   ```

   Use default `ctx search` to find promising sessions. Use scoped
   `ctx search ... --session <ctx-session-id>` when a session looks
   relevant and you need dense event-level matches from that session.
   Normal search may refresh the local ctx index before querying; use
   `--refresh off` when the prompt requires strictly read-only research over the
   existing index.

   In Codex, ctx excludes the active session tree by default when
   `CODEX_THREAD_ID` is available, so the current prompt and subagents do not
   dominate historical retrieval. Use `--include-current-session` only when the
   active session tree is the target.

5. Inspect the best cited result before relying on it:

   ```bash
   ctx show event <ctx-event-id> --window 5 --format json
   ctx show session <ctx-session-id> --format json
   ```

6. Locate original provider material when source identity or resume hints matter:

   ```bash
   ctx locate event <ctx-event-id> --format json
   ctx locate session <ctx-session-id> --format json
   ```

7. Export a transcript only when another agent or artifact needs a file:

   ```bash
   ctx export session <ctx-session-id> --format markdown --out <output-path>
   ```

## History Research Reports

When asked to research a historical topic, stay read-only unless the user also
asks for edits. The agent writes the report; ctx only retrieves local source
material.

1. Restate the topic, scope, and desired length if the prompt is ambiguous.
   Prefer concise reports by default; use a longer report when the user asks for
   chronology, alternatives, or detailed evidence.
2. Run several targeted searches. Vary query terms across user wording, file or
   module names, error text, commands, branch names, and decision terms. Start
   with default `ctx search`, then narrow with `--repo`, `--provider`, `--file`,
   `--since`, or `--session <ctx-session-id>`.
   Add `--refresh off` when the report must not update the local ctx index.
3. Inspect focused sources before drawing conclusions. Prefer `ctx show event`
   for a hit plus nearby turns, and `ctx show session` when the whole session
   arc matters:

   ```bash
   ctx show event <ctx-event-id> --window 5 --format json
   ctx show session <ctx-session-id> --format json
   ```

   Use full or log mode only when lite output omits necessary evidence.
4. Compare evidence across sessions. Note agreements, conflicts, stale results,
   missing raw sources, and gaps where searches did not find evidence.
5. Produce the report as agent synthesis with citations. Do not claim ctx
   generated, inferred, or validated the report.

Concise report shape:

- answer or finding;
- strongest supporting ctx IDs;
- important caveats or gaps;
- optional next search or verification step.

Long report shape:

- question and scope;
- search method, including key queries and filters;
- findings or chronology;
- evidence table with provider, ctx session ID, ctx event ID when available, and
  why each source matters;
- conflicts, gaps, and suggested follow-up.

## Citation Rules

- Cite ctx material when it affects your answer or implementation.
- Include the provider, ctx session ID, ctx event ID when available, provider
  session ID when available, and source path or cursor when present.
- If you synthesize across multiple snippets, label the conclusion as your
  synthesis and cite the supporting snippets.
- If a source citation is stale or unavailable, say ctx returned indexed text
  but the raw source could not be opened.

## Safety Rules

- Prefer JSON for ranking and routing.
- Do not say ctx inferred a decision unless the cited text explicitly states
  that decision.
- Do not state that ctx wrote model analysis.
- Do not paste raw transcripts, large JSON payloads, secrets, tokens, or private
  paths into a user-facing report. Summarize reviewed evidence and quote only
  short excerpts needed to support a claim.
- Treat `~/.ctx`, provider transcript paths, and JSON output as private local
  history unless the user explicitly asks to share reviewed excerpts.
- Use typed IDs. Do not fall back to old ambiguous `ctx show <uuid>` behavior.
