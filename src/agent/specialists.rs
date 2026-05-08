use adk_rust::prelude::*;
use adk_rust::tool::AgentTool;
use std::collections::HashMap;
use std::sync::Arc;


/// Returns a map of available specialist agents.
///
/// Each specialist is wrapped as a `Tool` to be used by the main agent.
pub fn get_specialists(model: Arc<dyn Llm>) -> HashMap<String, Arc<dyn Tool>> {
    let generalist = Arc::new(
        LlmAgentBuilder::new("generalist")
            .description(
                "A high-efficiency agent with access to all tools. Use this for repetitive batch tasks or high-volume data processing to keep the main conversation history lean."
            )
            .instruction(
                "You are a generalist agent. Perform the requested batch tasks or data processing efficiently."
            )
            .model(model)
            .build()
            .expect("Failed to build generalist agent")
    );

    let generalist_tool: Arc<dyn Tool> = Arc::new(AgentTool::new(generalist));

    let mut specialists: HashMap<String, Arc<dyn Tool>> = HashMap::new();
    specialists.insert("generalist".to_string(), generalist_tool);

    specialists
}
