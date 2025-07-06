# Git Commit Message Reflow Tool – PRD

## 1. Problem Statement
Developers frequently write Git commit messages with inconsistent line-widths and formatting. This hampers readability in terminals and web tools that assume 72-column wrapped body text and ≤50-character subject lines. Manual re-wrapping is tedious and error-prone.

## 2. Objective
Provide a **stream-oriented** command-line filter (`commit-reflow`) that reads an unformatted commit message on **stdin**, applies smart reflow rules, and writes the formatted message to **stdout**. The tool focuses solely on wrapping and indentation—it is **not** a linter or spell-checker.

## 3. Scope
### In scope
- Parsing commit messages into logical **chunks**:
  1. *Headline / subject line*
  2. *Body* → paragraphs, bullet/numbered/emoji lists, nested lists, tables, code blocks
  3. *Footer* blocks (e.g., `Signed-off-by:`, `Co-authored-by:`)
- Re-wrapping prose paragraphs at **72 columns** (configurable)
- Preserving indentation and list structure when wrapping
- Leaving untouched constructs that should never wrap (URLs, code fences, comments)
- Unix-style streaming (stdin→stdout) for easy Git hook/editor integration

### Out of scope
- Content validation (Conventional Commits, spelling, etc.)
- Interactive UI
- Non-commit prose formatting

## 4. Functional Requirements
1. **Chunk Detection**
   - Detect first non-comment line as *headline*; keep as single line ≤50 columns if possible (no auto-split).
   - Require exactly one blank line between headline and body (insert if missing).
   - Classify remaining lines until first recognised footer tag as *body*.
   - Detect footers via `^\w[\w-]*:` pattern; treat consecutive footer lines as one footer block, preserve order.
2. **Paragraph Reflow**
   - For plain paragraphs (no leading whitespace except optional indent) wrap at **wrap-width** (default 72).
   - Break at word boundaries; a word that exceeds width moves entirely to next line.
3. **List Handling**
   - Recognise list prefix patterns at identical indent depth:
     * `*` or `-` followed by space
     * ASCII digits+`.[)]:`
     * Single emoji followed by space
   - Preserve bullet symbol, spacing, and relative indent.
   - Continuation lines are indented to align with first text character of list item.
   - Support nested lists via increased indent (≥2 spaces or one tab).
4. **Tables & Code Blocks**
   - Detect Markdown tables (`|` separated rows) and 3-backtick fenced blocks; leave untouched.
5. **Non-Wrappable Lines**
   - Lines starting with `#` (Git comments), `>`, four spaces, or tab → copy verbatim.
   - Lines containing a single unindented URL longer than width → do **not** wrap.
6. **Footers**
   - Each footer line stays intact; no wrapping.
   - Ensure exactly one blank line between body and first footer.
7. **Configuration**
   - Flags:
     * `-w`, `--width` <N>: set wrap width (default 72)
     * `--headline-width` <N>: advisory width for headline (default 50; no hard split)
     * `--no-ansi`: strip ANSI color codes before measuring width
8. **Exit Codes**
   - `0` success; formatted text on stdout
   - `1` input parse error (should not happen; fallback copies input)

## 5. Non-Functional Requirements
- **Performance**: reformat 10 kB message in <1 ms on modern CPU.
- **Determinism**: same input → same output.
- **Safety**: never delete input lines; at worst copy through unchanged.
- **Portability**: POSIX shell usage; distributed as static Rust binary.

## 6. Algorithm Overview

