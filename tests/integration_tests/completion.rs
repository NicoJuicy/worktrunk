use crate::common::TestRepo;
use assert_cmd::Command;
use std::process::Command as StdCommand;

#[test]
fn test_completion_command_static() {
    // Test that static completion generates valid Fish script
    let mut cmd = Command::cargo_bin("wt").unwrap();
    let output = cmd.arg("completion").arg("fish").output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check for expected Fish completion functions
    assert!(stdout.contains("__fish_wt_needs_command"));
    assert!(stdout.contains("complete -c wt"));
    assert!(stdout.contains("switch"));
    assert!(stdout.contains("finish"));
    assert!(stdout.contains("push"));
    assert!(stdout.contains("merge"));
}

#[test]
fn test_completion_command_bash() {
    let mut cmd = Command::cargo_bin("wt").unwrap();
    let output = cmd.arg("completion").arg("bash").output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check for Bash completion structure
    assert!(stdout.contains("_wt()"));
    assert!(stdout.contains("COMPREPLY"));
    assert!(stdout.contains("complete"));
}

#[test]
fn test_complete_switch_shows_branches() {
    let temp = TestRepo::new();
    temp.commit("initial");

    // Create some branches using git
    StdCommand::new("git")
        .args(&["branch", "feature/new"])
        .current_dir(temp.root_path())
        .output()
        .unwrap();

    StdCommand::new("git")
        .args(&["branch", "hotfix/bug"])
        .current_dir(temp.root_path())
        .output()
        .unwrap();

    // Test completion for switch command
    let mut cmd = Command::cargo_bin("wt").unwrap();
    let output = cmd
        .current_dir(temp.root_path())
        .args(&["complete", "wt", "switch", ""])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let branches: Vec<&str> = stdout.lines().collect();

    // Should include both branches (no worktrees created yet)
    assert!(branches.iter().any(|b| b.contains("feature/new")));
    assert!(branches.iter().any(|b| b.contains("hotfix/bug")));
}

#[test]
fn test_complete_switch_excludes_branches_with_worktrees() {
    let mut temp = TestRepo::new();
    temp.commit("initial");

    // Create worktree (this creates a new branch "feature/new")
    temp.add_worktree("feature-worktree", "feature/new");

    // Create another branch without worktree
    StdCommand::new("git")
        .args(&["branch", "hotfix/bug"])
        .current_dir(temp.root_path())
        .output()
        .unwrap();

    // Test completion
    let mut cmd = Command::cargo_bin("wt").unwrap();
    let output = cmd
        .current_dir(temp.root_path())
        .args(&["complete", "wt", "switch", ""])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let branches: Vec<&str> = stdout.lines().collect();

    // Should NOT include feature/new (has worktree)
    assert!(!branches.iter().any(|b| b.contains("feature/new")));
    // Should include hotfix/bug (no worktree)
    assert!(branches.iter().any(|b| b.contains("hotfix/bug")));
}

#[test]
fn test_complete_push_shows_all_branches() {
    let mut temp = TestRepo::new();
    temp.commit("initial");

    // Create worktree (creates "feature/new" branch)
    temp.add_worktree("feature-worktree", "feature/new");

    // Create another branch without worktree
    StdCommand::new("git")
        .args(&["branch", "hotfix/bug"])
        .current_dir(temp.root_path())
        .output()
        .unwrap();

    // Test completion for push (should show ALL branches, including those with worktrees)
    let mut cmd = Command::cargo_bin("wt").unwrap();
    let output = cmd
        .current_dir(temp.root_path())
        .args(&["complete", "wt", "push", ""])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let branches: Vec<&str> = stdout.lines().collect();

    // Should include both branches (push shows all)
    assert!(branches.iter().any(|b| b.contains("feature/new")));
    assert!(branches.iter().any(|b| b.contains("hotfix/bug")));
}

