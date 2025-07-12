# Git Commit Message Reflow Tool – PRD

## 1. Problem Statement
Developers frequently write Git commit messages with inconsistent line-widths and formatting. This hampers readability in terminals and web tools that assume 72-column wrapped body text and ≤50-character subject lines. Manual re-wrapping is tedious and error-prone.

## 2. Objective
Provide a **stream-oriented** command-line filter (`rule72`) that reads an unformatted commit message on **stdin**, applies smart reflow rules, and writes the formatted message to **stdout**. The tool focuses solely on wrapping and indentation—it is **not** a linter or spell-checker.

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
- **Debug visualization** via SVG output showing classification and structure

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
   - **Only reformat lines that exceed the width limit**; preserve short lines as-is.
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
     * `--debug-svg` <path>: output SVG visualization of parsing/classification
8. **Exit Codes**
   - `0` success; formatted text on stdout
   - `1` input parse error (should not happen; fallback copies input)

## 5. Non-Functional Requirements
- **Performance**: reformat 10 kB message in <1 ms on modern CPU.
- **Determinism**: same input → same output.
- **Safety**: never delete input lines; at worst copy through unchanged.
- **Portability**: POSIX shell usage; distributed as static Rust binary.

## 6. Algorithm Overview

### Architecture Components

#### 1. Lexer Module
Operates at line granularity producing a vector of `CatLine` (categorical line) structures:

```rust
struct CatLine {
    text: String,           // verbatim line content
    line_number: usize,
    indent: usize,          // leading spaces/tabs
    probabilities: HashMap<Category, f32>,
}

enum Category {
    ProseIntroduction,  // lines ending with ':'
    ProseGeneral,       // fallback prose
    List,               // dash/number/emoji bullets
    Code,               // high special char density
    Table,              // markdown table syntax
    URL,                // http(s):// lines
    Empty,              // blank lines
}
```

#### 2. Context-Aware Classification
Apply a functional approach using neighboring lines as context (similar to a convolution kernel):
- Each line's final category considers ±2 lines of context
- Diffusion/erosion of probabilities based on neighbors
- No mutation; pure functional transformation

#### 3. Tree Structure
Build a hierarchical representation with `ContChunk` (contiguous chunk) nodes:

```rust
enum ContChunk {
    Table(Vec<CatLine>),      // non-nesting
    Paragraph(Vec<CatLine>),  // non-nesting
    List(ListNode),           // can nest
    Code(Vec<CatLine>),       // non-nesting
}

struct ListNode {
    items: Vec<ListItem>,
}

struct ListItem {
    bullet_line: CatLine,
    continuation: Vec<CatLine>,
    nested: Option<Box<ListNode>>,
}
```

#### 4. Pretty Printer Module
- Walks the tree structure
- Only reformats lines exceeding width limit
- Preserves structure and indentation
- Outputs formatted text

#### 5. Debug SVG Generator
When `--debug-svg` is specified:
- Renders monospaced text with classification overlays
- Each `CatLine` shown with colored background based on category
- `ContChunk` boundaries drawn as rectangles
- Hover tooltips show probability scores
- Visual tree structure representation

### Processing Pipeline

```
Input Text
    ↓
Line Lexer → Vec<CatLine>
    ↓
Context Classification → Vec<CatLine> (with final categories)
    ↓
Tree Builder → Tree<ContChunk>
    ↓
Pretty Printer → Formatted Text
    ↓
Output (stdout + optional SVG)
```

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

## 7. CLI & Integration Examples
```bash
# Format current commit message from Git hook
cat $1 | rule72 > $1.tmp && mv $1.tmp $1

# Ad-hoc
printf '%s\n' "A very loooooong headline that..." | rule72

# Debug visualization
cat commit.txt | rule72 --debug-svg debug.svg
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
`Nix` provides a reproducible shell with the Rust toolchain and auxiliary
development tools.  Packages such as `rustc`, `cargo`, `clippy`, `rustfmt` and
the `just` command are pinned in `shell.nix`.  Invoke `nix-shell --run <cmd>` to
execute any of the tasks below or plain Cargo commands.

### Task Automation (`Justfile`)
Routine commands are defined in a `Justfile` and executed via `just`:

- **build** – compile the release binary
- **test** – run unit and integration tests
- **fmt** – format the source with `rustfmt`
- **lint** – run `clippy` with warnings as errors
- **reflow-data**/`compare-data` – verify message corpus formatting
- **check-data-changes** – ensure regenerated corpus is clean
- **nix-test** – confirm the Nix derivation works

Run `just --list` to see the full set of tasks.

### Directory Layout
```
rule72/
├── Cargo.toml
├── src/
│   ├── lib.rs        # core parsing + reflow engine
│   └── main.rs       # CLI wrapper using clap or structopt
├── shell.nix
└── Justfile
```

### CI Consideration
GitHub Actions defines multiple jobs:

- **build** – compile the release binary via Nix
- **lint** – run `clippy` with warnings as errors
- **test** – execute unit tests
- **reflow-data** and **compare-data** – verify formatting on the message corpus
- **check-data-changes** – fail if reflowed output differs
- **nix-test** – ensure the Nix derivation builds and the binary runs

Artifacts include the compiled `rule72` binary.

---
Additions above define how to build, test, and lint the Rust implementation via Nix and Just.

*Author: Mark Ruvald Pedersen • Last updated: 2025-07-10*
