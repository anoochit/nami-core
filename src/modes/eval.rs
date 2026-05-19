use adk_rust::prelude::*;
use crate::runner::AgentRunner;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Instant;
use regex::Regex;

#[derive(Debug, Deserialize)]
pub struct TestCase {
    pub name: String,
    pub prompt: String,
    pub expected: String,
    #[serde(default = "default_match_type")]
    pub match_type: String,
}

fn default_match_type() -> String {
    "contains".to_string()
}

#[derive(Debug, Default)]
pub struct EvalReport {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub total_duration_ms: u128,
}

pub async fn run_eval(
    agent: Arc<dyn Agent>,
    sessions: Arc<dyn adk_session::SessionService>,
    memory: Arc<dyn adk_rust::Memory>,
    model: Arc<dyn Llm>,
) -> anyhow::Result<()> {
    let dataset_path = "evals.yaml";
    if !std::path::Path::new(dataset_path).exists() {
        println!("Error: evals.yaml not found. Please create one to run evaluations.");
        return Ok(());
    }

    let content = std::fs::read_to_string(dataset_path)?;
    let test_cases: Vec<TestCase> = serde_yaml::from_str(&content)?;

    let runner = AgentRunner::new(agent, sessions, memory, "eval", model);
    let mut report = EvalReport::default();
    report.total = test_cases.len();

    println!("Starting evaluation with {} test cases...", test_cases.len());
    println!("--------------------------------------------------");

    for test in test_cases {
        print!("Test: {} ... ", test.name);
        std::io::Write::flush(&mut std::io::stdout())?;

        let start = Instant::now();
        let result = runner.run("eval_user", &format!("eval_{}", test.name), &test.prompt).await;
        let duration = start.elapsed().as_millis();
        report.total_duration_ms += duration;

        match result {
            Ok(response) => {
                let passed = match test.match_type.as_str() {
                    "exact" => response.trim() == test.expected.trim(),
                    "contains" => response.contains(&test.expected),
                    "regex" => {
                        let re = Regex::new(&test.expected)?;
                        re.is_match(&response)
                    }
                    _ => {
                        println!("Unknown match type: {}", test.match_type);
                        false
                    }
                };

                if passed {
                    println!("PASSED ({}ms)", duration);
                    report.passed += 1;
                } else {
                    println!("FAILED ({}ms)", duration);
                    println!("  Expected ({}): {}", test.match_type, test.expected);
                    println!("  Got: {}", response);
                    report.failed += 1;
                }
            }
            Err(e) => {
                println!("ERROR: {:?}", e);
                report.failed += 1;
            }
        }
    }

    println!("--------------------------------------------------");
    println!("Evaluation Summary:");
    println!("  Total:  {}", report.total);
    println!("  Passed: {}", report.passed);
    println!("  Failed: {}", report.failed);
    println!("  Avg Latency: {}ms", if report.total > 0 { report.total_duration_ms / report.total as u128 } else { 0 });
    println!("--------------------------------------------------");

    Ok(())
}
