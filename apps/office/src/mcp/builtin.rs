//! Built-in Native Tools
//!
//! Core tools implemented in Rust that are always available.

use std::path::PathBuf;
use serde_json::{json, Value};
use tokio::fs;
use tokio::process::Command;
use tracing::{info, warn};

use crate::{Result, OfficeError};
use crate::mcp::protocol::CallToolResult;
use crate::mcp::native::{
    NativeToolProvider, ToolProvider, 
    text_result, error_result, simple_schema, json_schema
};

/// Create the core native tools provider
pub fn create_core_tools() -> NativeToolProvider {
    let mut provider = NativeToolProvider::new("native");

    // ============ File System Tools ============
    
    provider.register_fn(
        "read_file",
        "Read the contents of a file at the specified path",
        simple_schema(&[
            ("path", "The absolute path to the file to read", true),
            ("offset", "Line number to start from (1-indexed)", false),
            ("limit", "Maximum number of lines to read", false),
        ]),
        read_file_handler,
    );

    provider.register_fn(
        "write_file",
        "Write content to a file, creating it if it doesn't exist",
        simple_schema(&[
            ("path", "The absolute path to the file", true),
            ("content", "The content to write", true),
        ]),
        write_file_handler,
    );

    provider.register_fn(
        "list_directory",
        "List contents of a directory",
        simple_schema(&[
            ("path", "The absolute path to the directory", true),
        ]),
        list_directory_handler,
    );

    provider.register_fn(
        "file_exists",
        "Check if a file or directory exists",
        simple_schema(&[
            ("path", "The path to check", true),
        ]),
        file_exists_handler,
    );

    provider.register_fn(
        "delete_file",
        "Delete a file or empty directory",
        simple_schema(&[
            ("path", "The path to delete", true),
        ]),
        delete_file_handler,
    );

    // ============ Terminal/Shell Tools ============

    provider.register_fn(
        "run_command",
        "Execute a shell command and return the output",
        json_schema(json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The command to execute"
                },
                "cwd": {
                    "type": "string",
                    "description": "Working directory for the command"
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Timeout in seconds (default: 30)"
                }
            },
            "required": ["command"]
        })),
        run_command_handler,
    );

    // ============ Search Tools ============

    provider.register_fn(
        "grep_search",
        "Search for a pattern in files",
        json_schema(json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "The pattern to search for"
                },
                "path": {
                    "type": "string",
                    "description": "Directory or file to search in"
                },
                "is_regex": {
                    "type": "boolean",
                    "description": "Whether pattern is a regex"
                },
                "include": {
                    "type": "string",
                    "description": "Glob pattern for files to include"
                }
            },
            "required": ["pattern", "path"]
        })),
        grep_search_handler,
    );

    provider.register_fn(
        "find_files",
        "Find files matching a glob pattern",
        simple_schema(&[
            ("pattern", "Glob pattern to match (e.g., **/*.rs)", true),
            ("path", "Base directory to search from", true),
        ]),
        find_files_handler,
    );

    // ============ Git Tools ============

    provider.register_fn(
        "git_status",
        "Get the current git status",
        simple_schema(&[
            ("path", "Path to the git repository", true),
        ]),
        git_status_handler,
    );

    provider.register_fn(
        "git_diff",
        "Get git diff for staged or unstaged changes",
        json_schema(json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the git repository"
                },
                "staged": {
                    "type": "boolean",
                    "description": "Show staged changes only"
                },
                "file": {
                    "type": "string",
                    "description": "Specific file to diff"
                }
            },
            "required": ["path"]
        })),
        git_diff_handler,
    );

    provider.register_fn(
        "git_log",
        "Get recent git commits",
        json_schema(json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the git repository"
                },
                "count": {
                    "type": "integer",
                    "description": "Number of commits to show (default: 10)"
                },
                "oneline": {
                    "type": "boolean",
                    "description": "Use oneline format"
                }
            },
            "required": ["path"]
        })),
        git_log_handler,
    );

    // ============ Utility Tools ============

    provider.register_fn(
        "get_current_time",
        "Get the current date and time",
        simple_schema(&[]),
        |_| async move {
            let now = chrono::Utc::now();
            Ok(text_result(&format!(
                "Current time: {} UTC\nISO: {}",
                now.format("%Y-%m-%d %H:%M:%S"),
                now.to_rfc3339()
            )))
        },
    );

    provider.register_fn(
        "calculate",
        "Evaluate a mathematical expression",
        simple_schema(&[
            ("expression", "The mathematical expression to evaluate", true),
        ]),
        calculate_handler,
    );

    provider
}

