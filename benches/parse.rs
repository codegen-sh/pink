use std::{hint::black_box, path::PathBuf};

use codegen_sdk_analyzer::Codebase;
use criterion::{Criterion, criterion_group, criterion_main};
fn clone_repo(url: String, name: String, tmp_dir: &tempfile::TempDir) -> PathBuf {
    let repo_path = tmp_dir.path().join(name);
    if !repo_path.exists() {
        log::info!("Cloning repo: {} to {}", url, repo_path.display());
        let mut fetch_opts = git2::FetchOptions::new();
        fetch_opts.depth(1);
        let _ = git2::build::RepoBuilder::new()
            .fetch_options(fetch_opts)
            .clone(&url, &repo_path)
            .unwrap();
    }
    repo_path
}
fn parse_nest(path: &PathBuf) {
    let _ = Codebase::new(path.clone());
}

fn criterion_benchmark(c: &mut Criterion) {
    env_logger::init();
    let temp_dir = tempfile::tempdir().unwrap();
    let repo_path = clone_repo(
        "https://github.com/nestjs/nest".to_string(),
        "nest".to_string(),
        &temp_dir,
    );
    c.bench_function("parse_nest", |b| {
        b.iter(|| parse_nest(black_box(&repo_path)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
