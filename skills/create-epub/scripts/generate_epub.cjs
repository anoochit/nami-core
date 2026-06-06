const { execSync } = require('child_process');
const path = require('path');
const fs = require('fs');

const args = process.argv.slice(2);
if (args.length === 0) {
    console.error('Usage: node generate_epub.cjs <input.md> [output.epub]');
    process.exit(1);
}

const input = path.resolve(args[0]);

if (!fs.existsSync(input)) {
    console.error(`Error: File not found -> ${input}`);
    process.exit(1);
}

try {
    console.log(`Generating EPUB for ${input}...`);
    
    // Read the file as raw bytes to accurately strip BOMs
    let raw = fs.readFileSync(input);
    let content = '';

    if (raw.length >= 2 && raw[0] === 0xFF && raw[1] === 0xFE) {
        // UTF-16LE BOM
        content = raw.toString('utf16le').replace(/^\uFEFF/, '');
    } else if (raw.length >= 3 && raw[0] === 0xEF && raw[1] === 0xBB && raw[2] === 0xBF) {
        // UTF-8 BOM
        content = raw.slice(3).toString('utf8');
    } else {
        // Assume UTF-8 without BOM
        content = raw.toString('utf8');
        
        // Fallback check if it somehow still has a BOM char at the string level
        if (content.charCodeAt(0) === 0xFEFF) {
            content = content.slice(1);
        }
    }

    // Create a temporary clean markdown file (strictly UTF-8 without BOM)
    const tempInput = path.join(path.dirname(input), '.temp-clean-input-epub.md');
    fs.writeFileSync(tempInput, content, 'utf8');

    // Build the command
    let cmd = `md-to-epub "${tempInput}"`;
    
    execSync(cmd, { stdio: 'inherit' });
    
    // md-to-epub outputs to an `output/` directory in the current working directory
    const tempFileName = path.basename(tempInput).replace(/\.md$/, '.epub');
    const generatedEpubPath = path.join(process.cwd(), 'output', tempFileName);
    const finalOutput = args[1] ? path.resolve(args[1]) : input.replace(/\.md$/, '.epub');

    if (fs.existsSync(generatedEpubPath)) {
        // Ensure destination directory exists
        const outDir = path.dirname(finalOutput);
        if (!fs.existsSync(outDir)) {
            fs.mkdirSync(outDir, { recursive: true });
        }
        
        fs.renameSync(generatedEpubPath, finalOutput);
        
        // Try to clean up the empty output dir if it was just created
        try { fs.rmdirSync(path.join(process.cwd(), 'output')); } catch(e) {}
    } else {
        console.warn(`Warning: Expected output file not found at ${generatedEpubPath}`);
    }
    
    // Clean up temporary files
    if (fs.existsSync(tempInput)) fs.unlinkSync(tempInput);
    
    console.log(`Success: EPUB generated at ${finalOutput}`);
} catch (error) {
    console.error('Failed to generate EPUB. Is md-to-epub installed? Run "npm install -g md-to-epub"');
    console.error(error.message);
    
    const tempInput = path.join(path.dirname(input), '.temp-clean-input-epub.md');
    if (fs.existsSync(tempInput)) fs.unlinkSync(tempInput);
    process.exit(1);
}
