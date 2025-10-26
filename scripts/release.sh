#!/usr/bin/env bash
set -euo pipefail

# Usage: ./scripts/release.sh 0.1.0 [--commit-each]
# if --commit-each is passed it will run scripts/commit-each.sh before final release commit.
VERSION=$1
COMMIT_EACH=false
if [ "${2:-}" = "--commit-each" ]; then
  COMMIT_EACH=true
fi

if [ -z "$VERSION" ]; then
  echo "Usage: $0 <version> [--commit-each]"
  echo "Example: $0 0.1.0"
  exit 1
fi

echo "🚀 Preparing release v$VERSION"
echo

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# sanity: in a git repo?
if ! git rev-parse --git-dir >/dev/null 2>&1; then
  echo "Error: Not in a git repository"
  exit 1
fi

# check working tree clean
if ! git diff-index --quiet HEAD --; then
  echo -e "${YELLOW}⚠️  You have uncommitted changes${NC}"
  echo "Please commit or stash them before creating a release"
  exit 1
fi

# function: update version key in a Cargo.toml (adds under [package] if missing)
update_version_in_file() {
  local f="$1"
  if [ ! -f "$f" ]; then
    return
  fi

  # only operate on files that are crate Cargo.toml (contain [package])
  if ! grep -q '^\[package\]' "$f"; then
    # no package section (likely workspace Cargo.toml) -> skip
    return
  fi

  if grep -q '^[[:space:]]*version[[:space:]]*=' "$f"; then
    # replace existing version
    sed -i.bak "s/^[[:space:]]*version[[:space:]]*=.*/version = \"$VERSION\"/" "$f"
  else
    # insert version line after [package] header
    awk -v ver="$VERSION" '
      BEGIN{p=0}
      /^\[package\]/{print; p=1; next}
      { if(p==1){ print "version = \"" ver "\""; p=0 } print }
    ' "$f" > "$f.tmp" && mv "$f.tmp" "$f"
  fi
}

echo -e "${BLUE}📝 Updating version numbers...${NC}"

