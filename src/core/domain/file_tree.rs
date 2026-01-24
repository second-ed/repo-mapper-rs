use crate::core::domain::file_node::FileNode;
use rayon::prelude::*;
use std::collections::HashMap;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Default)]
pub struct FileTree {
    nodes: HashMap<String, FileTree>,
    desc: Option<String>,
}

impl FileTree {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            desc: None,
        }
    }
    pub(crate) fn from_file_nodes(nodes: &[FileNode]) -> Self {
        let mut tree = FileTree::new();

        for file in nodes {
            tree.insert_file(&file.parts, &file.desc);
        }

        tree
    }

    fn insert_file(&mut self, parts: &[String], desc: &Option<String>) {
        let mut node = self;

        for part in parts {
            node = node.nodes.entry(part.clone()).or_default();
        }

        node.desc = desc.clone();
    }

    pub fn render(&self) -> String {
        fn _walk(
            tree: &HashMap<String, FileTree>,
            prefix: String,
            out: &mut Vec<(String, Option<String>)>,
        ) {
            let mut items: Vec<_> = tree.iter().collect();

            // Sort directories before files
            items.sort_by_key(|(name, node)| (node.nodes.is_empty(), name.to_owned()));

            for (i, (name, node)) in items.iter().enumerate() {
                let is_last = i == items.len() - 1;
                let connector = if is_last { "└── " } else { "├── " };

                let line = format!("{prefix}{connector}{name}");
                out.push((line, node.desc.clone()));

                if !node.nodes.is_empty() {
                    let new_prefix = format!("{prefix}{}", if is_last { "    " } else { "│   " });
                    _walk(&node.nodes, new_prefix, out);
                }
            }
        }

        let mut out = Vec::new();
        _walk(&self.nodes, String::new(), &mut out);

        let max_len = out
            .iter()
            .map(|(line, _)| UnicodeWidthStr::width(line.as_str()))
            .max()
            .unwrap_or(40);

        let padded_lines: Vec<String> = out
            .into_par_iter()
            .map(|(line, desc)| {
                if let Some(desc) = desc {
                    format!("{:<width$}  # {}", line, desc, width = max_len)
                } else {
                    line
                }
            })
            .collect();

        let mut out = padded_lines;
        out.push("\n(generated with repo-mapper-rs)".to_string());
        format!("# Repo map\n```\n{}\n::\n```", out.join("\n"))
    }
}
