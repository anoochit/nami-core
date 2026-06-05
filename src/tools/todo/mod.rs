use crate::utils::get_nami_dir;
use adk_rust::Tool;
use adk_rust::serde::{Deserialize, Serialize};
use adk_tool::{AdkError, tool};
use schemars::JsonSchema;
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
struct Todo {
    id: usize,
    description: String,
    done: bool,
}

#[derive(Deserialize, JsonSchema)]
struct AddTodoArgs {
    /// The todo description.
    description: String,
}

#[derive(Deserialize, JsonSchema)]
struct TodoIdArgs {
    /// The ID of the todo item.
    id: usize,
}

async fn get_todo_file() -> std::result::Result<PathBuf, AdkError> {
    Ok(get_nami_dir().join("todos.json"))
}

async fn load_todos() -> std::result::Result<Vec<Todo>, AdkError> {
    let path = get_todo_file().await?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&path)
        .await
        .map_err(|e| AdkError::tool(format!("Failed to read todos: {}", e)))?;
    serde_json::from_str(&content)
        .map_err(|e| AdkError::tool(format!("Failed to parse todos: {}", e)))
}

async fn save_todos(todos: &[Todo]) -> std::result::Result<(), AdkError> {
    let path = get_todo_file().await?;
    let content = serde_json::to_string_pretty(todos)
        .map_err(|e| AdkError::tool(format!("Failed to serialize todos: {}", e)))?;
    fs::write(&path, content)
        .await
        .map_err(|e| AdkError::tool(format!("Failed to write todos: {}", e)))
}

#[derive(Deserialize, JsonSchema)]
struct ListTodosArgs {}

/// Adds a new todo item to the TODO list.
#[tool]
async fn add_todo(args: AddTodoArgs) -> std::result::Result<Value, AdkError> {
    let mut todos = load_todos().await?;
    let next_id = todos.iter().map(|t| t.id).max().unwrap_or(0) + 1;
    let new_todo = Todo {
        id: next_id,
        description: args.description.clone(),
        done: false,
    };
    todos.push(new_todo);
    save_todos(&todos).await?;
    Ok(json!({"status": "success", "message": format!("Added todo item: {}", args.description), "id": next_id}))
}

/// Lists all todo items in the TODO list.
#[tool]
async fn list_todos(_args: ListTodosArgs) -> std::result::Result<Value, AdkError> {
    let todos = load_todos().await?;
    if todos.is_empty() {
        Ok(json!({"message": "Your TODO list is empty."}))
    } else {
        Ok(json!({ "todo_items": todos }))
    }
}

/// Marks a todo item as completed.
#[tool]
async fn mark_todo_done(args: TodoIdArgs) -> std::result::Result<Value, AdkError> {
    let mut todos = load_todos().await?;
    if let Some(todo) = todos.iter_mut().find(|t| t.id == args.id) {
        todo.done = true;
        save_todos(&todos).await?;
        Ok(json!({"status": "success", "message": format!("Marked todo item #{} as done", args.id)}))
    } else {
        Err(AdkError::tool(format!("Todo item #{} not found", args.id)))
    }
}

/// Removes a todo item from the TODO list.
#[tool]
async fn remove_todo(args: TodoIdArgs) -> std::result::Result<Value, AdkError> {
    let mut todos = load_todos().await?;
    let initial_len = todos.len();
    todos.retain(|t| t.id != args.id);
    if todos.len() < initial_len {
        save_todos(&todos).await?;
        Ok(json!({"status": "success", "message": format!("Removed todo item #{}", args.id)}))
    } else {
        Err(AdkError::tool(format!("Todo item #{} not found", args.id)))
    }
}

pub fn todo_tools() -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(AddTodo),
        Arc::new(ListTodos),
        Arc::new(MarkTodoDone),
        Arc::new(RemoveTodo),
    ]
}
