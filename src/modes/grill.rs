use std::io::{self, Write};
use adk_rust::runner::Runner;
use crate::modes::cli::run_and_stream_prompt;
use termimad::MadSkin;
use crossterm::style::{self, Stylize};

async fn execute_silent_prompt(
    runner: &mut Runner,
    user_id: &str,
    session_id: &str,
    prompt: &str,
) -> anyhow::Result<String> {
    use adk_rust::prelude::*;
    use futures::StreamExt;
    let content = Content::new("user").with_text(prompt);
    let stream = runner.run_str(user_id, session_id, content).await?;
    let mut full_response = String::new();
    let mut stream = stream;
    while let Some(event) = stream.next().await {
        if let Ok(event) = event {
            if let Some(content) = &event.llm_response.content {
                for part in &content.parts {
                    if let Some(text) = &part.text() {
                        full_response.push_str(text);
                    }
                }
            }
        }
    }
    Ok(full_response)
}

fn parse_plan_steps(plan: &str) -> Vec<String> {
    let mut steps = Vec::new();
    for line in plan.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("- [ ]") || trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
            let step_text = trimmed
                .trim_start_matches("- [ ]")
                .trim_start_matches("- [x]")
                .trim_start_matches("- [X]")
                .trim()
                .to_string();
            if !step_text.is_empty() {
                steps.push(step_text);
            }
        }
    }
    if steps.len() >= 3 {
        return steps;
    }
    let mut step_section = false;
    for line in plan.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') && (trimmed.to_lowercase().contains("step") || trimmed.to_lowercase().contains("plan")) {
            step_section = true;
            continue;
        }
        if step_section {
            if trimmed.starts_with('#') {
                break;
            }
            if !trimmed.is_empty() {
                steps.push(trimmed.to_string());
            }
        }
    }
    steps
}

pub async fn run_grill_flow(
    args: &str,
    runner: &mut Runner,
    user_id: &str,
    session_id: &str,
    nami_skin: &MadSkin,
    provider: &str,
    model_name: &str,
    last_response: &mut Option<String>,
) -> anyhow::Result<()> {
    if args.trim().is_empty() {
        println!("{} Please provide a goal or topic. Usage: /grill <your goal>\n", style::style("⚠️").yellow());
        return Ok(());
    }

    let generate_questions_prompt = format!(
        "The user wants to plan the following goal: '{}'. \
         Please generate 3 to 5 highly precise, concise clarification questions to help design this plan. \
         Format your response ONLY as a plain list of questions, one per line, with each line starting with 'Q: ' and nothing else.",
         args
    );

    println!("\n{} Analyzing goal and generating clarification questions...", style::style("🧠").magenta().bold());
    let questions_resp = execute_silent_prompt(runner, user_id, session_id, &generate_questions_prompt).await?;
    
    let mut questions = Vec::new();
    for line in questions_resp.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Q:") || trimmed.starts_with("Q: ") {
            let q = trimmed.trim_start_matches("Q:").trim().to_string();
            if !q.is_empty() {
                questions.push(q);
            }
        }
    }

    if questions.is_empty() {
        for line in questions_resp.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && (trimmed.ends_with('?') || trimmed.starts_with('-') || trimmed.starts_with('*')) {
                let q = trimmed.trim_start_matches(|c: char| c == '-' || c == '*' || c == ' ' || c.is_digit(10) || c == '.').trim().to_string();
                if !q.is_empty() {
                    questions.push(q);
                }
            }
        }
    }

    if questions.is_empty() {
        questions.push("Can you describe any specific requirements or preferences for this plan?".to_string());
    }

    let mut answers = Vec::new();
    for (i, q) in questions.iter().enumerate() {
        println!("\n{} {}/{} > {}", style::style("❓").magenta().bold(), i + 1, questions.len(), style::style(q).bold());
        print!("Your Answer > ");
        io::stdout().flush()?;
        let mut answer = String::new();
        io::stdin().read_line(&mut answer)?;
        let trimmed_answer = answer.trim().to_string();
        answers.push(format!("Q: {}\nA: {}", q, trimmed_answer));
    }

    println!("\n{} Synthesizing implementation plan...", style::style("⚙️").cyan().bold());
    let qa_context = answers.join("\n\n");
    let plan_prompt = format!(
        "Based on the user's goal: '{}' and their answers to the clarification questions:\n\n{}\n\n\
         Please synthesize a highly precise, step-by-step implementation plan. \
         The plan MUST be formatted as a Markdown document. \
         It MUST contain a section '## Implementation Steps' (3-6 Steps) where each step is a checkbox list item of the exact format:\
         '- [ ] Step N: <detailed task explanation>'\n\
         For example:\
         '- [ ] Step 1: Create the main function'\n\
         '- [ ] Step 2: Implement error handling'\n\n\
         Keep the steps concrete and actionable so they can be parsed and executed programmatically.",
        args, qa_context
    );

    let plan_content = run_and_stream_prompt(runner, user_id, session_id, &plan_prompt, nami_skin, provider, model_name).await?;

    let nami_dir = crate::utils::get_nami_dir();
    let plans_dir = nami_dir.join("plans");
    let _ = std::fs::create_dir_all(&plans_dir);
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let plan_filename = format!("plan_{}.md", timestamp);
    let plan_path = plans_dir.join(&plan_filename);
    let _ = std::fs::write(&plan_path, &plan_content);

    println!("{} Plan saved to {}", style::style("💾").green(), style::style(plan_path.to_string_lossy()).underlined());

    let steps = parse_plan_steps(&plan_content);
    if steps.is_empty() {
        println!("{} No executable steps could be parsed from the plan.\n", style::style("⚠️").yellow());
        return Ok(());
    }

    println!("\n{} Parsed {} steps for execution.", style::style("📋").cyan().bold(), steps.len());
    for (i, step) in steps.iter().enumerate() {
        println!("  {}. {}", i + 1, step);
    }

    print!("\n{} Do you want to execute this plan? (y/n) > ", style::style("🤔").yellow().bold());
    io::stdout().flush()?;
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;
    if confirm.trim().to_lowercase() != "y" && confirm.trim().to_lowercase() != "yes" {
        println!("{} Plan execution cancelled.\n", style::style("❌").red());
        return Ok(());
    }

    println!("\n{} Starting execution of plan steps...", style::style("🚀").green().bold());
    for (i, step) in steps.iter().enumerate() {
        let header = format!("🚀 Executing Step {} of {}: {}", i + 1, steps.len(), step);
        println!("\n{}", style::style(&header).green().bold());
        println!("{}", style::style("─".repeat(header.chars().count())).green().dim());
        
        let step_prompt = if let Some(prev_resp) = last_response.as_deref() {
            format!(
                "Previous step output / state handoff:\n{}\n\n\
                 Please execute the following subsequent step of our plan, building directly upon the state / results of the previous step.\n\
                 Step {} of {}: {}",
                prev_resp, i + 1, steps.len(), step
            )
        } else {
            format!(
                "Execute the following step of our plan.\n\
                 Step {} of {}: {}",
                i + 1, steps.len(), step
            )
        };

        let resp = run_and_stream_prompt(runner, user_id, session_id, &step_prompt, nami_skin, provider, model_name).await?;
        *last_response = Some(resp);
    }

    println!("\n{} All {} steps of the plan have been successfully executed! 🎉\n", style::style("✅").green().bold(), steps.len());

    Ok(())
}