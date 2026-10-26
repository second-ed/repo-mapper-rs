use criterion::Criterion;
use repo_mapper_rs::core::domain::render::generate_repo_map;

pub fn benchmark(c: &mut Criterion) {
    super::benchmark_renderer(c, "generate_repo_map", generate_repo_map);
}
