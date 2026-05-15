//! The `modes` module contains the entry-point implementations for the various ways Nami can be run.
//! 
//! Each mode acts as an interface layer, bridging the user input to the core agent logic.

pub mod bot;
pub mod cli;
pub mod command_registry;
pub mod init;
pub mod api;
pub mod browse;
pub mod run;
pub mod serve;
pub mod ui_utils;
pub mod line;
pub mod startup;
pub mod scheduler;
