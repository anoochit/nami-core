---
name: book-mockup
description: Generate photo-realistic book mockup images using the internal `image_generator` tool. Use this whenever you want to visualize, preview, or render a book cover — including 3D mockups, scene renders, flat lays, or any realistic book presentation.
---

# Skill: Generate Book Mockup

Transform flat 2D book cover images into professional, photo-realistic 3D mockups directly through Nami.

## Overview

This skill integrates seamlessly with Nami's internal `image_generator` tool to create high-fidelity 3D book visualizations. By providing a 2D cover image and a descriptive prompt, Nami synthesizes a photo-realistic mockup scene with sophisticated lighting and environmental effects.

## Usage

This tool is natively integrated and does not require executing any external commands or scripts. Simply ask Nami to generate a mockup, and she will handle the process internally using `image_generator`.

**Example Request:**
> "Generate a 3D mockup of this book cover resting on a dark walnut desk with dramatic cinematic side lighting."

## Agent Instructions (How to use this skill)

When the user asks for a book mockup, follow these steps:
1. **Analyze Intent:** Confirm the user wants a visualization of a book cover. If a cover image is provided, ensure you have access to it. If not, politely ask the user to share the cover image file.
2. **Clarify Scene:** Ask for or infer the scene details (lighting, surface, atmosphere) if not explicitly provided.
3. **Invoke Tool:** Call the internal `image_generator` tool. Use the provided cover image (or reference it) and craft a rich prompt based on the user's description. Ensure your prompt includes camera angles, lighting conditions, and environmental context.
4. **Present Result:** Once generated, present the final photo-realistic mockup to the user, and confirm if they would like any refinements (e.g., lighting changes, angle shifts).

## Best Practices

* **Prompt Engineering:** Include scene details like "soft natural sunlight," "top-down flat lay," "office desk background," or "minimalist studio lighting" to get the best visual results.
* **Image Quality:** Use high-resolution source covers (min. 1000px height) for optimal detail and realism.
* **Refinement:** If the first result isn't perfect, just ask for a tweak (e.g., "Make it a bit brighter," or "Change the angle to isometric"), and Nami will generate a new, refined version using `image_generator`.

## How it Works

1. **Analysis:** Analyzes your source 2D cover image.
2. **Synthesis:** Invokes the internal `image_generator` tool to render the cover into a 3D environment, applying advanced lighting and texture mapping based on your prompt.
3. **Delivery:** The final photo-realistic mockup is rendered and presented to you, ready for use.