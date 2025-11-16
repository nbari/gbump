use git2::{Commit, ObjectType, Repository, Signature};
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};
use tempfile::TempDir;

#[test]
fn cli_prints_current_and_next_version() {
    let fixture = RepoFixture::with_commit();
    add_tag(fixture.repo(), "0.1.1");
    let output = run_gbump(fixture.path(), &[]);
    assert!(
        output.status.success(),
        "gbump should succeed: stderr={}",
        stderr(&output)
    );
    assert_eq!(stdout(&output), "0.1.1 --> 0.1.2");
}

#[test]
fn cli_supports_quiet_mode() {
    let fixture = RepoFixture::with_commit();
    add_tag(fixture.repo(), "2.4.5");
    let output = run_gbump(fixture.path(), &["-q", "minor"]);
    assert!(
        output.status.success(),
        "gbump should succeed: stderr={}",
        stderr(&output)
    );
    assert_eq!(stdout(&output), "2.5.0");
}

#[test]
fn cli_creates_tag_when_flag_is_set() {
    let fixture = RepoFixture::with_commit();
    let output = run_gbump(fixture.path(), &["-t", "patch"]);
    assert!(
        output.status.success(),
        "gbump should succeed: stderr={}",
        stderr(&output)
    );
    let stdout_text = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout_text.contains("0.0.0 --> 0.0.1"),
        "stdout missing semver output: {stdout_text}"
    );
    assert!(
        stdout_text.contains("Tag: 0.0.1 created:"),
        "stdout missing tag output: {stdout_text}"
    );
    let repo = Repository::open(fixture.path()).unwrap();
    let tags = repo.tag_names(None).unwrap();
    assert!(
        tags.iter().flatten().any(|name| name == "0.0.1"),
        "expected tag 0.0.1 to exist"
    );
}

#[test]
fn cli_errors_outside_repository() {
    let dir = TempDir::new().unwrap();
    let output = run_gbump(dir.path(), &[]);
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("Not in a git repository"),
        "unexpected stderr: {}",
        stderr(&output)
    );
}

#[test]
fn cli_tagging_without_commit_fails() {
    let fixture = RepoFixture::without_commit();
    let output = run_gbump(fixture.path(), &["-t"]);
    assert!(!output.status.success(), "gbump should fail without HEAD");
    assert!(
        stderr(&output).contains("Could not create tag"),
        "unexpected stderr: {}",
        stderr(&output)
    );
}

#[test]
fn cli_errors_when_tags_are_forced_to_fail() {
    let fixture = RepoFixture::with_commit();
    let output = run_gbump_with(fixture.path(), &[], |cmd| {
        cmd.env("GBUMP_FORCE_TAG_FAILURE", "1");
    });
    assert!(
        !output.status.success(),
        "gbump should fail when tag listing errors"
    );
    assert!(
        stderr(&output).contains("Could not get tags from repo"),
        "unexpected stderr: {}",
        stderr(&output)
    );
}

#[test]
fn cli_tagging_without_identity_fails() {
    let fixture = RepoFixture::with_commit_without_identity();
    let fake_home = fixture.path().join("home");
    std::fs::create_dir_all(&fake_home).unwrap();
    let output = run_gbump_with(fixture.path(), &["-t"], |cmd| {
        cmd.env("GIT_CONFIG_NOSYSTEM", "1");
        cmd.env("HOME", &fake_home);
        for var in [
            "GIT_AUTHOR_NAME",
            "GIT_AUTHOR_EMAIL",
            "GIT_COMMITTER_NAME",
            "GIT_COMMITTER_EMAIL",
        ] {
            cmd.env_remove(var);
        }
    });
    assert!(
        !output.status.success(),
        "gbump should fail when identity is missing"
    );
    assert!(
        stderr(&output).contains("Could not create tag"),
        "unexpected stderr: {}",
        stderr(&output)
    );
}

fn run_gbump(dir: &Path, args: &[&str]) -> Output {
    run_gbump_with(dir, args, |_| {})
}

fn run_gbump_with(dir: &Path, args: &[&str], configure: impl FnOnce(&mut Command)) -> Output {
    let mut cmd = Command::new(bin_path());
    cmd.current_dir(dir);
    cmd.env("CARGO_TERM_COLOR", "never");
    cmd.args(args);
    configure(&mut cmd);
    cmd.output().expect("run gbump")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).trim().to_string()
}

fn bin_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_gbump"))
}

struct RepoFixture {
    dir: TempDir,
    repo: Repository,
}

impl RepoFixture {
    fn with_commit() -> Self {
        Self::new(true, true)
    }

    fn with_commit_without_identity() -> Self {
        Self::new(false, true)
    }

    fn without_commit() -> Self {
        Self::new(true, false)
    }

    fn new(set_identity: bool, with_commit: bool) -> Self {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        if set_identity {
            configure_identity(&repo);
        }
        if with_commit {
            seed_commit(&repo);
        }
        Self { dir, repo }
    }

    fn path(&self) -> &Path {
        self.dir.path()
    }

    const fn repo(&self) -> &Repository {
        &self.repo
    }
}

fn add_tag(repo: &Repository, name: &str) {
    let head = repo.head().unwrap();
    let commit = head.peel(ObjectType::Commit).unwrap();
    let sig = repo.signature().unwrap();
    repo.tag(name, &commit, &sig, name, false).unwrap();
}

fn seed_commit(repo: &Repository) {
    let workdir = repo.workdir().expect("repo should have workdir");
    std::fs::write(workdir.join("README.md"), "initial").unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(Path::new("README.md")).unwrap();
    index.write().unwrap();
    write_commit(repo, "initial");
}

fn write_commit(repo: &Repository, message: &str) {
    let mut index = repo.index().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = Signature::now("Tester", "tester@example.com").unwrap();
    let parents = repo.head().map_or_else(
        |_| Vec::new(),
        |head| vec![repo.find_commit(head.target().unwrap()).unwrap()],
    );
    let parent_refs: Vec<&Commit> = parents.iter().collect();
    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parent_refs)
        .unwrap();
}

fn configure_identity(repo: &Repository) {
    let mut config = repo.config().unwrap();
    config.set_str("user.name", "Tester").unwrap();
    config.set_str("user.email", "tester@example.com").unwrap();
}
