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

# Profile performance on entire corpus (wall-clock time)
profile: build
  #!/usr/bin/env bash
  set -euo pipefail
  echo "Profiling rule72 over test corpus (discarding output)..."

  # Run sequentially for deterministic timing; change to xargs -P for parallel
  echo "command,mean,stddev,median,user,system,min,max" > rule72-profile.csv
  find data -type f -name '*.txt' -print0 | \
    while IFS= read -r -d '' f; do \
      hyperfine \
        --shell=none \
        --warmup 3 \
        -r 100 \
        --input "$f" \
        --output null \
        --export-csv tmp.csv \
        "rule72/target/release/rule72"
      tail -n 1 tmp.csv >> rule72-profile.csv
    done

# Debug a single commit message, diff and show SVG
debug txtfile: build
  rule72/target/release/rule72 --debug-trace --debug-svg {{txtfile}}.svg < {{txtfile}} > {{txtfile}}.tmp
  -colordiff -U10 {{txtfile}} {{txtfile}}.tmp
  @feh {{txtfile}}.svg || true
