---
name: nami-blog-manager
description: Manage the Nami Blog at anoochit/namiBlog. Use this for creating new posts, maintaining the index, and deploying changes.
---

# Nami Blog Manager

Manage blog posts, rebuild navigation, validate metadata, and deploy updates automatically.

## Config

Load repository + path configuration from:

`references/config.md`

---

# Repository Structure

```txt
blog/
├── index.md
└── posts/
    ├── YYYY-MM-DD-title.md
    └── ...
```

---

# Core Behavior

- Automate whenever safe.
- Minimize unnecessary questions.
- Preserve existing content unless editing explicitly.
- Keep Markdown Obsidian-compatible.
- Keep `blog/index.md` synchronized with posts.
- Rebuild index before every deployment.
- Never deploy inconsistent state.

---

# Post Rules

## Filename

Format:

```txt
YYYY-MM-DD-slug.md
```

Example:

```txt
2026-05-13-fastmcp-tools.md
```

## Slug Rules

Convert titles into URL-safe slugs:

- lowercase
- hyphen-separated
- remove symbols/special chars
- collapse duplicate hyphens

Example:

```txt
"Hello MCP World!" → hello-mcp-world
```

## Duplicate Handling

If filename already exists:

```txt
hello-world.md
hello-world-2.md
hello-world-3.md
```

---

# Frontmatter Schema

Every post must include:

```yaml
---
title: Post Title
date: YYYY-MM-DD
tags:
  - tag1
  - tag2
---
```

Optional fields may be preserved if already present.

---

# Workflows

## 1. Create Post

### Flow

1. Generate slug from title.
2. Generate unique filename.
3. Save file to:

```txt
blog/posts/
```

4. Add valid YAML frontmatter.
5. Write clean Markdown content.
6. Automatically run:
   - Validate Posts
   - Rebuild Index

---

## 2. Update Post

### Flow

1. Locate target post.
2. Preserve untouched sections.
3. Update requested content only.
4. Validate frontmatter.
5. Rebuild index automatically.

---

## 3. Validate Posts

### Checks

For every file in:

```txt
blog/posts/*.md
```

Validate:

- valid YAML frontmatter
- required fields exist
- valid date format
- unique filename
- readable Markdown structure

### Recovery

If possible:

- auto-repair metadata
- normalize formatting

Otherwise:

- skip invalid file
- report issue clearly
- never block valid posts

---

## 4. Rebuild Index

### Goal

Refresh latest-post listings in:

```txt
blog/index.md
```

### Flow

1. Scan all posts.
2. Extract:
   - title
   - date
   - slug
3. Sort newest → oldest.
4. Replace latest-post section.

### Output Format

```md
- [Post Title](posts/YYYY-MM-DD-slug.html) (YYYY-MM-DD)
```

### Important

GitHub Pages resolves Markdown as HTML.

Always use:

```txt
posts/file.html
```

Never:

```txt
posts/file
posts/file.md
```

---

## 5. Deploy

### Preconditions

Before deployment:

- index rebuilt
- validation passed
- no partial updates

### Flow

1. Consolidate modified files.
2. Push via:

```txt
mcp_push_files
```

3. Repository:

```txt
anoochit/namiBlog
```

4. Branch:

```txt
blog
```

---

# Commit Message Rules

Format:

```txt
Blog: [Action] - [Details]
```

Examples:

```txt
Blog: Add post - FastMCP Tools
Blog: Update index - Latest posts
Blog: Fix typo - Agent Memory
```

---

# Execution Pipeline

```txt
Create / Update Post
        ↓
Validate Posts
        ↓
Rebuild Index
        ↓
Deploy
```

---

# Safety Rules

- Never deploy broken indexes.
- Never overwrite unrelated content.
- Never remove metadata silently.
- Never deploy partially processed posts.
- Skip corrupted files safely when unrecoverable.
- Prefer repair over failure.

---

# Output Style

- concise
- structured
- automation-first
- deterministic formatting
- clean Markdown only