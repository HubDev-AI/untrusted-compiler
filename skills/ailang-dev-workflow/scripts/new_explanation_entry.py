#!/usr/bin/env python3
"""Append a standard Untrusted<T> explanation section to a markdown file."""

import argparse
from datetime import datetime
from pathlib import Path

TEMPLATE = """
## {title}

_Date: {date}_

### What it is

### Why it exists

### How it works internally

### Inputs, outputs, and constraints

### Failure modes and diagnostics

### Example usage

### Tradeoffs and next steps
"""


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--title", required=True, help="Feature/component title")
    parser.add_argument("--file", required=True, help="Markdown file to append to")
    args = parser.parse_args()

    path = Path(args.file).expanduser().resolve()
    path.parent.mkdir(parents=True, exist_ok=True)

    stamp = datetime.now().strftime("%Y-%m-%d")
    entry = TEMPLATE.format(title=args.title.strip(), date=stamp)

    if path.exists():
        current = path.read_text(encoding="utf-8")
        if not current.endswith("\n"):
            current += "\n"
        path.write_text(current + "\n" + entry.strip() + "\n", encoding="utf-8")
    else:
        path.write_text(entry.strip() + "\n", encoding="utf-8")

    print(f"Appended explanation template to {path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
