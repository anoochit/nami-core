# Nami Agent Skills Guide

Agent Skills extend the capabilities of Nami by providing specialized knowledge, workflows, or tool integrations. All active skills are stored in the `workspace/.skills/` directory.

## Creating a New Skill

There are two primary ways to create an Agent Skill: manual creation or using the built-in `skill-creator`.

### 1. Manual Creation
To create a skill manually, follow these steps:

1. **Create the Directory:** Create a new folder for your skill within `workspace/.skills/`.
   ```bash
   mkdir workspace/.skills/my-new-skill
   ```
2. **Define the Skill:** Create a `SKILL.md` file inside that folder. This file should contain:
   - **Name:** The unique name of your skill.
   - **Description:** A concise explanation of what the skill does.
   - **Instructions:** Detailed procedural guidance for the agent on when and how to use this skill.
   - **Available Resources:** Any local scripts, templates, or files the skill relies on.

### 2. Using `skill-creator`
For a guided experience, you can use the built-in `skill-creator` tool:

1. **Prompts:** Create skill provide the name, description, and key capabilities of your new skill, then The `skill-creator` will automatically generate the required directory structure and the `SKILL.md` file for you.

## Using Skills

Once created, skills are automatically available to the agent. You can view, manage, and invoke these skills using Nami's built-in commands or by referencing them in your prompts when you need the agent to perform a task associated with a specific skill.

## Best Practices
- Keep your `SKILL.md` file concise and actionable.
- Ensure that the instructions within `SKILL.md` follow the project's safety and security guidelines.
- Test your new skill by providing a prompt that triggers its specific functionality to verify that the agent adheres to your defined instructions.
