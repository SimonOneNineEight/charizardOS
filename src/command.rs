extern crate alloc;

use crate::file_system::FileSystem;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

pub fn parse_and_execute_command(command: &str, fs: &mut FileSystem) -> Result<String, String> {
    let parts: Vec<&str> = command.split_whitespace().collect();

    if parts.is_empty() {
        return Err(String::from("No command entered"));
    }

    // Match the command
    match parts[0] {
        "mkdir" => {
            if parts.len() < 2 {
                return Err(String::from("Usage: mkdir <path>"));
            }
            fs.create_directory("/", parts[1])?;
            Ok(format!("Directory '{}' created", parts[1]))
        }
        "touch" => {
            if parts.len() < 2 {
                return Err(String::from("Usage: touch <path>"));
            }
            let content: &str = if parts.len() > 2 { parts[2] } else { "" };
            let _ = fs.create_file("/", parts[1], content);
            Ok(format!("File '{}' created", parts[1]))
        }
        "ls" => {
            if parts.len() < 2 {
                return Err(String::from("Usage: ls <path>"));
            }
            let contents = fs.list_directory(parts[1])?;
            Ok(format!("Contents of '{}': {:?}", parts[1], contents))
        }
        "cat" => {
            if parts.len() < 2 {
                return Err(String::from("Usage: cat <path>"));
            }
            let content = fs.read_file("/", parts[1])?;
            Ok(String::from(content))
        }
        "rm" => {
            if parts.len() < 2 {
                return Err(String::from("Usage: rm <path>"));
            }
            fs.delete_node("/", parts[1])?;
            Ok(format!("Node '{}' deleted", parts[1]))
        }
        "rename" => {
            if parts.len() < 3 {
                return Err(String::from("Usage: rename <path> <new_name>"));
            }
            fs.rename_node("/", parts[1], parts[2])?;
            Ok(format!("Node '{}' renamed to '{}'", parts[1], parts[2]))
        }
        _ => Err(format!("Unknown command: {}", parts[0])),
    }
}
