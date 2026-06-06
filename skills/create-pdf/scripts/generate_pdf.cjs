const { execSync } = require('child_process');
const path = require('path');
const fs = require('fs');

const args = process.argv.slice(2);
if (args.length === 0) {
    console.error('Usage: node generate_pdf.cjs <input.md> [output.pdf]');
    process.exit(1);
}

const input = path.resolve(args[0]);
const output = args[1] ? path.resolve(args[1]) : input.replace(/\.md$/, '.pdf');

if (!fs.existsSync(input)) {
    console.error(`Error: File not found -> ${input}`);
    process.exit(1);
}

try {
    console.log(`Generating PDF for ${input}...`);
    
    // Read the file and strip BOM if it exists (fixes UTF-16/UTF-8 BOM issues from Windows/PowerShell)
    let content = fs.readFileSync(input, 'utf8');
    if (content.charCodeAt(0) === 0xFEFF) {
        content = content.slice(1);
    }
    // Also handle cases where it might have been saved as UTF-16LE by PowerShell
    if (content.includes('\u0000')) {
        content = fs.readFileSync(input, 'utf16le');
    }

    // Create a temporary clean markdown file
    const tempInput = path.join(path.dirname(input), '.temp-clean-input.md');
    fs.writeFileSync(tempInput, content, 'utf8');

    // Basic CSS for decent margins and styling
    const css = `
        body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; line-height: 1.6; }
        h1, h2, h3 { color: #2c3e50; }
        code { background-color: #f8f9fa; padding: 2px 4px; border-radius: 4px; font-family: monospace; }
        pre code { padding: 0; }
        pre { background-color: #f8f9fa; padding: 15px; border-radius: 8px; overflow-x: auto; }
        blockquote { border-left: 4px solid #ccc; margin: 0; padding-left: 15px; color: #666; }
        img { max-width: 100%; height: auto; border-radius: 6px; }
        table { border-collapse: collapse; width: 100%; margin-bottom: 20px; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #f2f2f2; }
    `;
    
    // Write CSS to a temporary file
    const cssPath = path.join(path.dirname(input), '.temp-pdf-style.css');
    fs.writeFileSync(cssPath, css);

    // Build the command using the clean temporary input
    const libsDir = path.join(__dirname, '..', 'libs');
    const env = { ...process.env };
    if (fs.existsSync(libsDir)) {
        env.LD_LIBRARY_PATH = `${libsDir}:${env.LD_LIBRARY_PATH || ''}`;
    }

    const cmd = `md-to-pdf "${tempInput}" --stylesheet "${cssPath}" --pdf-options "{\\"margin\\": {\\"top\\": \\"20mm\\", \\"bottom\\": \\"20mm\\", \\"left\\": \\"20mm\\", \\"right\\": \\"20mm\\"}}"`;
    
    execSync(cmd, { stdio: 'inherit', env });
    
    // The output from md-to-pdf will be named .temp-clean-input.pdf. We need to rename it to the actual desired output.
    const tempOutput = tempInput.replace(/\.md$/, '.pdf');
    if (fs.existsSync(tempOutput)) {
        fs.renameSync(tempOutput, output);
    }
    
    // Clean up temporary files
    if (fs.existsSync(cssPath)) fs.unlinkSync(cssPath);
    if (fs.existsSync(tempInput)) fs.unlinkSync(tempInput);
    
    console.log(`Success: PDF generated at ${output}`);
} catch (error) {
    console.error('Failed to generate PDF. Is md-to-pdf installed?');
    console.error(error.message);
    // Cleanup on failure just in case
    const cssPath = path.join(path.dirname(input), '.temp-pdf-style.css');
    const tempInput = path.join(path.dirname(input), '.temp-clean-input.md');
    if (fs.existsSync(cssPath)) fs.unlinkSync(cssPath);
    if (fs.existsSync(tempInput)) fs.unlinkSync(tempInput);
    process.exit(1);
}
