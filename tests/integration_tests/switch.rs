use crate::common::TestRepo;
use insta::Settings;
use insta_cmd::{assert_cmd_snapshot, get_cargo_bin};
use std::process::Command;

/// Helper to create snapshot with normalized paths and SHAs
fn snapshot_switch(test_name: &str, repo: &TestRepo, args: &[&str]) {
    let mut settings = Settings::clone_current();
    settings.set_snapshot_path("../snapshots");

    // Normalize paths - replace absolute paths with semantic names
    settings.add_filter(repo.root_path().to_str().unwrap(), "[REPO]");
    for (name, path) in &repo.worktrees {
        settings.add_filter(
            path.to_str().unwrap(),
            format!("[WORKTREE_{}]", name.to_uppercase().replace('-', "_")),
        );
    }

    // Normalize git SHAs (7-40 hex chars) to [SHA]
    settings.add_filter(r"\b[0-9a-f]{7,40}\b", "[SHA]");

    // Normalize Windows paths to Unix style
    settings.add_filter(r"\\", "/");

    settings.bind(|| {
        let mut cmd = Command::new(get_cargo_bin("wt"));
        repo.clean_cli_env(&mut cmd);
        cmd.arg("switch").args(args).current_dir(repo.root_path());

        assert_cmd_snapshot!(test_name, cmd);
    });
}

#[test]
fn test_switch_create_new_branch() {
    let repo = TestRepo::new();
    repo.commit("Initial commit");

    snapshot_switch("switch_create_new", &repo, &["--create", "feature-x"]);
}

#[test]
fn test_switch_create_existing_branch_error() {
    let mut repo = TestRepo::new();
    repo.commit("Initial commit");

    // Create a branch first
    repo.add_worktree("feature-y", "feature-y");

    // Try to create it again - should error
    snapshot_switch(
        "switch_create_existing_error",
        &repo,
        &["--create", "feature-y"],
    );
}

#[test]
fn test_switch_existing_branch() {
    let mut repo = TestRepo::new();
    repo.commit("Initial commit");

    // Create a worktree for a branch
    repo.add_worktree("feature-z", "feature-z");

    // Switch to it (should find existing worktree)
    snapshot_switch("switch_existing_branch", &repo, &["feature-z"]);
}

#[test]
fn test_switch_with_base_branch() {
    let repo = TestRepo::new();
    repo.commit("Initial commit on main");

    snapshot_switch(
        "switch_with_base",
        &repo,
        &["--create", "--base", "main", "feature-with-base"],
    );
}

#[test]
fn test_switch_base_without_create_warning() {
    let repo = TestRepo::new();
    repo.commit("Initial commit");

    snapshot_switch(
        "switch_base_without_create",
        &repo,
        &["--base", "main", "main"],
    );
}

#[test]
fn test_switch_internal_mode() {
    let repo = TestRepo::new();
    repo.commit("Initial commit");

    snapshot_switch(
        "switch_internal_mode",
        &repo,
        &["--create", "--internal", "internal-test"],
    );
}

#[test]
fn test_switch_existing_worktree_internal() {
    let mut repo = TestRepo::new();
    repo.commit("Initial commit");

    repo.add_worktree("existing-wt", "existing-wt");

    snapshot_switch(
        "switch_existing_internal",
        &repo,
        &["--internal", "existing-wt"],
    );
}
