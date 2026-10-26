// repo-map-desc: basic data creation for benches
use criterion::{BatchSize, Criterion};
use regex::Regex;
use repo_mapper_rs::core::{
    adapters::{FakeFileSystem, FileSystem},
    domain::{repo_entry::RepoEntry, transform::pathbufs_to_repo_entries},
};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

pub mod render;

pub fn benchmark_renderer(c: &mut Criterion, name: &str, render: fn(Vec<RepoEntry>) -> String) {
    let mut group = c.benchmark_group(name);

    for file_count in [10, 100, 1_000] {
        let entries = repo_entries(file_count);
        group.bench_function(format!("{file_count}_files"), |b| {
            b.iter_batched(
                || entries.clone(),
                |entries| std::hint::black_box(render(entries)),
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn repo_entries(file_count: usize) -> Vec<RepoEntry> {
    let root = PathBuf::from("fake/repo/root");
    let files = (0..file_count)
        .map(|index| {
            let path = root
                .join("src")
                .join(format!("module_{}", index % 10))
                .join(format!("file_{index}.rs"));
            let contents = if index % 10 == 0 {
                "// repo-map-desc: benchmark file"
            } else {
                "fn benchmark_fixture() {}"
            };
            (path, contents.to_string())
        })
        .collect::<HashMap<_, _>>();

    let mut file_system = FakeFileSystem::new(files);
    let paths = file_system.list_files(&root, &HashSet::new(), false);

    pathbufs_to_repo_entries(
        &mut file_system,
        paths,
        &root,
        &HashSet::new(),
        &HashSet::new(),
        &[] as &[Regex],
        false,
        false,
    )
}