// ============ Handler Implementations ============

async fn read_file_handler(args: Option<Value>) -> Result<CallToolResult> {
    let args = args.ok_or_else(|| OfficeError::McpError("Missing arguments".into()))?;
    let path = args.get("path")
        .and_then(|p| p.as_str())
        .ok_or_else(|| OfficeError::McpError("Missing path".into()))?;

    let offset = args.get("offset").and_then(|o| o.as_u64()).unwrap_or(1) as usize;
    let limit = args.get("limit").and_then(|l| l.as_u64()).map(|l| l as usize);

    match fs::read_to_string(path).await {
        Ok(content) => {
            let lines: Vec<&str> = content.lines().collect();
            let start = (offset.saturating_sub(1)).min(lines.len());
            let end = limit.map(|l| (start + l).min(lines.len())).unwrap_or(lines.len());
            
            let selected: Vec<&str> = lines[start..end].to_vec();
            let result = selected.join("\n");
            
            Ok(text_result(&format!(
                "File: {} (lines {}-{} of {})\n\n{}",
                path, start + 1, end, lines.len(), result
            )))
        }
        Err(e) => Ok(error_result(&format!("Failed to read file: {}", e))),
    }
}

async fn write_file_handler(args: Option<Value>) -> Result<CallToolResult> {
    let args = args.ok_or_else(|| OfficeError::McpError("Missing arguments".into()))?;
    let path = args.get("path")
        .and_then(|p| p.as_str())
        .ok_or_else(|| OfficeError::McpError("Missing path".into()))?;
    let content = args.get("content")
        .and_then(|c| c.as_str())
        .ok_or_else(|| OfficeError::McpError("Missing content".into()))?;

    // Ensure parent directory exists
    if let Some(parent) = PathBuf::from(path).parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).await
                .map_err(|e| OfficeError::McpError(format!("Failed to create directory: {}", e)))?;
        }
    }

    match fs::write(path, content).await {
        Ok(()) => Ok(text_result(&format!("Successfully wrote {} bytes to {}", content.len(), path))),
        Err(e) => Ok(error_result(&format!("Failed to write file: {}", e))),
    }
}

async fn list_directory_handler(args: Option<Value>) -> Result<CallToolResult> {
    let args = args.ok_or_else(|| OfficeError::McpError("Missing arguments".into()))?;
    let path = args.get("path")
        .and_then(|p| p.as_str())
        .ok_or_else(|| OfficeError::McpError("Missing path".into()))?;

    match fs::read_dir(path).await {
        Ok(mut entries) => {
            let mut items = Vec::new();
            while let Ok(Some(entry)) = entries.next_entry().await {
                let name = entry.file_name().to_string_lossy().to_string();
                let is_dir = entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false);
                items.push(if is_dir { format!("{}/", name) } else { name });
            }
            items.sort();
            Ok(text_result(&items.join("\n")))
        }
        Err(e) => Ok(error_result(&format!("Failed to list directory: {}", e))),
    }
}

async fn file_exists_handler(args: Option<Value>) -> Result<CallToolResult> {
    let args = args.ok_or_else(|| OfficeError::McpError("Missing arguments".into()))?;
    let path = args.get("path")
        .and_then(|p| p.as_str())
        .ok_or_else(|| OfficeError::McpError("Missing path".into()))?;

    let exists = tokio::fs::metadata(path).await.is_ok();
    Ok(text_result(&format!("{}", exists)))
}

