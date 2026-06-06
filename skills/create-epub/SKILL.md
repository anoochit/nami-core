---
name: create-epub
description: Convert Markdown files into EPUB e-book documents. Use this skill whenever the user asks to "convert to epub", "create epub", "make an ebook", "generate an epub", or wants to package any Markdown content into an e-reader-friendly format — even if they don't say "epub" explicitly and just want a "book file" or "downloadable book."
---

# create-epub

This skill provides a standardized workflow for converting Markdown documents into EPUB e-book files. It uses a bundled Node.js script that leverages the `md-to-epub` package, ideal for compiling documentation or books into e-reader-friendly formats.

## Prerequisites

The `generate_epub.cjs` script requires `md-to-epub` to be installed globally. Check if it's available first, and install if missing:

```bash
npm list -g md-to-epub || npm install -g md-to-epub
```

## Quick Start

Run the bundled generation script, passing the path to the Markdown file and an optional output path:

```bash
node .skills/create-epub/scripts/generate_epub.cjs path/to/your_file.md [optional/output/path.epub]
```

The script automatically:
1. Validates the file path and sanitizes encoding issues (e.g. Windows UTF-16 BOMs).
2. Runs the Markdown-to-EPUB converter.
3. Moves the output to the requested location (defaults to the same directory as the input file, with an `.epub` extension).

## Workflow

1. **Check dependencies:** Run the prerequisite install command above before anything else.
2. **Verify the file exists:** Confirm the target `.md` file is accessible before running the script.
3. **Execute the script:** Run `generate_epub.cjs` with the correct paths.
4. **Confirm output:** The script prints `Success: EPUB generated at path/to/your_file.epub` on completion. If this line doesn't appear, treat it as a failure and surface the error to the user.
5. **Notify the user:** Share the absolute or relative path to the generated EPUB so they can open it in an e-reader (Apple Books, Calibre, Kindle, etc.).

## Troubleshooting

* **`md-to-epub` not found:** Run `npm install -g md-to-epub` and retry.
* **Encoding errors:** The script handles UTF-16 BOMs automatically. If you still see garbled output, ensure the source file is saved as UTF-8.
* **Script exits without printing success:** Check stderr for the underlying error — most commonly a bad file path or a malformed Markdown structure.
* **Output file not where expected:** Without a second argument, the EPUB lands next to the input `.md` file. Pass an explicit output path to control the destination.