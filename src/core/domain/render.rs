use crate::core::domain::repo_entry::RepoEntry;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use std::{cmp::Ordering, path::Component};
use unicode_width::UnicodeWidthStr;

pub(crate) fn generate_repo_map(mut repo_entries: Vec<RepoEntry>) -> String {
    repo_entries.sort_unstable_by(cmp_entries);

    let render_objects = repo_entries_to_render_entries(&repo_entries)
        .into_par_iter()
        .with_min_len(1_000)
        .map(render_entry_to_render_object)
        .collect::<Vec<RenderObject>>();

    render_repo_map(render_objects)
}

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
fn repo_entries_to_render_entries(repo_entries: &[RepoEntry]) -> Vec<RenderEntry> {
    let is_last = calculate_is_last(repo_entries);

    let mut result = Vec::with_capacity(repo_entries.len());
    let mut ancestors: Vec<bool> = Vec::new();

    for (i, entry) in repo_entries.iter().enumerate() {
        let path = entry.path();
        let depth = path.components().count() - 1;
        ancestors.truncate(depth);

        let continuations = ancestors
            .iter()
            .enumerate()
            .fold(0u64, |bits, (level, &ancestor_is_last)| {
                bits | (u64::from(!ancestor_is_last) << level)
            });

        let entry_is_last = is_last[i];

        let info = RenderInfo {
            name: path.file_name().unwrap().to_string_lossy().into_owned(),
            desc: entry.desc().map(str::to_owned),
            continuations,
        };

        result.push(match (entry.is_dir(), entry_is_last) {
            (true, true) => RenderEntry::LastDir(info),
            (true, false) => RenderEntry::Dir(info),
            (false, true) => RenderEntry::LastFile(info),
            (false, false) => RenderEntry::File(info),
        });

        ancestors.push(entry_is_last);
    }

    result
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone)]
struct RenderInfo {
    name: String,
    desc: Option<String>,
    continuations: u64,
}

#[derive(Debug)]
enum RenderEntry {
    Dir(RenderInfo),
    File(RenderInfo),
    LastDir(RenderInfo),
    LastFile(RenderInfo),
}

#[derive(Debug)]
struct RenderObject {
    line: String,
    desc: Option<String>,
}

#[inline]
fn render_entry_to_render_object(entry: RenderEntry) -> RenderObject {
    let (name, desc, continuations, connector) = match entry {
        RenderEntry::Dir(info) | RenderEntry::File(info) => {
            (info.name, info.desc, info.continuations, "├── ")
        }
        RenderEntry::LastDir(info) | RenderEntry::LastFile(info) => {
            (info.name, info.desc, info.continuations, "└── ")
        }
    };

    let depth = if continuations == 0 {
        0
    } else {
        continuations.ilog2() + 1
    };

    let prefix: String = (0..depth)
        .map(|level| {
            if continuations & (1 << level) != 0 {
                "│   "
            } else {
                "    "
            }
        })
        .collect();

    RenderObject {
        line: format!("{prefix}{connector}{name}"),
        desc,
    }
}

#[inline]
fn calculate_is_last(entries: &[RepoEntry]) -> Vec<bool> {
    let mut result = vec![true; entries.len()];
    let mut next_sibling_at_depth = Vec::<usize>::new();

    for i in (0..entries.len()).rev() {
        let path = entries[i].path();
        let depth = path.components().count() - 1;

        if let Some(&next) = next_sibling_at_depth.get(depth) {
            result[i] = entries[next].path().parent() != path.parent();
        }

        if next_sibling_at_depth.len() <= depth {
            next_sibling_at_depth.resize(depth + 1, i);
        } else {
            next_sibling_at_depth[depth] = i;
        }
    }

    result
}

#[inline]
fn render_repo_map(objects: Vec<RenderObject>) -> String {
    let max_width = objects
        .iter()
        .map(|object| UnicodeWidthStr::width(object.line.as_str()))
        .max()
        .unwrap_or(40);

    let mut lines = objects
        .into_par_iter()
        .with_min_len(1_000)
        .map(|object| {
            let RenderObject { line, desc } = object;

            match desc {
                Some(desc) => {
                    let width = UnicodeWidthStr::width(line.as_str());
                    let padding = max_width - width;

                    format!("{line}{}  # {desc}", " ".repeat(padding))
                }
                None => line,
            }
        })
        .collect::<Vec<_>>();

    lines.push("\n(generated with repo-mapper-rs)".to_string());
    format!("# Repo map\n```\n{}\n::\n```", lines.join("\n"))
}
