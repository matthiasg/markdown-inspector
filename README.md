# markdown-inspector (mdi)

CLI tool and library to inspect markdown document structure.

## Installation

```bash
cargo install --path .
```

### Auto-install on version tags (optional)

To automatically build and install to `~/.local/bin` when pushing version tags:

```bash
export MDI_LOCAL_INSTALL=1
```

This is opt-in and skipped in CI environments.

## CLI Usage

### Show document outline

```bash
mdi outline README.md
```

Output:
```
   1:Title
   5:  Section 1
  12:    Subsection A
  20:  Section 2
```

Limit depth:
```bash
mdi outline README.md --depth 2
```

### Read a section

By line number:
```bash
mdi read README.md 5
```

By heading text (partial match, case-insensitive):
```bash
mdi read README.md "section 1"
```

Show only subsection outline:
```bash
mdi read README.md "section 1" --outline
```

Show section intro text + subsections as outline:
```bash
mdi read README.md "section 1" --summary
```

Show full section content with subsections collapsed to outline:
```bash
mdi read README.md "section 1" --shallow
```

### Stdin support

```bash
cat doc.md | mdi outline -
```

## Library Usage

```rust
use markdown_inspector::{parse_headings, find_section, get_section_range, extract_section};

let content = std::fs::read_to_string("doc.md")?;
let headings = parse_headings(&content);

// Find a section
if let Some(heading) = find_section(&headings, "Installation") {
    let (start, end) = get_section_range(&headings, heading);
    let section_text = extract_section(&content, start, end);
    println!("{}", section_text);
}
```

## License

MIT
