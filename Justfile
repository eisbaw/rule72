# Build release binary
build:
  (cd rule72 && cargo build --release)

test:
  (cd rule72 && cargo test --all)

fmt:
  (cd rule72 && cargo fmt --all)

lint:
  (cd rule72 && cargo clippy -- -D warnings)

# Reflow all commit message .txt files under data/ into data.out/
# Preserves directory structure for easy comparison.
reflow-data: build
  #!/usr/bin/env bash
  set -euo pipefail
  mkdir -p data.out
  for f in $(find data -type f -name '*.txt'); do \
    rel="${f#data/}"; \
    out="data.out/$rel"; \
    mkdir -p "$(dirname "$out")"; \
    cat "$f" | rule72/target/release/rule72 > "$out"; \
  done
  echo "Look for git diffs in data.out/"

# Compare actual commit messages with their reflowed versions
compare-data:
  colordiff -U10 -r data data.out/ | less -SNR

# Test nix build with callPackage and run --help
nix-test:
  #!/usr/bin/env bash
  set -euo pipefail
  echo "Building with nix..."
  # Build without creating ./result symlink - just get store path
  RESULT=$(nix-build -E "with import <nixpkgs> {}; callPackage ./default.nix {}" --no-link)
  echo "Built: $RESULT"
  echo "Testing --help..."
  "$RESULT/bin/rule72" --help
