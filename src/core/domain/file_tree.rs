use crate::core::domain::repo_entry::RepoEntry;
use rayon::prelude::*;
use std::collections::HashMap;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Default)]
pub struct FileTree {
    nodes: HashMap<String, FileTree>,
    desc: Option<String>,
    is_dir: bool,
}

impl FileTree {
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            desc: None,
            is_dir: true,
        }
    }
    pub(crate) fn from_repo_entries(entries: &[RepoEntry]) -> Self {
        let mut tree = FileTree::new();

        for entry in entries {
            tree.insert_entry(entry);
        }

        tree
    }

    fn insert_entry(&mut self, entry: &RepoEntry) {
        let mut node = self;

        let mut components = entry.path().components().peekable();

        while let Some(component) = components.next() {
            let name = component.as_os_str().to_string_lossy().into_owned();
            node = node.nodes.entry(name).or_default();

            // every component before the final one is a dir
            if components.peek().is_some() {
                node.is_dir = true;
            }
        }

        node.is_dir |= entry.is_dir();

        if let Some(desc) = entry.desc() {
            node.desc = Some(desc.to_owned());
        }
    }

    #[must_use]
    pub fn render(&self) -> String {
        fn walk(
            tree: &HashMap<String, FileTree>,
            prefix: &str,
            out: &mut Vec<(String, Option<String>)>,
        ) {
            let mut items: Vec<_> = tree.iter().collect();

            // Sort directories before files
            items.sort_by_key(|(name, node)| (!node.is_dir, name.to_owned()));

            for (i, (name, node)) in items.iter().enumerate() {
                let is_last = i == items.len() - 1;
                let connector = if is_last { "└── " } else { "├── " };

                let line = format!("{prefix}{connector}{name}");
                out.push((line, node.desc.clone()));

                if !node.nodes.is_empty() {
                    let new_prefix = format!("{prefix}{}", if is_last { "    " } else { "│   " });
                    walk(&node.nodes, &new_prefix, out);
                }
            }
        }

        let mut out = Vec::new();
        walk(&self.nodes, "", &mut out);

        let max_len = out
            .iter()
            .map(|(line, _)| UnicodeWidthStr::width(line.as_str()))
            .max()
            .unwrap_or(40);

        let padded_lines: Vec<String> = out
            .into_par_iter()
            .with_min_len(1_000)
            .map(|(line, desc)| {
                if let Some(desc) = desc {
                    format!("{line:<max_len$}  # {desc}")
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
