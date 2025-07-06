# Build release binary
build:
  (cd commit-reflow && cargo build --release)

test:
  (cd commit-reflow && cargo test --all)

fmt:
  (cd commit-reflow && cargo fmt --all)

lint:
  (cd commit-reflow && cargo clippy -- -D warnings)

# Reflow all commit message .txt files under data/ into data.out/
# Preserves directory structure for easy comparison.
reflow-data:
  #!/usr/bin/env bash
  set -euo pipefail
  mkdir -p ../data.out
  for f in $(find ../data -type f -name '*.txt'); do \
    rel="$${f#../data/}"; \
    out="../data.out/$$rel"; \
    mkdir -p "$$(dirname "$$out")"; \
    cat "$$f" | target/release/commit-reflow > "$$out"; \
  done 