//! Output handlers for worktree operations using the global output context

use crate::commands::worktree::{RemoveResult, SwitchResult};
use worktrunk::git::{GitError, GitResultExt};
use worktrunk::styling::AnstyleStyle;

/// Format plain message for switch operation (no emoji - added by OutputContext)
fn format_switch_message_plain(result: &SwitchResult, branch: &str) -> String {
    let bold = AnstyleStyle::new().bold();

    match result {
        SwitchResult::ExistingWorktree(path) => {
            format!(
                "Switched to worktree for {bold}{branch}{bold:#} at {}",
                path.display()
            )
        }
        SwitchResult::CreatedWorktree {
            path,
            created_branch,
        } => {
            if *created_branch {
                format!(
                    "Created new worktree for {bold}{branch}{bold:#} at {}",
                    path.display()
                )
            } else {
                format!(
                    "Added worktree for {bold}{branch}{bold:#} at {}",
                    path.display()
                )
            }
        }
    }
}

/// Format plain message for remove operation (no emoji - added by OutputContext)
fn format_remove_message_plain(result: &RemoveResult) -> String {
    let bold = AnstyleStyle::new().bold();

    match result {
        RemoveResult::AlreadyOnDefault(branch) => {
            format!("Already on default branch {bold}{branch}{bold:#}")
        }
        RemoveResult::RemovedWorktree { primary_path } => {
            format!(
                "Removed worktree, returned to primary at {}",
                primary_path.display()
            )
        }
        RemoveResult::SwitchedToDefault(branch) => {
            format!("Switched to default branch {bold}{branch}{bold:#}")
        }
        RemoveResult::RemovedOtherWorktree { branch } => {
            format!("Removed worktree for {bold}{branch}{bold:#}")
        }
    }
}

/// Shell integration hint message
fn shell_integration_hint() -> &'static str {
    "To enable automatic cd, run: wt configure-shell"
}

/// Handle output for a switch operation
pub fn handle_switch_output(
    result: &SwitchResult,
    branch: &str,
    execute: Option<&str>,
) -> Result<(), GitError> {
    // Set target directory for command execution
    super::change_directory(result.path())?;

    // Show success message (plain text - formatting added by OutputContext)
    super::success(format_switch_message_plain(result, branch))?;

    // Execute command if provided
    if let Some(cmd) = execute {
        super::execute(cmd)?;
    } else {
        // No execute command: show shell integration hint
        // (suppressed in directive mode since user already has integration)
        super::hint(format!("\n{}", shell_integration_hint()))?;
    }

    // Flush output (important for directive mode)
    super::flush()?;

    Ok(())
}

/// Handle output for a remove operation
pub fn handle_remove_output(result: &RemoveResult) -> Result<(), GitError> {
    // For removed worktree, set target directory for shell to cd to
    if let RemoveResult::RemovedWorktree { primary_path } = result {
        super::change_directory(primary_path)?;
    }

    // Show success message
    super::success(format_remove_message_plain(result))?;

    // Flush output
    super::flush()?;

    Ok(())
}

/// Execute a command in a worktree directory
///
/// Merges stdout into stderr using shell redirection (1>&2) to ensure deterministic output ordering.
/// Per CLAUDE.md guidelines: child process output goes to stderr, worktrunk output goes to stdout.
/// Streams output line-by-line in real-time (no buffering) to provide immediate feedback for
/// long-running commands.
///
/// The shell-level redirect ensures all output flows through a single pipe (stderr) in the order written,
/// eliminating race conditions that would occur with separate stdout/stderr threads.
///
/// Calls terminate_output() after completion to handle mode-specific cleanup
/// (NUL terminator in directive mode, no-op in interactive mode).
pub fn execute_command_in_worktree(
    worktree_path: &std::path::Path,
    command: &str,
) -> Result<(), GitError> {
    use std::io::{BufRead, BufReader, Write};
    use std::process::{Command, Stdio};

    // Flush stdout before executing command to ensure all our messages appear
    // before the child process output
    super::flush()?;

    // Redirect stdout to stderr in the shell command to merge streams
    // This ensures deterministic ordering: all output flows through a single pipe (stderr)
    // in the order it was written, with no race conditions between threads
    // Per CLAUDE.md: child process output goes to stderr, worktrunk output goes to stdout
    let merged_command = format!("{{ {}; }} 1>&2", command);

    let mut child = Command::new("sh")
        .arg("-c")
        .arg(&merged_command)
        .current_dir(worktree_path)
        .stdout(Stdio::inherit()) // Inherit stdout for any shell errors (though redirected to stderr)
        .stderr(Stdio::piped())
        .spawn()
        .git_context("Failed to execute command")?;

    // Read and stream output line-by-line in real-time (no buffering)
    let stderr = child.stderr.take().unwrap();
    let reader = BufReader::new(stderr);

    for line in reader.lines().map_while(Result::ok) {
        eprintln!("{}", line);
        let _ = std::io::stderr().flush();
    }

    // Wait for command to complete
    let status = child.wait().git_context("Failed to wait for command")?;

    if !status.success() {
        return Err(GitError::CommandFailed(format!(
            "Command failed with exit code: {}",
            status
        )));
    }

    // Flush to ensure all output appears before we continue
    super::flush()?;

    // Terminate output (adds NUL in directive mode, no-op in interactive)
    super::terminate_output()?;

    Ok(())
}
