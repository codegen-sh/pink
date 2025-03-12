use std::{
    fmt::{self, Display},
    path::PathBuf,
};

use divan::AllocProfiler;
use rayon::prelude::*;
#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

use codegen_sdk_analyzer::Codebase;
// fn thread_counts() -> Vec<usize> {
//     vec![/* available parallelism */ 0, 1, 4, 8]
// }
const TMP_PATH: &str = "/tmp/pink-bench";

fn clone_repo(url: String, name: String) -> PathBuf {
    let repo_path = PathBuf::from(TMP_PATH).join(name);
    if !repo_path.exists() {
        log::info!("Cloning repo: {} to {}", url, repo_path.display());
        let mut fetch_opts = git2::FetchOptions::new();
        fetch_opts.depth(1);
        let _ = git2::build::RepoBuilder::new()
            .fetch_options(fetch_opts)
            .clone(&url, &repo_path)
            .unwrap();
        log::info!("Cloned repo: {} to {}", url, repo_path.display());
    }
    repo_path
}
struct Repo {
    pub name: &'static str,
    pub url: &'static str,
}
impl Display for Repo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
const REPOS: &[Repo] = &[
    // Typescript libraries
    Repo {
        name: "nest",
        url: "https://github.com/nestjs/nest",
    },
    // Repo {
    //     name: "react",
    //     url: "https://github.com/facebook/react",
    // },
    // Repo {
    //     name: "vscode",
    //     url: "https://github.com/microsoft/vscode",
    // },
    // Python libraries
    // Repo {
    //     name: "pytorch",
    //     url: "https://github.com/pytorch/pytorch",
    // },
    Repo {
        name: "mypy",
        url: "https://github.com/python/mypy",
    },
    // Repo {
    //     name: "django",
    //     url: "https://github.com/django/django",
    // },
    // Go libraries
    Repo {
        name: "gin",
        url: "https://github.com/gin-gonic/gin",
    },
    // Repo {
    //     name: "kubernetes",
    //     url: "https://github.com/kubernetes/kubernetes",
    // },
    // Rust libraries
    Repo {
        name: "tokio",
        url: "https://github.com/tokio-rs/tokio",
    },
    Repo {
        name: "rust-analyzer",
        url: "https://github.com/rust-lang/rust-analyzer",
    },
];
const fn repo_indices() -> [usize; REPOS.len()] {
    [0, 1, 2, 3, 4, 5, 6]
}
#[divan::bench(consts = repo_indices())]
fn parse<const REPO: usize>(bencher: divan::Bencher) {
    let repo = &REPOS[REPO];
    log::info!("Parsing repo: {}", repo.name);
    let repo_path = clone_repo(repo.url.to_string(), repo.name.to_string());
    bencher.bench(|| Codebase::new(repo_path.clone()));
}

fn main() {
    env_logger::init();
    REPOS.par_iter().for_each(|repo| {
        clone_repo(repo.url.to_string(), repo.name.to_string());
    });
    // Run registered benchmarks.
    divan::main();
}