### Grammar
```
CommitMessage
├── Headline                # single line, ≤ 50 cols
├── Body                    # zero or more Block nodes
│   └── Block*
│        ├── Prose          # paragraph(s)
│        ├── List           # may contain nested ListItem*
│        ├── Code           # fenced ``` or 4-space indented
│        ├── Table          # Markdown pipe table
│        └── URL            # standalone long URL line
└── Footer*                 # one or more footer lines
```

### Bounding-Rectangle Segmentation (marching-squares inspired)
1. Convert input lines to a 2-D **text matrix**: rows = lines, columns = UTF-8 columns. Track each line's **indent depth** (spaces → depth; tab = 4).
2. **Scan top-to-bottom** to discover **regions** where all lines share the same minimal indent (ignoring blank lines inside region). Think of indent depth as a height-map; stepping down into a deeper indent starts a potential _inner_ region, stepping back up closes the current region.
3. Use a marching-squares–style edge-following algorithm to mark the **bounding rectangle** of every region:  
   • `top = first line`, `bottom = last line` of region  
   • `left = indent depth`, `right = max(line length)` within region.
4. The result is a set of rectangles where each rectangle is either **fully outside** or **strictly inside** another rectangle; this naturally forms a **nesting tree** (outer = shallower indent).
5. **Heuristic classification** (executed _after_ geometry is fixed):
   - Bullet/number/emoji prefix on rectangle's first non-blank line → **List**
   - Starts with "```" or ≥4-space constant indent → **Code**
   - Contains ʻ|ʼ separators on every row → **Table**
   - Single non-wrappable URL line (>wrap-width) → **URL**
   - Otherwise → **Prose** (paragraph)
6. **Reflow rules** applied per rectangle type:
   - Prose & List: greedy minimum-line wrap at `wrap-width` (default 72); continuation indent aligns with first text col.
   - Code, Table, URL: copied verbatim.
7. **Rendering**: depth-first walk of rectangle tree → emit text; insert blank lines between sibling rectangles as in source; enforce mandatory blank lines (after Headline, before first Footer).
8. **Footers** are detected _outside_ the rectangle scan (`^\w[\w-]*:`); rendered last, unchanged.
9. Output to stdout; track column width with Unicode grapheme clusters.

## 7. CLI & Integration Examples
```bash
# Format current commit message from Git hook
cat $1 | commit-reflow > $1.tmp && mv $1.tmp $1

# Ad-hoc
printf '%s\n' "A very loooooong headline that..." | commit-reflow
```

## 8. Risks / Gotchas
- Mis-detecting code blocks could wrap code; mitigate by 4-space rule.
- Excessively long unbreakable words (hashes) exceed width; accept ragged line.
- Merge commit templates with `#` lines stay intact (desired).

## 9. Milestones
1. MVP: headline/body/footer split + paragraph wrap
2. List support (flat → nested)
3. Tables & fenced code passthrough
4. Config flags & binary packaging
5. Optional Vim `formatprg` and Git hook snippets

## 10. Alternatives & Prior Art
- [`commitmsgfmt`](https://mkjeldsen.gitlab.io/blog/introducing-commitmsgfmt/) – inspiration; ours reclassifies into chunks earlier in pipeline.
- Standard `fmt(1)`, `par(1)` – general purpose, unaware of lists/footers.
- GitHub & GitLab web editor – server-side wrapping only.

## 11. Implementation Plan

### Language & Toolchain
- **Rust ≥ 1.70** (edition 2021)
- Target: `x86_64-unknown-linux-musl` for fully static binary
- Lints: `clippy`, formatting via `rustfmt`

### Nix Environment (`shell.nix`)
```nix
{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  buildInputs = [
    pkgs.rustc pkgs.cargo pkgs.clippy pkgs.rustfmt
    pkgs.pkg-config pkgs.openssl # future TLS crates if needed
    pkgs.musl    # static linking target
  ];
  RUSTFLAGS = "-C target-feature=+crt-static";
}
```
Invoke with `nix-shell --run <cmd>` as required by user rules.

### Task Automation (`Justfile`)
```just
# Build release binary (static)
build:
  nix-shell --run "cargo build --release --target x86_64-unknown-linux-musl"

# Run unit + integration tests
test:
  nix-shell --run "cargo test --all"

# Format source tree
fmt:
  nix-shell --run "cargo fmt --all"

# Static analysis
lint:
  nix-shell --run "cargo clippy -- -D warnings"
```

### Directory Layout
```
commit-reflow/
├── Cargo.toml
├── src/
│   ├── lib.rs        # core parsing + reflow engine
│   └── main.rs       # CLI wrapper using clap or structopt
├── shell.nix
└── Justfile
```

### CI Consideration
- GitHub Actions job uses `nix` to enter shell and run `just build && just test`.
- Produces `commit-reflow` artifact uploaded as release.

---
Additions above define how to build, test, and lint the Rust implementation via Nix and Just.

*Author: TBD • Last updated: <!-- YYYY-MM-DD -->* 