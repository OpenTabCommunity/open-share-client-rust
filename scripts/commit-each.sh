#!/usr/bin/env bash
set -euo pipefail

# Commits every changed file individually with a conventional commit message heuristic.
# Run from repo root.

# You can tweak these heuristics as needed.

map_scope_from_path() {
  local path="$1"
  # map folders to scopes
  if [[ "$path" =~ ^mdns-tool/crates/([^/]+)/ ]]; then
    echo "${BASH_REMATCH[1]}"
    return
  fi
  case "$path" in
    README.md|CHANGELOG.md|CONTRIBUTING.md) echo "docs"; return;;
    LICENSE) echo "license"; return;;
    scripts/*) echo "scripts"; return;;
    mdns-tool/Cargo.toml) echo "workspace"; return;;
    mdns-tool/*) echo "mdns-tool"; return;;
    *) echo "repo"; return;;
  esac
}

map_type_from_path() {
  local path="$1"
  # Source code files -> feat; Cargo.toml -> chore (bump), docs -> docs, CI -> ci, scripts -> chore
  if [[ "$path" =~ \.(rs|c|cpp|py|js|ts)$ ]]; then
    echo "feat"
    return
  fi

  if [[ "$path" =~ ^(README.md|CHANGELOG.md|CONTRIBUTING.md)$ ]]; then
    echo "docs"
    return
  fi

  if [[ "$path" =~ Cargo.toml$ ]]; then
    echo "chore"
    return
  fi

  if [[ "$path" =~ ^(scripts/|\.github/|Dockerfile) ]]; then
    echo "chore"
    return
  fi

  echo "chore"
}

# gather changed files (but since release.sh updates files, git status should be clean before running it)
CHANGED_FILES=$(git status --porcelain | awk '{print $2}')

if [ -z "$CHANGED_FILES" ]; then
  echo "No changed files to commit."
  exit 0
fi

# Stage nothing initially to avoid surprises
for f in $CHANGED_FILES; do
  # heuristics to skip binary or generated files if you want
  if [[ "$f" == target/* || "$f" == release-*/* || "$f" == mdns-tool/target/* ]]; then
    echo "Skipping generated/binary: $f"
    continue
  fi

  type=$(map_type_from_path "$f")
  scope=$(map_scope_from_path "$f")
  # make friendly subject
  filename=$(basename "$f")
  subject=""
  case "$type" in
    feat) subject="add/modify ${filename}";;
    fix)  subject="fix ${filename}";;
    docs) subject="docs: update ${filename}";;
    chore) subject="chore: update ${filename}";;
    ci) subject="ci: update ${filename}";;
    *) subject="chore: update ${filename}";;
  esac

  # Stage file and commit
  git add -- "$f"
  # Compose conventional commit message: type(scope): subject
  commit_msg="${type}(${scope}): ${subject}"
  git commit -m "$commit_msg" || {
    echo "Commit failed for $f (maybe no changes). Continuing."
  }
  echo "Committed $f -> $commit_msg"
done

echo "All individual file commits created (where applicable)."
