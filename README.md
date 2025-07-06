# Commit-Reflow

Smart command-line formatter that rewraps Git commit messages while **preserving structure** (headline, paragraphs, nested lists, tables, code blocks, footers, emoji bullets, etc.).  It reads from **stdin** and writes the reformatted message to **stdout** so it plugs into editors, Git hooks, pipes, or batch jobs.

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
cat "$1" | commit-reflow > "$1.tmp" && mv "$1.tmp" "$1"

# Ad-hoc from shell
printf '%s\n' "fix: extremely long headline ..." | commit-reflow

# Batch-reformat repository message corpus (Justfile target)
just reflow-data   # Creates data.out/ mirrored directory
just compare-data  # Diff original vs reflowed with colordiff/less
```

CLI flags:
```
  -w, --width <N>           set body wrap width (default 72)
      --headline-width <N>  advisory headline width (default 50)
      --no-ansi             strip colour codes before measuring width
```

---
## Test-Catalogue: `data/` vs `data.out/`

The repo ships with a large set of real-world commit messages under `data/`.  Running `just reflow-data` pipes every `*.txt` file through `commit-reflow`, writing the result to **identical relative paths** under `data.out/`.  `just compare-data` opens a unified diff (`colordiff | less -SNR`) so you can inspect:

* Correct wrapping of long paragraphs
* List continuation alignment and nested bullets
* Emoji bullets retained as list markers
* Code/table blocks untouched

This serves as an integration regression suite on top of unit tests.

---
## Algorithm (bounding rectangles)

1. **Line scan → indent matrix**: count leading spaces/tabs per line.
2. **Marching-squares-style segmentation**: find rectangular regions of equal min-indent – yields a nesting tree (outer → inner).
3. **Heuristic classification** of each rectangle:
   * Prose / List / Code / Table / URL.
4. **Pretty-print** per node type: greedy wrap for prose & list items, verbatim for others, mandatory blank lines enforced.
5. Reassemble chunks: headline + body blocks + footers.

The rectangle tree allows arbitrarily nested lists or code blocks without complex parsing.

### Architecture

```
src/
 ├─ main.rs  → arg-parse + stdin/stdout glue
 └─ lib.rs   → parser, classifier, wrapper
```

Key crates: `clap`, `regex`, `unicode-segmentation`, `unicode-width`, `anyhow`.

Build tooling via **Nix** + **Just** (`shell.nix`, `Justfile`).

---
## Related Tools

* [`commitmsgfmt`](https://mkjeldsen.gitlab.io/blog/introducing-commitmsgfmt/) – Vim filter that inspired many rules; `commit-reflow` generalises with rectangle parser & Rust CLI.
* `fmt(1)`, `par(1)` – generic text wrappers (no commit-specific semantics).
* GitHub / GitLab web editors – server side wrapping only.

---
Happy rebasing! 🚀 