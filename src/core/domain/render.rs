use crate::core::domain::repo_entry::RepoEntry;
use std::{
    cmp::{max, Ordering},
    path::Component,
};
use unicode_width::UnicodeWidthStr;

pub fn generate_repo_map(mut repo_entries: Vec<RepoEntry>) -> String {
    repo_entries.sort_unstable_by(cmp_entries);

    let is_lasts = calculate_is_last(&repo_entries);

    let (objects, max_width) = populate_render_objects(&repo_entries, &is_lasts);

    let mut lines_with_descs = objects
        .into_iter()
        .map(|obj| {
            let line = obj.line;
            match obj.desc {
                Some(desc) => {
                    let width = UnicodeWidthStr::width(line.as_str());
                    let padding = max_width - width;

                    format!("{line}{}  # {desc}", " ".repeat(padding))
                }
                None => line,
            }
        })
        .collect::<Vec<String>>();

    lines_with_descs.push("\n(generated with repo-mapper-rs)".to_string());
    format!("# Repo map\n```\n{}\n::\n```", lines_with_descs.join("\n"))
}

#[inline]
fn cmp_entries(a: &RepoEntry, b: &RepoEntry) -> Ordering {
    let a_components: Vec<_> = a.path().components().collect();
    let b_components: Vec<_> = b.path().components().collect();

    for (i, (a_component, b_component)) in a_components.iter().zip(&b_components).enumerate() {
        if a_component == b_component {
            continue;
        }

        let a_is_dir = component_is_dir(a, &a_components, i);
        let b_is_dir = component_is_dir(b, &b_components, i);

        return b_is_dir
            .cmp(&a_is_dir) // directories first
            .then_with(|| a_component.as_os_str().cmp(b_component.as_os_str()));
    }
    a_components.len().cmp(&b_components.len())
}

/// A component is a directory if:
/// - it isn't the final component (it must lead to another component)
/// - it is the final component and this `RepoEntry` is a directory
#[inline]
fn component_is_dir(entry: &RepoEntry, components: &[Component<'_>], i: usize) -> bool {
    i + 1 < components.len() || entry.is_dir()
}

#[inline]
fn populate_render_objects(
    repo_entries: &[RepoEntry],
    is_lasts: &[bool],
) -> (Vec<RenderObject>, usize) {
    let mut render_objects: Vec<RenderObject> = Vec::with_capacity(repo_entries.len());
    let mut continuations: Vec<bool> = Vec::new();

    let mut max_width: usize = 2;

    for (index, entry) in repo_entries.iter().enumerate() {
        let depth = entry.path().components().count() - 1;

        continuations.truncate(depth);
        let prefix = continuation_to_prefix(&continuations);
        continuations.push(!is_lasts[index]);

        let render_type = match (entry.is_dir(), is_lasts[index]) {
            (true, true) => RenderType::LastDir.as_str(),
            (true, false) => RenderType::Dir.as_str(),
            (false, true) => RenderType::LastFile.as_str(),
            (false, false) => RenderType::File.as_str(),
        };

        let name = entry
            .path()
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();

        let line = format!("{prefix}{render_type}{name}");

        max_width = max(max_width, UnicodeWidthStr::width(line.as_str()));
        render_objects.push(RenderObject::new(line, entry.desc().map(str::to_owned)));
    }

    (render_objects, max_width)
}

#[inline]
fn continuation_to_prefix(continuations: &[bool]) -> String {
    continuations
        .iter()
        .map(
            |has_next_sibling| {
                if *has_next_sibling {
                    "│   "
                } else {
                    "    "
                }
            },
        )
        .collect()
}

#[derive(Debug)]
struct RenderObject {
    line: String,
    desc: Option<String>,
}

impl RenderObject {
    fn new(line: String, desc: Option<String>) -> Self {
        Self { line, desc }
    }
}

#[derive(Debug)]
enum RenderType {
    Dir,
    File,
    LastDir,
    LastFile,
}

impl RenderType {
    pub(crate) const fn as_str(&self) -> &'static str {
        match self {
            Self::Dir | Self::File => "├── ",
            Self::LastDir | Self::LastFile => "└── ",
        }
    }
}

#[inline]
fn calculate_is_last(entries: &[RepoEntry]) -> Vec<bool> {
    let mut is_last = vec![true; entries.len()];
    let mut last_at_depth: Vec<usize> = Vec::new();

    for (i, entry) in entries.iter().enumerate() {
        let depth = entry.path().components().count() - 1;

        if let Some(&previous) = last_at_depth.get(depth) {
            if entries[previous].path().parent() == entry.path().parent() {
                is_last[previous] = false;
            }
        }

        if last_at_depth.len() <= depth {
            last_at_depth.resize(depth + 1, i);
        } else {
            last_at_depth[depth] = i;
        }
    }
    is_last
}
