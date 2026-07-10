//! The `tools` module defines all executable capabilities available to the Nami agent.
//!
//! Each sub-module represents a specific domain-driven toolset, allowing the agent to perform real-world tasks.

pub mod current_datetime;
pub mod invoke_agent;
pub mod filesystem;
pub mod image_generator;
pub mod audio_generator;
pub mod video_generator;
pub mod memory;
pub mod parallel_tasks;
pub mod scheduler;
pub mod search;
pub mod shell;
pub mod soul;
pub mod todo;
pub mod web_fetch;
pub mod wiki;
pub mod evolution;
pub mod supervised_delegate;

