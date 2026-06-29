# First 10 Minutes

This path gets a fresh human or agent from an empty ctx root to a first cited
search result.

## 1. Confirm The Binary

```bash
ctx status
```

If ctx is not installed:

```bash
curl -fsSL https://cli.ctx.rs/install | sh
```

The Unix installer requires `curl` and OpenSSL to verify signed release
metadata. On Windows, use `irm https://cli.ctx.rs/install.ps1 | iex`.

## 2. Set Up And Index

```bash
ctx setup
ctx status --json
```

`ctx setup` creates local storage, discovers supported provider history,
catalogs Codex sessions, imports discovered sources, and optimizes the local
search index. The default root is `~/.ctx`. Use a temporary root for trials:

```bash
ctx --data-root /tmp/ctx-first-10 setup
```

## 3. Check Sources

```bash
ctx sources
ctx sources --json
```

Expect rows for supported local import providers such as Codex, Pi,
Antigravity, Claude, OpenCode, Gemini, Cursor, Copilot CLI, and Factory AI
Droid. A row with `exists: false` means ctx knows the default path but did not
find local history there. A JSON row with `status: "empty"` means the path
exists but no provider-specific transcript files were found. A row with
`status: "unknown"` means the bounded transcript probe hit its scan budget.

## 4. Re-Run Or Target Imports

```bash
ctx import --all
```

Setup already imports discovered sources. Use `ctx import` when you want to
repair, re-run, resume, or pass an explicit path:

```bash
ctx import --provider codex --path ~/.codex/sessions
ctx import --provider pi --path ~/.pi/sessions.jsonl
ctx import --provider cursor --path ~/.cursor/projects
```

## 5. Search

```bash
ctx search "build failure" --limit 5
ctx search "build failure" --limit 5 --json
```

`--limit` is capped at `200`. Search defaults to `--refresh auto`, which
best-effort refreshes discovered native provider sources before querying; use
`--refresh off` to search only the existing index.

Inside Codex, ctx excludes the active session tree by default when it can
identify it, so your current prompt and subagents do not dominate results. Add
`--include-current-session` when that is what you want to search.

Copy ctx-owned IDs from the result and inspect the hit or transcript:

```bash
ctx show event <ctx-event-id> --window 3
ctx show session <ctx-session-id>
ctx locate event <ctx-event-id>
```

Use citations from `ctx search --json` or `ctx show` when the retrieved material
affects an answer or implementation.

## Failure Paths

- No sources listed: this machine may not have supported local provider
  history. Use `ctx import --path` only for a known supported format.
- Import fails on a file: rerun with `--json` and inspect the per-source
  `failed` count.
- Search returns no results: confirm `ctx status` shows indexed items, then
  widen the query or remove filters.
- Citation source missing: ctx can still return indexed text, but the raw
  provider file is unavailable at the stored path.
