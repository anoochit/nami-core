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
    
    let content = fs.readFileSync(input, 'utf8');
    if (content.charCodeAt(0) === 0xFEFF) {
        content = content.slice(1);
    }
    if (content.includes('\u0000')) {
        content = fs.readFileSync(input, 'utf16le');
    }

    const tempInput = path.join(path.dirname(input), '.temp-clean-input.md');
    fs.writeFileSync(tempInput, content, 'utf8');

    // Updated CSS to include Prompt font for Thai support
    const css = `
        @import url('https://fonts.googleapis.com/css2?family=Prompt:wght@300;400;700&display=swap');
        body { font-family: 'Prompt', sans-serif; line-height: 1.6; }
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
    
    const cssPath = path.join(path.dirname(input), '.temp-pdf-style.css');
    fs.writeFileSync(cssPath, css);

    const libsDir = path.join(__dirname, '..', 'libs');
    const env = { ...process.env };
    if (fs.existsSync(libsDir)) {
        env.LD_LIBRARY_PATH = `${libsDir}:${env.LD_LIBRARY_PATH || ''}`;
    }

    const cmd = `md-to-pdf "${tempInput}" --stylesheet "${cssPath}" --pdf-options "{\\"margin\\": {\\"top\\": \\"20mm\\", \\"bottom\\": \\"20mm\\", \\"left\\": \\"20mm\\", \\"right\\": \\"20mm\\"}}"`;
    
    execSync(cmd, { stdio: 'inherit', env });
    
    const tempOutput = tempInput.replace(/\.md$/, '.pdf');
    if (fs.existsSync(tempOutput)) {
        fs.renameSync(tempOutput, output);
    }
    
    if (fs.existsSync(cssPath)) fs.unlinkSync(cssPath);
    if (fs.existsSync(tempInput)) fs.unlinkSync(tempInput);
    
    console.log(`Success: PDF generated at ${output}`);
} catch (error) {
    console.error('Failed to generate PDF.');
    console.error(error.message);
    const cssPath = path.join(path.dirname(input), '.temp-pdf-style.css');
    const tempInput = path.join(path.dirname(input), '.temp-clean-input.md');
    if (fs.existsSync(cssPath)) fs.unlinkSync(cssPath);
    if (fs.existsSync(tempInput)) fs.unlinkSync(tempInput);
    process.exit(1);
}