# Update crate Cargo.toml files under mdns-tool/crates/*
for cargo_toml in mdns-tool/crates/*/Cargo.toml; do
  update_version_in_file "$cargo_toml"
done

# NOTE: do NOT set version on workspace Cargo.toml (it is not a package)
# Remove backup files if any
find . -name "*.bak" -delete

echo -e "${GREEN}✓ Version updated to $VERSION in crate Cargo.toml files${NC}"
echo

# Optionally run per-file commits (user requested "commit each file")
if [ "$COMMIT_EACH" = true ]; then
  if [ ! -x "scripts/commit-each.sh" ]; then
    echo -e "${YELLOW}⚠️  commit-each.sh missing or not executable at scripts/commit-each.sh${NC}"
    exit 1
  fi
  echo -e "${BLUE}🧾 Creating per-file conventional commits...${NC}"
  ./scripts/commit-each.sh
  echo -e "${GREEN}✓ Per-file commits complete${NC}"
  echo
fi

# Build release
echo -e "${BLUE}📦 Building release binaries (workspace)...${NC}"
pushd mdns-tool >/dev/null
cargo build --release --workspace
popd >/dev/null

echo -e "${GREEN}✓ Build successful${NC}"
echo

# Run tests (non-failing if no tests)
echo -e "${BLUE}🧪 Running tests...${NC}"
pushd mdns-tool >/dev/null
# run tests; do not use --release for test compilation performance if you prefer,
# but using it matches your original script
cargo test --workspace --release
popd >/dev/null

echo -e "${GREEN}✓ All tests passed (or none)${NC}"
echo

# Update CHANGELOG.md: replace "## [Unreleased]" or prepend entry if not found
echo -e "${BLUE}📋 Updating CHANGELOG.md...${NC}"
TODAY=$(date +%Y-%m-%d)
if grep -q '^## \[Unreleased\]' CHANGELOG.md 2>/dev/null; then
  sed -i.bak "0,/^## \[Unreleased\]/{s/^## \[Unreleased\]/## [Unreleased]\n\n## [$VERSION] - $TODAY/}" CHANGELOG.md
  # The above duplicates Unreleased header and inserts new version after; adjust if needed.
  # Simpler fallback: directly replace first occurrence
  sed -i.bak "s/## \[Unreleased\]/## [Unreleased]\n\n## [$VERSION] - $TODAY/" CHANGELOG.md || true
else
  # prepend a new section at top
  { printf "## [$VERSION] - %s\n\n- Release prepared.\n\n" "$TODAY"; cat CHANGELOG.md; } > CHANGELOG.md.tmp && mv CHANGELOG.md.tmp CHANGELOG.md
fi
rm -f CHANGELOG.md.bak
echo -e "${GREEN}✓ CHANGELOG updated${NC}"
echo

# Create release directory
RELEASE_DIR="release-v$VERSION"
mkdir -p "$RELEASE_DIR"

# Find produced executables in target/release and copy them
echo -e "${BLUE}📦 Packaging release artifacts...${NC}"
BIN_DIR="mdns-tool/target/release"
if [ ! -d "$BIN_DIR" ]; then
  echo -e "${YELLOW}⚠️  Build output not found at $BIN_DIR${NC}"
fi

# copy all regular executable files (skip *.d / *.rlib etc.)
shopt -s nullglob
copied=false
for f in "$BIN_DIR"/*; do
  if [ -f "$f" ] && [ -x "$f" ]; then
    base=$(basename "$f")
    cp "$f" "$RELEASE_DIR/$base"
    copied=true
  fi
done
shopt -u nullglob

if [ "$copied" = false ]; then
  echo -e "${YELLOW}⚠️  No executables found to package. Check binary names or build target.${NC}"
else
  # create tarballs for each binary found
  for bin in "$RELEASE_DIR"/*; do
    bn=$(basename "$bin")
    tar -czf "$RELEASE_DIR/${bn}-v${VERSION}-linux-x86_64.tar.gz" -C "$RELEASE_DIR" "$bn"
  done

  # checksums
  pushd "$RELEASE_DIR" >/dev/null
  sha256sum ./* > SHA256SUMS
  popd >/dev/null

  echo -e "${GREEN}✓ Release artifacts created in $RELEASE_DIR/${NC}"
fi
echo

# Git operations: add changed files (Cargo.toml, CHANGELOG, release dir) and create commit+tag
echo -e "${BLUE}📝 Creating git commit and tag...${NC}"
git add -A

# conventional commit for release
git commit -m "chore(release): v$VERSION" || {
  echo -e "${YELLOW}⚠️  Nothing to commit (maybe commit-each already made commits)${NC}"
}

git tag -a "v$VERSION" -m "Release v$VERSION"

echo -e "${GREEN}✓ Commit and tag created${NC}"
echo

echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║     Release v$VERSION prepared!        ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
echo
echo -e "${BLUE}Next steps:${NC}"
echo
echo "1. Review the changes:"
echo "   git show"
echo
echo "2. Push the commit(s) and tag:"
echo "   git push origin main"
echo "   git push origin v$VERSION"
echo
echo "3. Create GitHub release (or use gh cli):"
echo "   - Go to https://github.com/OpenTabCommunity/open-share-client-rust/releases/new"
echo "   - Select tag: v$VERSION"
echo "   - Title: OpenShare v$VERSION"
echo "   - Copy release notes from CHANGELOG.md"
echo "   - Upload files from $RELEASE_DIR/"
echo
echo "4. Announce the release:"
echo "   - Update docs, social media, notify users"
echo
echo -e "${YELLOW}Release artifacts location:${NC}"
echo "   $RELEASE_DIR/"
ls -lh "$RELEASE_DIR" || true
