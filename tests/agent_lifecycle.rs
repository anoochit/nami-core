use nami::runner::AgentRunner;
use adk_rust::prelude::*;
use adk_session::InMemorySessionService;
use std::sync::Arc;
use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

struct MockAgent;

#[async_trait]
impl Agent for MockAgent {
    fn name(&self) -> &str { "mock" }
    fn description(&self) -> &str { "mock agent" }
    fn sub_agents(&self) -> &[Arc<dyn Agent>] { &[] }
    async fn run(&self, _ctx: Arc<dyn InvocationContext>) -> adk_rust::Result<Pin<Box<dyn Stream<Item = adk_rust::Result<adk_session::Event>> + Send + 'static>>> {
        unimplemented!()
    }
}

struct MockLlm;
#[async_trait]
impl Llm for MockLlm {
    fn name(&self) -> &str { "mock-model" }
    async fn generate_content(&self, _req: LlmRequest, _stream: bool) -> adk_rust::Result<Pin<Box<dyn Stream<Item = adk_rust::Result<LlmResponse>> + Send + 'static>>> {
        unimplemented!()
    }
}

struct MockMemory;
#[async_trait]
impl adk_rust::Memory for MockMemory {
    async fn add(&self, _entry: adk_rust::MemoryEntry) -> adk_rust::Result<()> { Ok(()) }
    async fn search(&self, _query: &str) -> adk_rust::Result<Vec<adk_rust::MemoryEntry>> { Ok(vec![]) }
}

#[tokio::test]
async fn test_agent_runner_initialization() {
    let agent = Arc::new(MockAgent);
    let sessions = Arc::new(InMemorySessionService::new());
    let memory = Arc::new(MockMemory);
    let model = Arc::new(MockLlm);
    
    let runner = AgentRunner::new(
        agent,
        sessions,
        memory,
        "test_app",
        model,
    );
    
    assert_eq!(runner.app_name(), "test_app");
}
