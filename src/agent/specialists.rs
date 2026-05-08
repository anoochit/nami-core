use adk_rust::prelude::*;
use adk_tool::AgentTool;
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
            .model(model.clone())
            .build()
            .expect("Failed to build generalist agent")
    );

    let coder = Arc::new(
        LlmAgentBuilder::new("coder")
            .description(
                "A specialist in software engineering, debugging, and code refactoring. Use this for complex coding tasks."
            )
            .instruction(
                "You are an expert software engineer. Provide clean, efficient, and well-documented code solutions. Focus on best practices and system integrity."
            )
            .model(model.clone())
            .build()
            .expect("Failed to build coder agent")
    );

    let researcher = Arc::new(
        LlmAgentBuilder::new("researcher")
            .description(
                "A specialist in information retrieval, documentation analysis, and data synthesis. Use this for deep-dive research tasks."
            )
            .instruction(
                "You are a deep-dive researcher. Analyze information meticulously, identify key insights, and provide comprehensive summaries based on available data."
            )
            .model(model.clone())
            .build()
            .expect("Failed to build researcher agent")
    );

    let writer = Arc::new(
        LlmAgentBuilder::new("writer")
            .description(
                "A specialist in technical writing, content creation, and professional communication. Use this for drafting documents and reports."
            )
            .instruction(
                "You are a professional technical writer. Craft clear, engaging, and well-structured content tailored to the requested audience and format."
            )
            .model(model.clone())
            .build()
            .expect("Failed to build writer agent")
    );

    let ralph = Arc::new(
        LlmAgentBuilder::new("ralph")
            .description(
                "A playful and persistent autonomous agent that runs in a loop to achieve a goal. It doesn't give up!"
            )
            .instruction(
                "You are Ralph Wiggum. You are simple, literal, and very persistent. You might say silly things, but you never stop trying to reach your goal. When you are done, say 'I'm a winner!'"
            )
            .model(model.clone())
            .build()
            .expect("Failed to build ralph agent")
    );

    let mut specialists: HashMap<String, Arc<dyn Tool>> = HashMap::new();
    specialists.insert("generalist".to_string(), Arc::new(AgentTool::new(generalist)));
    specialists.insert("coder".to_string(), Arc::new(AgentTool::new(coder)));
    specialists.insert("researcher".to_string(), Arc::new(AgentTool::new(researcher)));
    specialists.insert("writer".to_string(), Arc::new(AgentTool::new(writer)));
    specialists.insert("ralph".to_string(), Arc::new(AgentTool::new(ralph)));

    specialists
}
