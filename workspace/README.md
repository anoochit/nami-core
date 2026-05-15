# Nami Workspace Sandbox

The `workspace/` directory serves as the primary sandbox and configuration hub for your Nami agent. This is where the agent maintains its personal context, persistent memories, and custom extensions.

## Directory Structure

- `.skills/`: Contains custom Agent Skills that extend Nami's capabilities.
- `wiki/`: A repository for project-specific knowledge, documentation, and notes.
- `project/`, `blog/`, `generated/`, `images/`, `plans/`: Dedicated directories for project outputs, assets, and temporary files.

## Core Configuration & Context Files

These files are essential for defining the agent's personality, user preferences, and state management protocol:

- `AGENT.md`: Defines the agent's persona, core mandates, and operational guidelines.
- `USER.md`: Stores information about the user, their preferences, and project-specific requirements.
- `STATE_PROTOCOL.md`: Describes the protocols for managing agent state and conversation flow.
- `MEMORY.md`: The central index for personal, persistent memories. It serves as a pointer to more detailed notes stored in this directory.

## Usage Guidelines

- **Do not commit:** The contents of the `workspace/` directory (specifically `.skills/` and any memory files) are generally intended to be local-only or project-specific and are usually ignored by version control.
- **Maintain Consistency:** Keep your `AGENT.md` and `USER.md` up-to-date to ensure the agent remains aligned with your project requirements and personal workflow.
- **Manage Skills:** Use the `skill-creator` tool to safely add or update skills within the `.skills/` directory.