#[test]
fn test_complete_base_flag_shows_all_branches() {
    let temp = TestRepo::new();
    temp.commit("initial");

    // Create branches
    StdCommand::new("git")
        .args(&["branch", "develop"])
        .current_dir(temp.root_path())
        .output()
        .unwrap();

    StdCommand::new("git")
        .args(&["branch", "feature/existing"])
        .current_dir(temp.root_path())
        .output()
        .unwrap();

    // Test completion for --base flag
    let mut cmd = Command::cargo_bin("wt").unwrap();
    let output = cmd
        .current_dir(temp.root_path())
        .args(&[
            "complete",
            "wt",
            "switch",
            "--create",
            "new-branch",
            "--base",
            "",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let branches: Vec<&str> = stdout.lines().collect();

    // Should show all branches as potential base
    assert!(branches.iter().any(|b| b.contains("develop")));
    assert!(branches.iter().any(|b| b.contains("feature/existing")));
}

#[test]
fn test_complete_outside_git_repo_returns_empty() {
    let temp = tempfile::tempdir().unwrap();

    // Test completion outside a git repo
    let mut cmd = Command::cargo_bin("wt").unwrap();
    let output = cmd
        .current_dir(temp.path())
        .args(&["complete", "wt", "switch", ""])
        .output()
        .unwrap();

    // Should succeed but return no branches
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "");
}

#[test]
fn test_init_fish_includes_no_file_flag() {
    // Test that fish init includes -f flag to disable file completion
    let mut cmd = Command::cargo_bin("wt").unwrap();
    let output = cmd.arg("init").arg("fish").output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check that completions include -f flag
    assert!(stdout.contains("-f -a '(__wt_complete)'"));
}

#[test]
fn test_completion_command_zsh() {
    let mut cmd = Command::cargo_bin("wt").unwrap();
    let output = cmd.arg("completion").arg("zsh").output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check for Zsh completion structure
    assert!(stdout.contains("_wt"));
    assert!(stdout.contains("compdef"));
}

#[test]
fn test_complete_base_flag_short_form() {
    let temp = TestRepo::new();
    temp.commit("initial");

    // Create branches
    StdCommand::new("git")
        .args(&["branch", "develop"])
        .current_dir(temp.root_path())
        .output()
        .unwrap();

    // Test completion for -b flag (short form of --base)
    let mut cmd = Command::cargo_bin("wt").unwrap();
    let output = cmd
        .current_dir(temp.root_path())
        .args(&[
            "complete",
            "wt",
            "switch",
            "--create",
            "new-branch",
            "-b",
            "",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let branches: Vec<&str> = stdout.lines().collect();

    // Should show all branches as potential base
    assert!(branches.iter().any(|b| b.contains("develop")));
}

#[test]
fn test_complete_empty_repo_returns_empty() {
    let temp = TestRepo::new();
    // No commits, no branches

    // Test completion in empty repo
    let mut cmd = Command::cargo_bin("wt").unwrap();
    let output = cmd
        .current_dir(temp.root_path())
        .args(&["complete", "wt", "switch", ""])
        .output()
        .unwrap();

    // Should succeed but return no branches
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "");
}

#[test]
fn test_complete_with_partial_prefix() {
    let temp = TestRepo::new();
    temp.commit("initial");

    // Create branches with common prefix
    StdCommand::new("git")
        .args(&["branch", "feature/one"])
        .current_dir(temp.root_path())
        .output()
        .unwrap();

    StdCommand::new("git")
        .args(&["branch", "feature/two"])
        .current_dir(temp.root_path())
        .output()
        .unwrap();

    StdCommand::new("git")
        .args(&["branch", "hotfix/bug"])
        .current_dir(temp.root_path())
        .output()
        .unwrap();

    // Complete with partial prefix - should return all branches
    // (shell completion framework handles the prefix filtering)
    let mut cmd = Command::cargo_bin("wt").unwrap();
    let output = cmd
        .current_dir(temp.root_path())
        .args(&["complete", "wt", "switch", "feat"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should return all branches, not just those matching "feat"
    // The shell will filter based on user input
    assert!(stdout.contains("feature/one"));
    assert!(stdout.contains("feature/two"));
    assert!(stdout.contains("hotfix/bug"));
}

#[test]
fn test_complete_switch_no_available_branches() {
    let mut temp = TestRepo::new();
    temp.commit("initial");

    // Create two branches, both with worktrees - this should result in no available branches
    temp.add_worktree("feature-worktree", "feature/new");
    temp.add_worktree("hotfix-worktree", "hotfix/bug");

    // From the main worktree, test completion - should return empty since both non-main branches have worktrees
    // and we're currently in the main worktree
    let mut cmd = Command::cargo_bin("wt").unwrap();
    let output = cmd
        .current_dir(temp.root_path())
        .args(&["complete", "wt", "switch", ""])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should not include branches that have worktrees
    assert!(!stdout.contains("feature/new"));
    assert!(!stdout.contains("hotfix/bug"));
    // May include main if it doesn't have a worktree
}
