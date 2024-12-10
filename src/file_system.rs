extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

pub enum FSNode {
    File { name: String, content: String },
    Directory { name: String, children: Vec<FSNode> },
}

pub struct FileSystem {
    root: FSNode,
}

impl FileSystem {
    pub fn new() -> Self {
        Self {
            root: FSNode::Directory {
                name: String::from("root"),
                children: Vec::new(),
            },
        }
    }

    pub fn create_file(&mut self, path: &str, name: &str, content: &str) -> Result<(), String> {
        let parent_dir = self.find_directory_mut(path)?;

        if let FSNode::Directory { children, .. } = parent_dir {
            if children.iter().any(|node| match node {
                FSNode::File { name: n, .. } => n == name,
                _ => false,
            }) {
                return Err(format!("A file with the name '{}' already exists", name));
            }

            children.push(FSNode::File {
                name: String::from(name),
                content: String::from(content),
            });

            Ok(())
        } else {
            Err(format!("'{}' is not a directory", path))
        }
    }

    pub fn create_directory(&mut self, path: &str, name: &str) -> Result<(), String> {
        // Locate the parent directory where the new directory should be created
        let parent_dir = self.find_directory_mut(path)?;

        if let FSNode::Directory { children, .. } = parent_dir {
            // Check if a file or directory with the same name already exists
            if children.iter().any(|node| match node {
                FSNode::Directory { name: n, .. } => n == name,
                _ => false,
            }) {
                return Err(format!(
                    "A directory with the name '{}' already exists",
                    name
                ));
            }

            // Add the new directory to the parent directory's children
            children.push(FSNode::Directory {
                name: String::from(name),
                children: Vec::new(), // New directory starts empty
            });

            Ok(())
        } else {
            Err(format!("'{}' is not a directory", path))
        }
    }

    pub fn read_file(&self, path: &str, name: &str) -> Result<&str, String> {
        let dir = self.find_directory(path)?;

        if let FSNode::Directory { children, .. } = dir {
            if let Some(FSNode::File { content, .. }) = children
                .iter()
                .find(|node| matches!(node, FSNode::File{ name: n, ..} if n == name))
            {
                Ok(content)
            } else {
                Err(format!("File '{}' not found in '{}'", name, path))
            }
        } else {
            Err(format!("Path '{}' is not a directory", path))
        }
    }

    pub fn list_directory(&self, path: &str) -> Result<Vec<String>, String> {
        let dir = self.find_directory(path)?;

        if let FSNode::Directory { children, .. } = dir {
            let names = children
                .iter()
                .map(|node| match node {
                    FSNode::File { name, .. } => name.clone(),
                    FSNode::Directory { name, .. } => format!("/{name}"),
                })
                .collect();

            Ok(names)
        } else {
            Err(format!("Path '{}' is not a directory", path))
        }
    }

    pub fn rename_node(
        &mut self,
        path: &str,
        old_name: &str,
        new_name: &str,
    ) -> Result<(), String> {
        let parent_dir = self.find_directory_mut(path)?;

        if let FSNode::Directory { children, .. } = parent_dir {
            if children.iter().any(|node| match node {
                FSNode::File { name, .. } | FSNode::Directory { name, .. } => name == new_name,
            }) {
                return Err(format!("A node with the name '{}' already exist", new_name));
            }

            if let Some(node) = children.iter_mut().find(|node| match node {
                FSNode::File { name, .. } | FSNode::Directory { name, .. } => name == old_name,
            }) {
                match node {
                    FSNode::Directory { name, .. } | FSNode::File { name, .. } => {
                        *name = String::from(new_name)
                    }
                }
            }

            Ok(())
        } else {
            Err(format!("'{}' is not a directory", path))
        }
    }

    pub fn delete_node(&mut self, path: &str, name: &str) -> Result<(), String> {
        // Locate the parent directory of the node to delete
        let parent_dir = self.find_directory_mut(path)?;

        if let FSNode::Directory { children, .. } = parent_dir {
            if let Some(index) = children.iter().position(|node| match node {
                FSNode::File { name: n, .. } | FSNode::Directory { name: n, .. } => n == name,
            }) {
                if let FSNode::Directory {
                    children: sub_children,
                    ..
                } = &children[index]
                {
                    if !sub_children.is_empty() {
                        return Err(format!("Directory '{}' is not empty", name));
                    }
                }

                children.remove(index);

                Ok(())
            } else {
                Err(format!("Node '{}' not found in '{}'", name, path))
            }
        } else {
            Err(format!("'{}' is not a directory", path))
        }
    }

    pub fn find_directory(&self, path: &str) -> Result<&FSNode, String> {
        let mut current = &self.root;

        for part in path.split('/') {
            if part.is_empty() {
                continue;
            }

            if let FSNode::Directory { children, .. } = current {
                current = children
                    .iter()
                    .find(|node| match node {
                        FSNode::Directory { name, .. } => name == part,
                        _ => false,
                    })
                    .ok_or_else(|| format!("Direvtory '{}' not found", path))?;
            } else {
                return Err(format!("'{}' is not a directory", part));
            }
        }

        Ok(current)
    }

    pub fn find_directory_mut(&mut self, path: &str) -> Result<&mut FSNode, String> {
        let mut current = &mut self.root;

        for part in path.split('/') {
            if part.is_empty() {
                continue;
            }

            current = match current {
                FSNode::Directory { children, .. } => children
                    .iter_mut()
                    .find(|node| matches!( node, FSNode::Directory { name, .. } if name == part))
                    .ok_or_else(|| format!("Direvtory '{}' not found", path))?,
                _ => return Err(format!("'{}' is not a directory", part)),
            }
        }

        Ok(current)
    }
}
