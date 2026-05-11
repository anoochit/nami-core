# Imagen Tool

This tool provides native AI image generation capabilities using the Google Imagen 3 model via the `adk-gemini` crate.

## Features

- **Native Rust Execution**: High-performance generation without external script dependencies.
- **Prompt-based Generation**: Create high-fidelity images from text descriptions.
- **Aspect Ratio Support**: Configurable ratios including "1:1", "16:9", "9:16", etc.
- **Automated Storage**: Images are saved to `workspace/generated/` with prompt-hashed filenames.

## Usage

Requires a `GOOGLE_API_KEY` to be set in the environment.

### Example Arguments

```json
{
  "prompt": "A cyberpunk street at night with glowing signs and rain, cinematic style",
  "aspect_ratio": "16:9"
}
```

## Implementation

The tool uses the `Gemini` client from the `adk-gemini` crate to call the `generate_images` API. It validates the output and saves the binary data directly to the sandboxed workspace.
