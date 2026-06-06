---
name: create-pdf
description: Convert Markdown files into professionally formatted PDF documents. Use this skill whenever the user asks to "convert to pdf", "create pdf", "print markdown", "generate a pdf", or wants to export any Markdown content as a shareable document — even if they just say "save this as a file I can print" or "make a PDF version of this."
---

# create-pdf

This skill provides a standardized workflow for converting Markdown documents into beautifully formatted PDF files using a bundled Node.js script powered by `md-to-pdf`.

## Prerequisites

The `generate_pdf.cjs` script requires `md-to-pdf` to be installed globally. Check and install in one step:

```bash
npm list -g md-to-pdf || npm install -g md-to-pdf
```

## Quick Start

Run the bundled generation script with the path to the target Markdown file:

```bash
node workspace/.skills/create-pdf/scripts/generate_pdf.cjs path/to/your_file.md
```

The script automatically:
1. Validates the file path.
2. Applies a clean, professional CSS stylesheet (typography, margins, code block formatting).
3. Sets appropriate print margins.
4. Saves the PDF to the same directory as the input file, with a `.pdf` extension.

## Workflow

1. **Check dependencies:** Run the prerequisite install command above before anything else.
2. **Verify the file exists:** Confirm the target `.md` file is accessible before running the script.
3. **Execute the script:** Run `generate_pdf.cjs` with the correct path.
4. **Confirm output:** The script prints `Success: PDF generated at path/to/your_file.pdf` on completion. If this line doesn't appear, treat it as a failure and surface the error to the user.
5. **Notify the user:** Share the path to the generated PDF so they can open or print it.

## Troubleshooting

* **`md-to-pdf` not found:** Run `npm install -g md-to-pdf` and retry.
* **Script exits without printing success:** Check stderr for the underlying error — most commonly a bad file path or unsupported Markdown syntax.
* **Styling looks off:** The bundled stylesheet handles standard Markdown. If the source file uses non-standard extensions (e.g. raw HTML, custom directives), output may vary.
* **Output location unclear:** The PDF always lands in the same directory as the input `.md` file — there is no output path argument. Move the file afterward if a different destination is needed.