use crate::error::AgentError;
use std::io::{self, Write};
use std::path::Path;

pub fn prompt(message: &str) -> Result<String, AgentError> {
    print!("{}", message);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

pub fn prompt_watch_paths() -> Result<Vec<String>, AgentError> {
    let mut paths = Vec::new();
    loop {
        let path = prompt("Enter path to watch (or 'done' to finish): ")?;
        if path.to_lowercase() == "done" {
            break;
        }
        if Path::new(&path).exists() {
            paths.push(path);
        } else {
            println!("Path does not exist: {}", path);
        }
    }
    Ok(paths)
}