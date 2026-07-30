use criterion::{criterion_group, criterion_main, Criterion};
use rayon::prelude::*;
use repo_mapper_rs::core::{
    adapters::FakeFileSystem, domain::utils::to_collection_of_type, main as repo_mapper_main,
};
use std::hint::black_box;
use std::{collections::HashMap, path::PathBuf};

fn criterion_benchmark(c: &mut Criterion) {
    let repo_root = "fake/repo/root".to_string();
    let readme_path = "fake/repo/root/README.md".to_string();
    let gitignore_path = "fake/repo/root/.gitignore".to_string();
    let allowed_exts: Vec<String> = to_collection_of_type(vec!["rs", "md", "toml"]);
    let ignore_dirs: Vec<String> = to_collection_of_type(vec![".venv", "target"]);
    let ignore_hidden: bool = true;
    let dirs_only: bool = true;
    let current_readme: &str = "# Some readme\n\n\n# Repo map\n```\n├── src\n│   ├── lib.rs\n│   └── main.rs\n├── Cargo.toml\n└── README.md\n::\n```";

    let files = vec![
        ("fake/repo/root/src/main.rs", "let x = 1;"),
        ("fake/repo/root/src/lib.rs", "use std;"),
        ("fake/repo/root/Cargo.toml", ""),
        ("fake/repo/root/README.md", current_readme),
        ("fake/repo/root/.gitignore", "target/"),
        ("fake/repo/root/target/some_build.rs", ""),
        ("fake/repo/root/.venv/site-packages/some_package.py", ""),
        ("fake/repo/root/scratch.py", ""),
        ("fake/repo/root/secrets/.env", ""),
    ]
    .into_par_iter()
    .with_min_len(1_000)
    .map(|(k, v)| (PathBuf::from(k), v.to_string()))
    .collect::<HashMap<PathBuf, String>>();

    let mut file_sys = FakeFileSystem::new(files);

    c.bench_function("repo_mapper_main", |b| {
        b.iter(|| {
            repo_mapper_main(
                black_box(&mut file_sys),
                black_box(repo_root.clone()),
                black_box(readme_path.clone()),
                black_box(gitignore_path.clone()),
                black_box(allowed_exts.clone()),
                black_box(ignore_dirs.clone()),
                black_box("readme"),
                black_box(ignore_hidden),
                black_box(dirs_only),
            )
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
