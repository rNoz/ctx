# Search

`ctx search` finds matching indexed history. Default results are session-diverse:
ctx shows the strongest matching span from each session, then lets you drill
into dense event-level results when needed. By default it first performs a quiet
best-effort refresh of discovered native provider sources, then queries the
local SQLite store.

## Search

Examples:

```bash
ctx search "build failure"
ctx search "sqlite storage" --provider codex
ctx search "retry handling" --repo checkout --since 60d
ctx search "tool output" --event-type tool_output
ctx search --file crates/foo/src/lib.rs
ctx search "token budget" --refresh off
ctx search "token budget" --limit 5 --json
ctx search "token budget" --session <ctx-session-id> --json
ctx search "this current task" --include-current-session
```

A result can include:

- `ctx_event_id`, the ctx-owned event ID for event hits;
- `ctx_session_id`, the ctx-owned session ID when known;
- `provider_session_id`, the provider-owned session ID when known;
- title or event label;
- snippet with redaction and truncation where needed;
- rank, result scope, and match reasons;
- session importance and more-matches count for default session results;
- provider;
- event sequence;
- timestamp;
- working directory when known;
- source path and cursor when available;
- source availability flag when known;
- citations;
- `suggested_next_commands`, copyable commands for `ctx show`, `ctx locate`,
  and scoped follow-up searches.

Search result IDs are ctx-owned. Provider-owned IDs are exposed as metadata so
humans can recognize the original provider session, but they are not positional
lookup IDs. Provider-owned lookup must be explicit, for example
`--provider codex --provider-session <provider-session-id>` on commands that
support it.

## Filters

Search filters narrow both human output and JSON:

- `--provider codex|pi|claude|opencode|antigravity|gemini|cursor|copilot-cli|factory-ai-droid`;
- `--repo <name-or-path>`;
- `--since <rfc3339-or-days>d`;
- `--event-type <event-type>`;
- `--file <path>`;
- `--session <ctx-session-id>`;
- `--events`;
- `--primary-only`;
- `--include-subagents`;
- `--limit <n>`;
- `--refresh auto|off|strict`;
- `--include-current-session`.

`--since` accepts RFC 3339 timestamps such as `2026-06-01T00:00:00Z` or a day
window such as `30d`.

The default includes subagent material. `--primary-only` restricts results to
primary sessions and excludes subagent material. `--include-subagents` keeps the
default explicit; it does not override `--primary-only`.

`--limit` defaults to `20` and is capped at `200`.

Default search returns diverse session-level results. Use
`--session <ctx-session-id>` after a default search has identified a session to
inspect; scoped session search returns dense event hits. Use `--events` without
`--session` when you want dense event hits across sessions.

When ctx is run from Codex and `CODEX_THREAD_ID` is available, search excludes
the active Codex session tree by default so the current prompt and its subagent
work do not dominate history research. Use `--include-current-session` when you
are intentionally looking for material from the active session tree.

`--refresh` defaults to `auto`. `auto` attempts a best-effort pre-search import
of discovered native provider sources and serves the existing index if that
refresh fails. On large discovered sources or already-cataloged indexes, `auto`
serves current results without a foreground catch-up scan; use
`--refresh strict` or `ctx import --all` when you need a full catch-up before
querying. `off` skips the pre-search refresh. `strict` fails the search if the
refresh cannot run or import successfully. Search-only sources without native
import support are searched from the existing index until they are explicitly
imported through a supported path.

Use `--refresh off` for a strictly read-only search over the existing ctx index.
This avoids provider imports and avoids updating the ctx SQLite store.

## Machine Output

Use `ctx search --json` for agent workflows and scripts. JSON results include
the same result metadata and citations as the human output, plus a top-level
`freshness` object describing the pre-search refresh mode and outcome. A
citation with `source_exists: false` means ctx can return indexed text, but the
raw provider file was not available at the stored path when the result was
built.
