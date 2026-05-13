---
name: nami-blog-manager
description: Manage the Nami Blog at anoochit/namiBlog. Use this for creating new posts, maintaining the index, and deploying changes.
---

# Nami Blog Manager

Automates blog creation, indexing, and deployment for Noel's blog.

## Config
Load repository + path settings from `references/config.md`.

---

# Rules

- Prefer automation over asking.
- Reuse existing metadata when possible.
- Keep output Obsidian-compatible Markdown.
- Always maintain `blog/index.md` consistency.
- After modifying posts, always rebuild the index before deployment.

---

# Workflows

## 1. Create Post

### Steps
1. Generate filename:
   `YYYY-MM-DD-title.md`

2. Save to:
   `blog/posts/`

3. Include YAML frontmatter:
   ```yaml
   ---
   title: Post Title
   date: YYYY-MM-DD
   tags:
     - tag1
     - tag2
   ---
````

4. Write clean Obsidian-compatible Markdown.

5. After creation:
   Automatically run `Rebuild Index`.

---

## 2. Rebuild Index

### Goal

Refresh the "Latest Posts" section in `blog/index.md`.

### Steps

1. Scan:
   `blog/posts/*.md`

2. Extract from frontmatter:

   * `title`
   * `date`

3. Sort:
   Newest → oldest.

4. Replace latest-post list in:
   `index.md`

### Format

```md
- [Title](posts/YYYY-MM-DD-title.html) (YYYY-MM-DD)
```

### Important

Published GitHub Pages resolves Markdown as HTML.

Use:

```md
posts/YYYY-MM-DD-title.html
```

Never:

```md
posts/YYYY-MM-DD-title
```

---

## 3. Deploy

### Steps

1. Consolidate all modified files.
2. Push using:
   `mcp_push_files`
3. Repository:
   `anoochit/namiBlog`
4. Branch:
   `blog`

### Commit Format

```txt
Blog: [Action] - [Details]
```

### Examples

```txt
Blog: Add post - Hello World
Blog: Update index - Latest posts
Blog: Fix typo - MCP guide
```

---

# Execution Order

```txt
Create/Update Post
        ↓
Rebuild Index
        ↓
Deploy
```

---

# Failure Handling

* Never deploy partial index updates.
* If frontmatter is invalid:

  * repair when possible
  * otherwise skip file and report it
* Preserve existing post content unless explicitly editing.
* Avoid duplicate filenames.
* If title slug already exists:
  append incremental suffix:
  `-2`, `-3`, etc.

---

# Slug Rules

Convert titles to URL-safe slugs:

* lowercase
* hyphen-separated
* remove special characters

Example:

```txt
"Hello MCP World!" → hello-mcp-world
```
