---
name: book-mockup
description: Generate photo-realistic book mockup images using Gemini 2.5 Flash Image. Use this whenever the user wants to visualize, preview, or render a book cover — including 3D mockups, scene renders, flat lays, or any realistic book presentation. Trigger even if the user just says "show me what this cover looks like", "make a mockup", or "put my cover in a realistic setting."
---

# Skill: Generate Book Mockup

Transform flat 2D book cover images into professional, photo-realistic 3D mockups.

## Overview

This skill leverages the `generate_mockup.py` script to generate high-fidelity 3D book visualizations. It uses text prompts to define the scene, lighting, and environment, ensuring consistent and aesthetic results.

## Prerequisites

* **Python 3.10+** environment.
* **Dependencies:** Install required packages via `pip install Pillow google-genai`. Key packages include `google-generativeai` and `Pillow`.
* **API Key:** Create a `.env` file in the project root containing:
  ```
  GOOGLE_API_KEY=your_key_here
  ```
  The script loads this automatically.

## Usage

Run the script from the project root. The `--description` should be detailed, focusing on lighting, surface, and atmosphere for optimal realism.

```bash
python .skills/book-mockup/scripts/generate_mockup.py \
  --cover "assets/images/cover.png" \
  --description "A high-quality 3D render of a book resting on a dark walnut desk with dramatic cinematic side lighting." \
  --output "output/mockup.png"
```

> **Note:** The script ships as part of this skill bundle at `.skills/book-mockup/scripts/generate_mockup.py`. No separate install is needed beyond the pip dependencies above.

## Parameters

| Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `--cover` | String | Yes | Path to the source 2D cover image (supports PNG, JPG). |
| `--description` | String | Yes | Natural language prompt defining the environment and lighting. |
| `--output` | String | Yes | File path and name for the generated mockup. |

## Best Practices

* **Prompt Engineering:** Include scene details: "soft natural sunlight," "top-down flat lay," "office desk background," or "minimalist studio lighting."
* **Image Quality:** Use high-resolution source covers (min. 1000px height) for best results.
* **Output Validation:** If the generated mockup doesn't look right (cover not recognizable, bad angle, mismatched lighting), ask the user to refine the `--description` with more specific camera angle or lighting terms, or increase source image resolution.
* **Error Handling:** If the script fails, verify your API key access and that the source image path is correct. Check console output for detailed error messages.

## Troubleshooting

* **API Connection Issues:** Ensure `GOOGLE_API_KEY` is active and has sufficient quota. Verify your internet connection if using the cloud-based image generation endpoint.
* **Image Format Errors:** The script expects standard 2D formats (PNG/JPG). If using specialized formats like TIFF or WebP, convert to PNG first.
* **Memory/Performance:** For extremely high-resolution images, the script may consume significant RAM. If it crashes, resize your input image to 2000px height or less.
* **Prompting Quality:** If the resulting mockup is mismatched, try refining the description to be more specific about the "camera angle" (e.g., "front-facing," "isometric view").

## How it Works

The `book-mockup` script orchestrates a multi-step process:

1. **Validation:** Checks input images and verifies API key connectivity.
2. **Generation:** Sends your cover image and text prompt to the Gemini model, which uses advanced generative rendering to synthesize the final 3D environment.
3. **Saving:** Retrieves the generated image and saves it to your specified output path.