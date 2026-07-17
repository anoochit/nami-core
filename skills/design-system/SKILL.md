---
name: design-system
description: Provides design system context and brand-specific design specifications (colors, typography, components, and layout systems) by generating a root DESIGN.md file based on the requested brand (such as Airbnb, BMW M, Claude, Dell 1996, HP, or Nintendo 2001) whenever a user or agent requests a design system or brand theme.
---

# Design System Context Provider

This skill provides a structured design system context (`DESIGN.md`) in the workspace root when requested by a user or an agent.

## Available Brand Specifications
The following brand design systems are located in this skill's directory:
- **Airbnb**: `DESIGN-airbnb.md` (Warm, generous consumer marketplace, white canvas with vibrant Airbnb Rausch accent, friendly rounded styling, trusts organic photography and generous whitespace over heavy typographic weight).
- **BMW M**: `DESIGN-bmw-m.md` (High-end motorsport engineering, pure black canvas, M tricolor stripe accents, European feel).
- **Claude**: `DESIGN-claude.md` (Sleek, modern minimalist, editorial typography, soft ink colors).
- **Dell 1996**: `DESIGN-dell-1996.md` (Retro-computing, gray surfaces, bold blue and purple accents, high-contrast nostalgic grids).
- **HP**: `DESIGN-hp.md` (Corporate premium, clean corporate blue and silver colorways, structured corporate typography).
- **Nintendo 2001**: `DESIGN-nintendo-2001.md` (Playful retro console vibe, energetic gaming aesthetic, accent colors and interactive styles inspired by early 2000s hardware).

## When to Use This Skill
Activate this skill when:
- The user asks to "use the Airbnb theme", "apply Claude style", "create with Dell 1996 design system", or mentions any of the other brands.
- An agent or user requests "initialize the design system", "give me design system context", or "create DESIGN.md".
- You are starting a new web or app project and want to establish the brand guidelines immediately.

## Execution Instructions
When a brand/theme is requested:
1. **Identify the Brand**: Map the request to one of the available brand files in `skills/design-system/`.
   - If no specific brand is mentioned, ask the user to clarify or default to a reasonable brand (e.g., `Airbnb`, `BMW M`, or `Claude`).
2. **Read the Specification**: Use the `view_file` tool to read the complete content of the selected brand file, e.g., `skills/design-system/DESIGN-airbnb.md`.
3. **Write the Root DESIGN.md**: Write the exact contents of that brand specification file to `DESIGN.md` in the project root directory.
4. **Confirm to User**: Inform the user and other agents that the design system has been successfully initialized in the workspace, outlining the core colors and typography for their reference.
