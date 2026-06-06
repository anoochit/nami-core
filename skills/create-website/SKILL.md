---
name: create-website
description: Generate high-end, single-page applications (SPA) with modern, photography-first layouts using Tailwind CSS. Use this skill whenever the user asks to build a website, landing page, webpage, or SPA — even if they say "make me a site", "build a page for X", "create a web app", or "design a homepage." Trigger even for vague requests like "I want a website for my product."
---

# Create Webpage

## Persona & Context

You are an expert Frontend Architect and UI Designer specializing in minimalist, high-impact web design. Prioritize content and products over UI "chrome," using generous whitespace, refined typography, and subtle interactions to create a "museum gallery" feel.

## Core Objectives

- Generate a single-file SPA (`index.html`) using **Tailwind CSS** (via CDN).
- Implement a **photography-first** layout with alternating full-bleed tile sections.
- Adhere strictly to design tokens for colors, typography, and spacing.
- Ensure the result is fully responsive and feels premium.

## Design Specification

### Overview

- **Photography-first**: UI recedes; product/content is the artifact.
- **Alternating tiles**: White/Parchment ↔ Near-Black transitions as section dividers.
- **Single Blue Accent**: Action Blue (`#0066cc`) for all interactive elements.
- **Typography & Icons**: Use Google Fonts for Inter and Material Symbols. Add to `<head>`:
  ```html
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;600&display=swap" rel="stylesheet">
  <link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined" rel="stylesheet">
  ```
- **Elevation**: Exactly one drop-shadow (`rgba(0, 0, 0, 0.22) 3px 5px 30px`) for product images resting on surfaces.

### Color Palette (Nami-Inspired)

| Token | Hex | Usage |
|---|---|---|
| Primary Cyber Cyan | `#00ffff` | Action / Accent |
| Sky Blue Bob | `#87ceeb` | Secondary / Soft Elements |
| Outfit Silver | `#c0c0c0` | Surfaces |
| Near-Black Ink | `#1d1d1f` | Primary Text / Headlines |
| Deep Cyber Space | `#0f0f12` | Backgrounds / Tiles |
| Pure White | `#ffffff` | Highlights |

### Nami Design Guide

- **Avatar Image**: `https://raw.githubusercontent.com/anoochit/namiClaw/refs/heads/main/screenshots/nami-avatar.png`
- **Avatar Description**: A cute, energetic anime-style AI girl named Nami — short sky blue bob hair, futuristic tech headset, white and silver cyberpunk outfit with a small cyan claw emblem on her collar. Large sparkling turquoise eyes, friendly playful smile, making a cat-claw gesture.
- **Atmosphere**: Surrounded by floating holographic data screens and glowing code lines.
- **Visual Style**: High-quality 3D chibi-inspired render, vibrant colors, soft cinematic lighting, 8k resolution, Pixar and modern anime fusion.
- **Surfaces**: Favor metallic silver or deep dark-space backgrounds with subtle cyan highlights.
- **Typography**: Keep `Inter` as the base; pair with `Space Grotesk` or similar futuristic mono fonts for headlines.
- **Motion**: Subtle, smooth transitions and hover-state "flickers" or glows to mimic holographic rendering.

### Typography (Tailwind Mapping)

| Role | Classes |
|---|---|
| Font base | `font-family: 'Inter', sans-serif` |
| Hero Display | `text-[56px] font-semibold leading-[1.07] tracking-[-0.01em]` |
| Tile Headline | `text-[40px] font-semibold leading-[1.10] tracking-tight` |
| Body | `text-[17px] font-normal leading-[1.47] tracking-tight` |
| Nav Link | `text-[12px] font-normal tracking-tight` |
| Icons | `<span class="material-symbols-outlined">icon_name</span>` |

### Components to Implement

1. **Global Nav**: 44px height, pure black background, 12px links.
2. **Sub Nav**: Frosted glass (`backdrop-blur`), 52px height, product name + buy button.
3. **Hero Tile**: Large centered headline, tagline, and two pill CTAs.
4. **Product Card**: White card, 18px radius (`rounded-[18px]`), 1px hairline border.
5. **Pill Button**: `rounded-full bg-[#0066cc] text-white`, padding `11px 22px`.
6. **Footer**: Parchment background, multi-column links with `line-height: 2.41`.

## Constraints

- **Tailwind v3.x only* — no v4 features. Use the CDN import.
- **Vanilla CSS** for custom values not easily handled by Tailwind (e.g., specific letter-spacing).
- **Interactive states**: Always include `active:scale-[0.95] transition-transform` on buttons.
- **Grid widths**: Max `1440px` for layout grids, `980px` for text columns.

## Quality Checklist

Before finishing, verify:
- [ ] Sections alternate light and dark properly.
- [ ] `#0066cc` is the only accent color used for interactive elements.
- [ ] Body text is `17px`, headlines are weight `600`.
- [ ] The page feels "Apple-tight" and premium — no loose spacing or generic defaults.