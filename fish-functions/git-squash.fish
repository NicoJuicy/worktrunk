function git-squash -d "Squash all commits since branching from specified branch (default: main)"
    argparse -n git-squash 'r/rebase' 'A/all' -- $argv
    or return 1

    # Get target branch - use argument or default branch
    set -l target_branch
    if test (count $argv) -gt 0
        set target_branch $argv[1]
    else
        set target_branch (git default-branch)
        if test -z "$target_branch"
            echo (set_color red)"❌ Failed to determine default branch"(set_color normal) >&2
            return 1
        end
    end

    # Get merge base
    set -l merge_base (git merge-base HEAD "$target_branch")
    if test -z "$merge_base"
        echo (set_color red)"❌ Failed to find merge base with "(set_color red --bold)"$target_branch"(set_color normal) >&2
        return 1
    end

    # Get current branch name for context
    set -l current_branch (git rev-parse --abbrev-ref HEAD)
    if test -z "$current_branch"
        echo (set_color red)"❌ Failed to determine current branch"(set_color normal) >&2
        return 1
    end

    # Check if there are any commits to squash
    set -l commit_count (git rev-list --count "$merge_base..HEAD")

    # Check if we have staged changes
    set -l has_staged_changes 0
    if not git diff --cached --quiet --exit-code
        set has_staged_changes 1
    end

    # Check if we have unstaged changes
    set -l has_unstaged_changes 0
    if not git diff --quiet --exit-code
        set has_unstaged_changes 1
    end

    # Handle -A flag to add all changes
    if set -ql _flag_all; and test "$has_unstaged_changes" -eq 1
        echo (set_color yellow)"🟡 Adding all changes (including unstaged)"(set_color normal) >&2
        git add -A
        or begin
            echo (set_color red)"❌ Failed to add all changes"(set_color normal) >&2
            return 1
        end
        # After adding, everything is staged
        set has_staged_changes 1
        set has_unstaged_changes 0
    end

    # Handle different squashing scenarios
    if test "$commit_count" -eq 0; and test "$has_staged_changes" -eq 0
        # No commits and no staged changes - nothing to squash (but might still rebase)
        echo (set_color -d)"💬 No commits to squash - already at merge base"(set_color normal) >&2

    else if test "$commit_count" -eq 0; and test "$has_staged_changes" -eq 1
        # Just staged changes, no commits - delegate to git-commit-llm
        git-commit-llm
        or return 1

    else if test "$commit_count" -eq 1; and test "$has_staged_changes" -eq 0
        # Single commit, no staged changes - nothing to do
        echo (set_color -d)"💬 Only 1 commit since "(set_color -d --bold)"$target_branch"(set_color normal -d)" - no squashing needed"(set_color normal) >&2

    else
        # One or more commits (possibly with staged changes) - squash them all together
        if test "$commit_count" -eq 1; and test "$has_staged_changes" -eq 1
            echo (set_color cyan)"🔄 Amending staged changes into the existing commit"(set_color normal) >&2
        end

        # Get commit context before resetting
        set -l commit_context (__git_build_commit_context "$merge_base..HEAD")

        # Reset to the merge base (this stages all the changes)
        git reset --soft $merge_base
        or begin
            echo (set_color red)"❌ Failed to reset to merge base"(set_color normal) >&2
            return 1
        end

        # Generate commit message from the staged changes
        set -l commit_message
        printf "Squashing commits on branch '%s' since %s\n\n%s" "$current_branch" "$target_branch" "$commit_context" | \
            git-llm-message "Generate a commit message that combines these changes into one cohesive message." | \
            read -z commit_message
        or begin
            echo (set_color red)"❌ Failed to generate commit message"(set_color normal) >&2
            return 1
        end

        # Commit with the generated message
        printf "%s\n\nCo-authored-by: Claude <no-reply@anthropic.com>\n" "$commit_message" | git commit -F -
        or begin
            echo (set_color red)"❌ Failed to create commit"(set_color normal) >&2
            return 1
        end

        if test "$commit_count" -ne 1; or test "$has_staged_changes" -ne 1
            echo (set_color green)"✅ Successfully squashed $commit_count commit(s) into one"(set_color normal) >&2
        end
    end

    # Rebase if requested (happens for both single and multiple commits)
    if set -ql _flag_rebase
        # Check if we're already up to date with target branch
        if git merge-base --is-ancestor "$target_branch" HEAD
            echo (set_color -d)"💬 Already up to date with "(set_color -d --bold)"$target_branch"(set_color normal -d)" "(set_color normal) >&2
        else
            echo (set_color cyan)"🔄 Rebasing onto "(set_color cyan --bold)"$target_branch"(set_color normal)(set_color cyan)"..."(set_color normal) >&2
            if not git rebase $target_branch
                echo (set_color red)"❌ Failed to rebase onto "(set_color red --bold)"$target_branch"(set_color normal) >&2
                return 1
            end
        end
    end
end
