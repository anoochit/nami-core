---
name: nami-blog-manager
description: Manage the Nami Blog at anoochit/namiBlog. Use this for creating new posts, maintaining the index, and deploying changes.
---

# Nami Blog Manager

Use this skill to automate the maintenance and deployment of Noel's blog.

## Configuration
Refer to [config.md](references/config.md) for repository and path details.

## Workflows

### 1. Creating a New Post
- Generate a post with filename format: `YYYY-MM-DD-title.md`.
- Place it in the `posts/` directory.
- Include YAML frontmatter: `title`, `date`, `tags`.
- Content should be in Obsidian-compatible Markdown.
- **Action**: After creating a post, always run the "Rebuilding Index" workflow.

### 2. Rebuilding Index (Maintenance)
- Scan the `posts/` directory for all `.md` files.
- Read each file to extract `title` and `date` from the frontmatter.
- Sort posts by date in descending order (newest first).
- Update `index.md` by replacing the "Latest Posts" list.
- Format: `- [Title](posts/YYYY-MM-DD-title) (YYYY-MM-DD)`.

### 3. Deployment (Push to GitHub)
- Consolidate all changes (new posts and updated index).
- Use `mcp_push_files` to push to `anoochit/namiBlog` on branch `blog`.
- Commit message: "Blog: [Action] - [Title/Details]" (e.g., "Blog: Add new post - Hello World").