async fn delete_file_handler(args: Option<Value>) -> Result<CallToolResult> {
    let args = args.ok_or_else(|| OfficeError::McpError("Missing arguments".into()))?;
    let path = args.get("path")
        .and_then(|p| p.as_str())
        .ok_or_else(|| OfficeError::McpError("Missing path".into()))?;

    let metadata = fs::metadata(path).await
        .map_err(|e| OfficeError::McpError(format!("Path not found: {}", e)))?;

    if metadata.is_dir() {
        match fs::remove_dir(path).await {
            Ok(()) => Ok(text_result(&format!("Deleted directory: {}", path))),
            Err(e) => Ok(error_result(&format!("Failed to delete directory: {}", e))),
        }
    } else {
        match fs::remove_file(path).await {
            Ok(()) => Ok(text_result(&format!("Deleted file: {}", path))),
            Err(e) => Ok(error_result(&format!("Failed to delete file: {}", e))),
        }
    }
}

async fn run_command_handler(args: Option<Value>) -> Result<CallToolResult> {
    let args = args.ok_or_else(|| OfficeError::McpError("Missing arguments".into()))?;
    let command = args.get("command")
        .and_then(|c| c.as_str())
        .ok_or_else(|| OfficeError::McpError("Missing command".into()))?;
    let cwd = args.get("cwd").and_then(|c| c.as_str());
    let timeout = args.get("timeout_secs").and_then(|t| t.as_u64()).unwrap_or(30);

    info!("Running command: {}", command);

    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(command);
    
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(timeout),
        cmd.output(),
    ).await;

    match result {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            let mut result = String::new();
            if !stdout.is_empty() {
                result.push_str(&stdout);
            }
            if !stderr.is_empty() {
                if !result.is_empty() {
                    result.push_str("\n--- STDERR ---\n");
                }
                result.push_str(&stderr);
            }
            
            if output.status.success() {
                Ok(text_result(&result))
            } else {
                Ok(error_result(&format!(
                    "Command exited with code {}\n{}",
                    output.status.code().unwrap_or(-1),
                    result
                )))
            }
        }
        Ok(Err(e)) => Ok(error_result(&format!("Failed to execute command: {}", e))),
        Err(_) => Ok(error_result(&format!("Command timed out after {} seconds", timeout))),
    }
}

async fn grep_search_handler(args: Option<Value>) -> Result<CallToolResult> {
    let args = args.ok_or_else(|| OfficeError::McpError("Missing arguments".into()))?;
    let pattern = args.get("pattern")
        .and_then(|p| p.as_str())
        .ok_or_else(|| OfficeError::McpError("Missing pattern".into()))?;
    let path = args.get("path")
        .and_then(|p| p.as_str())
        .ok_or_else(|| OfficeError::McpError("Missing path".into()))?;
    let is_regex = args.get("is_regex").and_then(|r| r.as_bool()).unwrap_or(false);
    let include = args.get("include").and_then(|i| i.as_str());

    let mut cmd = Command::new("grep");
    cmd.arg("-rn"); // recursive, line numbers
    
    if is_regex {
        cmd.arg("-E");
    } else {
        cmd.arg("-F");
    }
    
    if let Some(inc) = include {
        cmd.arg("--include").arg(inc);
    }
    
    cmd.arg(pattern).arg(path);

    match cmd.output().await {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.is_empty() {
                Ok(text_result("No matches found"))
            } else {
                // Limit output
                let lines: Vec<&str> = stdout.lines().take(100).collect();
                let truncated = stdout.lines().count() > 100;
                let mut result = lines.join("\n");
                if truncated {
                    result.push_str("\n... (truncated, more matches exist)");
                }
                Ok(text_result(&result))
            }
        }
        Err(e) => Ok(error_result(&format!("Grep failed: {}", e))),
    }
}

async fn find_files_handler(args: Option<Value>) -> Result<CallToolResult> {
    let args = args.ok_or_else(|| OfficeError::McpError("Missing arguments".into()))?;
    let pattern = args.get("pattern")
        .and_then(|p| p.as_str())
        .ok_or_else(|| OfficeError::McpError("Missing pattern".into()))?;
    let path = args.get("path")
        .and_then(|p| p.as_str())
        .ok_or_else(|| OfficeError::McpError("Missing path".into()))?;

    let mut cmd = Command::new("find");
    cmd.arg(path).arg("-name").arg(pattern);

    match cmd.output().await {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.is_empty() {
                Ok(text_result("No files found"))
            } else {
                let lines: Vec<&str> = stdout.lines().take(100).collect();
                Ok(text_result(&lines.join("\n")))
            }
        }
        Err(e) => Ok(error_result(&format!("Find failed: {}", e))),
    }
}

