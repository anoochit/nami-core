---
name: imagen
description: Generate images from text descriptions using the Imagen model. Use this skill when the user explicitly requests image generation from a text prompt, or implicitly by describing an image to be created.
---

# Skill: Imagen

This skill allows for the generation of high-quality images from textual descriptions using the Imagen text-to-image diffusion model.

## Prerequisites

*   **Python 3.x** installed.
*   **Dependencies:** (Needs to be filled in based on Imagen skill's actual dependencies, e.g., `pip install google-genai` if applicable).
*   **API Key:** Set `GOOGLE_API_KEY` in your environment, or place it in a `.env` file at the project root as `GOOGLE_API_KEY=your_key_here`. (This is a common setup for Google AI models)

## Usage

To use this skill, provide a detailed text description (prompt) of the image you wish to generate.

```bash
python .skills/imagen/scripts/generate_image.py \
  --prompt "A futuristic city at sunset, with flying cars and towering skyscrapers, highly detailed, photorealistic" \
  --output generated_image.png \
  --size "1024x1024"
```
*(Note: `generate_image.py` and `--size` are assumed based on typical image generation tools. These need to be confirmed.)*

## Parameters

| Parameter | Description | Required | Default |
|---|---|---|---|
| `--prompt` | The detailed text description for the image to be generated. Be specific about style, content, and composition. | Yes | — |
| `--output` | Filename or path where the generated image will be saved (e.g., PNG). | Yes | — |
| `--size` | Desired resolution of the image (e.g., `512x512`, `1024x1024`). | No | `1024x1024` *(Assumed)* |
| `--style` | Optional artistic style (e.g., `photorealistic`, `oil painting`, `cartoon`). | No | `None` |
| `--aspect-ratio` | Optional aspect ratio (e.g., `1:1`, `16:9`, `4:3`). | No | `1:1` *(Assumed)* |

## Prompt Guidance

The quality of the generated image heavily depends on the `--prompt`. Consider the following:

*   **Be specific:** "A red car" vs. "A vintage cherry-red sports car, parked on a cobblestone street in Paris, golden hour lighting."
*   **Include details about style:** "digital painting", "photorealistic", "watercolor", "anime style".
*   **Specify composition:** "close-up", "wide shot", "portrait", "landscape".
*   **Use descriptive adjectives:** "ethereal", "vibrant", "moody", "serene".

## Workflow

1.  **Gather prompt:** Ask the user for a detailed description of the image they want to create.
2.  **Run the script:** Execute the `generate_image.py` script with the user's prompt and desired parameters.
3.  **Confirm output:** The script saves the image to the specified `--output` path. Share the file path with the user.
4.  **Iterate if needed:** If the result isn't satisfactory, refine the prompt (add more details, change style) and re-run the script.

## Troubleshooting

*   **Dependencies not found:** Install required libraries (e.g., `pip install google-genai`).
*   **API key error:** Ensure `GOOGLE_API_KEY` is correctly set.
*   **Poor image quality:** Improve prompt specificity and detail. Experiment with different styles or parameters.
*   **Incorrect dimensions/aspect ratio:** Adjust `--size` or `--aspect-ratio` parameters.
