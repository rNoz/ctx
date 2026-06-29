#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${repo_root}"

required_paths=(
  README.md
  LICENSE
  docs/product-contract.md
  docs/getting-started.md
  docs/first-10-minutes.md
  docs/cli-reference.md
  docs/contracts/json.md
  docs/storage.md
  docs/privacy-storage.md
  docs/providers.md
  docs/provider-support.md
  docs/provider-support-matrix.json
  docs/search.md
  docs/limitations.md
  docs/security-checks.md
  docs/agent-usage.md
  docs/testing-taxonomy.md
  docs/troubleshooting.md
  docs/threat-model.md
  docs/provider-adapter-api.md
  docs/redaction-corpus.md
  docs/agent-skill-install.md
  skills/ctx-agent-history-search/SKILL.md
  plugins/ctx-agent-history-search/skills/ctx-agent-history-search/SKILL.md
)

for path in "${required_paths[@]}"; do
  test -f "${path}"
done

if command -v jq >/dev/null 2>&1; then
  jq empty docs/provider-support-matrix.json
fi

public_docs=(
  README.md
  SECURITY.md
  docs/*.md
  docs/contracts/*.md
  skills/ctx-agent-history-search/SKILL.md
  plugins/ctx-agent-history-search/skills/ctx-agent-history-search/SKILL.md
)

analytics_scope=()
for path in "${public_docs[@]}"; do
  if [[ "${path}" != "docs/storage.md" ]]; then
    analytics_scope+=("${path}")
  fi
done

scan_docs() {
  local pattern="$1"
  shift

  if command -v rg >/dev/null 2>&1; then
    rg -n -i -e "${pattern}" "$@"
  else
    grep -R -n -i -E -e "${pattern}" "$@"
  fi
}

unsupported_surface_pattern='dashboard|shim|shims|pull request|pull-request|pr evidence|pr-evidence|ctx pr|ctx publish|ctx evidence|ctx update|ctx uninstall|hosted|\bADE\b|automatic summar|\bMVP\b|recover prior decisions|ctx remembers everything|privacy-first|ctx context|--until|ctx list --provider|ctx list --repo|ctx list --since|\b[Aa]mp\b|[Aa]mpcode|normalized-only|normalized only|normalized_import_only|normalized provider JSONL|CTX_PROVIDER_NORMALIZED_IMPORT_DEV|[W]ork Recorder|[w]ork recorder|\bwork-[r]ecord\b'
private_path_pattern='/home/[d]addy|/home/[^[:space:]]+/(code|Documents|Desktop)|/Users/[^[:space:]]+/(code|Documents|Desktop)|ctx-[p]rivate|ctx-multi-repo-workspace|\.ctx/worktrees'

if scan_docs "${unsupported_surface_pattern}" "${public_docs[@]}"; then
    printf 'public docs contain removed or unsupported product surface wording\n' >&2
    exit 1
fi

if scan_docs "${private_path_pattern}" "${public_docs[@]}"; then
  printf 'public docs contain private host/workspace paths\n' >&2
  exit 1
fi

if scan_docs 'analytics|telemetry' "${analytics_scope[@]}"; then
  printf 'public analytics copy must stay limited to docs/storage.md\n' >&2
  exit 1
fi

if ! diff -u skills/ctx-agent-history-search/SKILL.md plugins/ctx-agent-history-search/skills/ctx-agent-history-search/SKILL.md >/dev/null; then
  printf 'plugin skill copy differs from public skill source\n' >&2
  exit 1
fi

printf 'public docs ok\n'
