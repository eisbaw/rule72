# `rule72` is a git commit message formatter / reflower

Smart command-line formatter that rewraps Git commit messages while
**preserving structure** (headline, paragraphs, nested lists, tables, code
blocks, footers, emoji bullets, etc.). It reads from **stdin** and writes the
reformatted message to **stdout** so it plugs into editors, Git hooks, pipes,
or batch jobs.

Performance: ~1.5ms per commit message on a laptop ⚡.\
Run `just profile` for detailed benchmarks across the test corpus.

---
## What

* Enforces 50-char headline and 72-char body width (configurable).
* Understands Markdown-style bullets (`*`, `-`, numbered, emoji).
* Keeps indentation, continuation alignment, fenced code, URLs, tables.
* Chunk-aware – headline, body blocks, footers detected automatically.
* Written in safe, fast Rust.

---
## Quick Usage

```bash
# Rewrap the current COMMIT_EDITMSG from a Git hook
cat "$1" | rule72 > "$1.tmp" && mv "$1.tmp" "$1"

# Ad-hoc from shell
printf '%s\n' "fix: extremely long headline ..." | rule72

# Batch-reformat repository message corpus (Justfile target)
just reflow-data   # Creates data.out/ mirrored directory
just compare-data  # Diff original vs reflowed with colordiff/less
```

CLI flags:
```
  -w, --width <N>           set body wrap width (default 72)
      --headline-width <N>  advisory headline width (default 50)
      --no-ansi             strip colour codes before measuring width
      --debug-svg <PATH>    generate SVG visualization of parsing/classification
      --debug-trace         output detailed trace of parsing pipeline
```

---
## Debug Visualization

For explainability and development, `rule72` provides comprehensive debug output:

* **SVG Visualization**: `--debug-svg output.svg` generates a visual breakdown
  showing how each line is classified (prose, list, code, table, etc.) with
  color coding and probability scores.
* **Debug Tracing**: `--debug-trace` outputs detailed parsing pipeline
  information with automatic file:line prefixes, showing input processing and
  classification decisions.

These features help understand how the tool parses complex commit messages and
can aid in troubleshooting formatting decisions.

---
## Test-Catalogue: `data/` vs `data.out/`

The repo ships with a large set of real-world commit messages under `data/`.\
Running `just reflow-data` pipes every `*.txt` file through `rule72`, writing
the result to **identical relative paths** under `data.out/`.\
`just compare-data` opens a unified color diff so you can inspect:

* Correct wrapping of long paragraphs
* List continuation alignment and nested bullets
* Emoji bullets retained as list markers
* Code/table blocks untouched

This serves as an integration regression suite on top of unit tests.

---
## Algorithm (bounding rectangles)

Computer-vision inspired, but works on text.

1. **Line scan → indent matrix**: count leading spaces/tabs per line.
2. **Marching-squares-style segmentation**: find rectangular regions of equal
   min-indent – yields a nesting tree (outer → inner).
3. **Heuristic classification** of each rectangle:
   * Prose / List / Code / Table / URL.
4. **Pretty-print** per node type: greedy wrap for prose & list items,
   verbatim for others, mandatory blank lines enforced.
5. Reassemble chunks: headline + body blocks + footers.

The rectangle tree allows arbitrarily nested lists or code blocks without
complex parsing.

### Architecture

```
src/
 ├─ main.rs  → arg-parse + stdin/stdout glue
 └─ lib.rs   → parser, classifier, wrapper
```

Key crates: `clap`, `regex`, `unicode-segmentation`, `unicode-width`,
`anyhow`.

Build tooling via **Nix** + **Just** (`shell.nix`, `Justfile`).

---
## Related Tools

* [`commitmsgfmt`](https://mkjeldsen.gitlab.io/blog/introducing-commitmsgfmt/) –
  Vim filter that inspired many rules; `rule72` generalises with rectangle
  parser & Rust CLI.
* `fmt(1)`, `par(1)` – generic text wrappers (no commit-specific
  semantics).