async fn git_status_handler(args: Option<Value>) -> Result<CallToolResult> {
    let args = args.ok_or_else(|| OfficeError::McpError("Missing arguments".into()))?;
    let path = args.get("path")
        .and_then(|p| p.as_str())
        .ok_or_else(|| OfficeError::McpError("Missing path".into()))?;

    let output = Command::new("git")
        .arg("-C").arg(path)
        .arg("status")
        .arg("--porcelain=v2")
        .arg("--branch")
        .output()
        .await;

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            Ok(text_result(&stdout))
        }
        Err(e) => Ok(error_result(&format!("Git status failed: {}", e))),
    }
}

async fn git_diff_handler(args: Option<Value>) -> Result<CallToolResult> {
    let args = args.ok_or_else(|| OfficeError::McpError("Missing arguments".into()))?;
    let path = args.get("path")
        .and_then(|p| p.as_str())
        .ok_or_else(|| OfficeError::McpError("Missing path".into()))?;
    let staged = args.get("staged").and_then(|s| s.as_bool()).unwrap_or(false);
    let file = args.get("file").and_then(|f| f.as_str());

    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(path).arg("diff");
    
    if staged {
        cmd.arg("--staged");
    }
    
    if let Some(f) = file {
        cmd.arg("--").arg(f);
    }

    match cmd.output().await {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.is_empty() {
                Ok(text_result("No changes"))
            } else {
                Ok(text_result(&stdout))
            }
        }
        Err(e) => Ok(error_result(&format!("Git diff failed: {}", e))),
    }
}

async fn git_log_handler(args: Option<Value>) -> Result<CallToolResult> {
    let args = args.ok_or_else(|| OfficeError::McpError("Missing arguments".into()))?;
    let path = args.get("path")
        .and_then(|p| p.as_str())
        .ok_or_else(|| OfficeError::McpError("Missing path".into()))?;
    let count = args.get("count").and_then(|c| c.as_u64()).unwrap_or(10);
    let oneline = args.get("oneline").and_then(|o| o.as_bool()).unwrap_or(true);

    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(path)
        .arg("log")
        .arg(format!("-{}", count));
    
    if oneline {
        cmd.arg("--oneline");
    }

    match cmd.output().await {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(text_result(&stdout))
        }
        Err(e) => Ok(error_result(&format!("Git log failed: {}", e))),
    }
}

async fn calculate_handler(args: Option<Value>) -> Result<CallToolResult> {
    let args = args.ok_or_else(|| OfficeError::McpError("Missing arguments".into()))?;
    let expression = args.get("expression")
        .and_then(|e| e.as_str())
        .ok_or_else(|| OfficeError::McpError("Missing expression".into()))?;

    // Use bc for calculation (available on most systems)
    let output = Command::new("echo")
        .arg(expression)
        .output()
        .await;

    match output {
        Ok(_) => {
            // Use python for safer evaluation
            let result = Command::new("python3")
                .arg("-c")
                .arg(format!("print(eval('{}'))", expression.replace("'", "\\'")))
                .output()
                .await;

            match result {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    if out.status.success() {
                        Ok(text_result(&format!("{} = {}", expression, stdout)))
                    } else {
                        Ok(error_result(&format!("Calculation error: {}", stderr)))
                    }
                }
                Err(e) => Ok(error_result(&format!("Failed to calculate: {}", e))),
            }
        }
        Err(e) => Ok(error_result(&format!("Failed: {}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_core_tools_creation() {
        let provider = create_core_tools();
        let tools = provider.list_tools().await.unwrap();
        
        // Should have all core tools
        assert!(tools.iter().any(|t| t.name == "read_file"));
        assert!(tools.iter().any(|t| t.name == "write_file"));
        assert!(tools.iter().any(|t| t.name == "run_command"));
        assert!(tools.iter().any(|t| t.name == "git_status"));
        assert!(tools.iter().any(|t| t.name == "grep_search"));
    }

    #[tokio::test]
    async fn test_get_current_time() {
        let provider = create_core_tools();
        let result = provider.call_tool("get_current_time", None).await.unwrap();
        assert!(!result.is_error.unwrap_or(true));
    }
}
