# Imagen Tool

This tool provides AI image generation capabilities using the Google Imagen model.

## Features

- **Prompt-based Generation**: Create high-quality images from text descriptions.
- **Aspect Ratio Control**: Support for various aspect ratios (default is "1:1").
- **Workspace Integration**: Generated images are automatically saved to `workspace/generated/`.

## Usage

The tool requires a `GOOGLE_API_KEY` to be set in the environment.

### Example Arguments

```json
{
  "prompt": "A futuristic city with neon lights and flying cars",
  "aspect_ratio": "16:9"
}
```

## Implementation Details

The tool acts as a wrapper around a Python script located at `workspace/.skills/imagen/scripts/generate_image.py`. It communicates with the script via JSON over stdin and processes the resulting image path.
