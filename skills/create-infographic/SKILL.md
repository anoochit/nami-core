---
name: create-infographic
description: Generate professional infographics from text data or descriptions using the internal 'image_generator' tool. Use this skill whenever the user wants to visualize data, stats, processes, comparisons, or summaries as an image — even if they say "make a chart image", "turn this into a visual", "illustrate these stats", or "create an infographic." Trigger on any request to turn structured or unstructured information into a shareable visual graphic.
---

# Skill: Create Infographic

Transform raw data or a text description into a visually appealing infographic using the internal `image_generator` tool.

## Prerequisites

* Ensure `GOOGLE_API_KEY` is set in your environment as the `image_generator` tool requires it for authentication.

## Usage

Use the `image_generator` tool directly from the terminal or via agent orchestration.

### Example

```json
{
  "prompt": "Create an infographic showing the Top 5 programming languages in 2024: 1. Python (25%), 2. JavaScript (20%), 3. Java (15%), 4. C++ (10%), 5. Go (8%). Use a clean, professional, flat design style.",
  "aspect_ratio": "16:9"
}
```

## Parameters

| Parameter | Description | Required | Default |
| --- | --- | --- | --- |
| `prompt` | The data or content to visualize. Be specific — include labels, values, and any layout preferences. | Yes | — |
| `aspect_ratio` | Aspect ratio for the image. Options: `1:1`, `4:3`, `16:9`, `9:16`. | No | `1:1` |

## Description Guidance

The quality of the output depends heavily on the `prompt`. When the user's input is vague, expand it before calling the tool:

* **Include numbers and labels:** "3 steps: 1. Sign up, 2. Configure, 3. Launch" beats "the onboarding process."
* **Specify layout intent if known:** "side-by-side comparison", "vertical timeline", "pie breakdown."
* **Choose the right ratio:** Use `16:9` for wide/landscape layouts (presentations, banners), `1:1` for social media squares, `9:16` for mobile.

## Workflow

1. **Gather content:** If the user hasn't provided structured data, ask what they want the infographic to show.
2. **Call the `image_generator` tool** with a detailed `prompt` that includes visual style and data structure.
3. **Confirm output:** The tool saves the image to `generated/` and returns the file path. Share this path with the user.
4. **Iterate if needed:** If the result doesn't match expectations, refine the prompt (more specific labels, different style, or ratio) and re-run.

## Troubleshooting

* **API key error:** Ensure `GOOGLE_API_KEY` is set in your shell environment.
* **Poor visual output:** The prompt was likely too vague. Add specific values, categories, or visual style instructions and re-run.
* **Wrong dimensions:** Adjust `aspect_ratio` — `16:9` for wide layouts, `1:1` for square, `9:16` for mobile.