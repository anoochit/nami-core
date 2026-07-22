use crate::modes::command_registry::CommandRegistry;

pub struct SlashRequest<'a> {
    pub command: &'a str,
    pub args: &'a str,
    pub registry: &'a CommandRegistry,
}

pub enum SlashAction {
    RunPrompt(String),
    Reply(String),
    PassThrough,
}

pub fn dispatch(req: SlashRequest) -> SlashAction {
    let command = req.command;
    let args = req.args;

    match command {
        "/help" | "/?" => SlashAction::Reply(get_help(req.registry)),
        "/version" => {
            SlashAction::Reply(format!("Nami CLI {}", env!("CARGO_PKG_VERSION")))
        }
        "/plan" => {
            if args.is_empty() {
                return SlashAction::Reply("Usage: /plan <task description>".to_string());
            }
            let prompt = format!(
                "Create a detailed step-by-step implementation plan for the following task. \
                 The plan must outline: \
                 1. Goals & Requirements, 2. Design Decisions & Architecture, \
                 3. Success Criteria & Verification Steps, \
                 and 4. A sequential task list with concrete steps under a section \
                 '## Implementation Steps' (3-6 Steps) where each step is a checkbox list item \
                 of the exact format:\n\
                 '- [ ] Step N: <detailed task explanation>'\n\
                 For example:\n\
                 '- [ ] Step 1: Create the main function'\n\
                 '- [ ] Step 2: Implement error handling'\n\n\
                 Save the compiled plan to the workspace or `~/.nami/plans/` directory \
                 (e.g., as `plan_[date-time].md`) as a user-facing artifact, \
                 and present it clearly to the user for feedback and approval \
                 before executing any code. Task: {}",
                args
            );
            SlashAction::RunPrompt(prompt)
        }
        "/status" => {
            let prompt = "Please retrieve and report the system status using your system_status skill.".to_string();
            SlashAction::RunPrompt(prompt)
        }
        "/clear" => SlashAction::Reply("__CLEAR__".to_string()),
        "/new" => SlashAction::Reply("__NEW_SESSION__".to_string()),
        _ => {
            if let Some(prompt) = req.registry.format_prompt(command, args) {
                SlashAction::RunPrompt(prompt)
            } else {
                SlashAction::PassThrough
            }
        }
    }
}

pub fn get_help(registry: &CommandRegistry) -> String {
    let mut help = String::new();
    help.push_str("Available Commands\n\n");
    help.push_str("- /help: Show this help\n");
    help.push_str("- /clear: Clear session\n");
    help.push_str("- /new: New session\n");
    help.push_str("- /status: Agent status\n");
    help.push_str("- /version: CLI version\n");
    help.push_str("- /plan: Create a structured execution/implementation plan\n");

    if !registry.commands.is_empty() {
        help.push_str("\nCustom Commands\n\n");
        let mut commands: Vec<_> = registry.commands.iter().collect();
        commands.sort_by(|a, b| a.0.cmp(b.0));
        for (name, cmd) in commands {
            help.push_str(&format!("- {}: {}\n", name, cmd.help));
        }
    }

    help.push_str("\nExamples:\n  /plan Build AI research system\n");
    help
}
