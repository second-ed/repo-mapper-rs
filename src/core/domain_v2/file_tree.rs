use crate::core::domain_v2::file_node::FileNode;
use std::collections::HashMap;

#[derive(Debug)]
pub struct FileTree {
    pub nodes: HashMap<String, FileTree>,
}

impl FileTree {
    pub fn new() -> Self {
        FileTree {
            nodes: HashMap::new(),
        }
    }

    fn insert(&mut self, file_node: &FileNode) {
        let mut node = self;
        for part in &file_node.parts {
            node = node.nodes.entry(part.clone()).or_default();
        }
    }

    pub fn create_map(mut self, nodes: Vec<FileNode>) -> Self {
        for node in nodes {
            self.insert(&node);
        }
        self
    }

    pub fn render(&self) -> String {
        fn _walk(tree: &HashMap<String, FileTree>, prefix: String, out: &mut Vec<String>) {
            let mut items: Vec<_> = tree.iter().collect();

            items.sort_by_key(|(name, node)| (node.nodes.is_empty(), name.to_owned()));

            for (i, (name, _)) in items.iter().enumerate() {
                let is_last = i == items.len() - 1;
                let connector = if is_last { "└── " } else { "├── " };
                out.push(format!("{prefix}{connector}{name}"));

                if let Some(subtree) = tree.get(*name) {
                    let new_prefix = format!("{prefix}{}", if is_last { "    " } else { "│   " });
                    _walk(&subtree.nodes, new_prefix, out);
                }
            }
        }

        let mut out = Vec::new();
        _walk(&self.nodes, String::new(), &mut out);
        out.push("\n(generated with repo-mapper-rs)".to_string());
        format!("# Repo map\n```\n{}\n::\n```", out.join("\n"))
    }
}

impl Default for FileTree {
    fn default() -> Self {
        Self::new()
    }
}
