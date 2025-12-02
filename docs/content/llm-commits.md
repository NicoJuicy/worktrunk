+++
title = "LLM Commit Messages"
weight = 5
+++

Worktrunk generates commit messages by building a templated prompt and piping it to an external command. This integrates with `wt merge` and `wt step commit`.

## Setup

### Install llm

[llm](https://llm.datasette.io/) from Simon Willison is recommended:

```bash
$ uv tool install -U llm
```

### Configure an API key

For Claude (recommended):

```bash
$ llm install llm-anthropic
$ llm keys set anthropic
```

For OpenAI:

```bash
$ llm keys set openai
```

### Add to user config

Create the config file if it doesn't exist:

```bash
$ wt config create
```

Then add the commit generation settings to `~/.config/worktrunk/config.toml`:

```toml
[commit-generation]
command = "llm"
args = ["-m", "claude-haiku-4.5"]
```

Or for OpenAI:

```toml
[commit-generation]
command = "llm"
args = ["-m", "gpt-4o-mini"]
```

## Two generation modes

Worktrunk has two LLM generation functions, each with its own template:

| Mode | Analyzes | Template | Used by |
|------|----------|----------|---------|
| **Commit** | Git diff (staged changes) | `template` | `wt step commit`, `wt merge --no-squash` (when dirty) |
| **Squash** | Commit subjects (message titles) | `squash-template` | `wt step squash`, `wt merge` (default) |

The key difference: commit mode sees *what changed* (diffs), squash mode sees *what was already said* (commit messages).

## Usage

### wt merge (default behavior)

By default, `wt merge` squashes all commits on the branch into one. Any uncommitted changes are staged and included in the squash. The commit message is generated using **squash mode** (analyzing commit subjects, not diffs):

```bash
$ wt merge
🔄 Squashing 3 commits into a single commit (5 files, +48)...
🔄 Generating squash commit message...
   feat(auth): Implement JWT authentication system
   ...
```

### wt merge --no-squash

<!-- TODO: This example only shows committing uncommitted changes. Add an example
     that shows multiple preserved commits being rebased onto the target branch,
     which is the main point of --no-squash. -->

With `--no-squash`, individual commits are preserved and rebased onto the target branch. If there are uncommitted changes, they're committed first using **commit mode**:

```bash
$ wt merge --no-squash
🔄 Generating commit message and committing changes... (2 files, +15)
   fix: Handle edge case in config parser
✅ Committed changes @ a1b2c3d
🔄 Rebasing onto main...
```

If there are no uncommitted changes, the commits are rebased as-is without generating any new messages.

### wt step commit

Commits staged changes using **commit mode**:

```bash
$ wt step commit
```

### wt step squash

Squashes commits on the current branch using **squash mode**:

```bash
$ wt step squash
```

## Prompt templates

Worktrunk uses [minijinja](https://docs.rs/minijinja/) templates (Jinja2-like syntax) to build prompts. There are sensible defaults, but templates are fully customizable.

### Template variables

**Commit template** (analyzes diffs):

| Variable | Description |
|----------|-------------|
| `{{ git_diff }}` | The staged diff |
| `{{ branch }}` | Current branch name |
| `{{ recent_commits }}` | List of recent commit subjects (for style reference) |
| `{{ repo }}` | Repository name |

**Squash template** (analyzes commit messages):

| Variable | Description |
|----------|-------------|
| `{{ commits }}` | List of commit messages being squashed (chronological order) |
| `{{ target_branch }}` | Branch being merged into |
| `{{ branch }}` | Current branch name |
| `{{ repo }}` | Repository name |

### Custom templates

Override the defaults with inline templates or external files:

```toml
[commit-generation]
command = "llm"
args = ["-m", "claude-haiku-4.5"]

# Commit mode template (analyzes diffs)
template = """
Write a commit message for this diff. One line, under 50 chars.

Branch: {{ branch }}
Diff:
{{ git_diff }}
"""

# Squash mode template (analyzes commit messages)
squash-template = """
Combine these {{ commits | length }} commits into one message:
{% for c in commits %}
- {{ c }}
{% endfor %}
"""
```

Or load templates from files (supports `~` expansion):

```toml
[commit-generation]
command = "llm"
args = ["-m", "claude-haiku-4.5"]

# Load from files instead
template-file = "~/.config/worktrunk/commit-template.txt"
squash-template-file = "~/.config/worktrunk/squash-template.txt"
```

### Template syntax

Templates use [minijinja](https://docs.rs/minijinja/latest/minijinja/syntax/index.html), which supports:

- **Variables**: `{{ branch }}`, `{{ repo | upper }}`
- **Filters**: `{{ commits | length }}`, `{{ repo | upper }}`
- **Conditionals**: `{% if recent_commits %}...{% endif %}`
- **Loops**: `{% for c in commits %}{{ c }}{% endfor %}`
- **Loop variables**: `{{ loop.index }}`, `{{ loop.length }}`
- **Whitespace control**: `{%- ... -%}` strips surrounding whitespace

See `wt config create --help` for the full default templates.

## Alternative tools

Any command that reads a prompt from stdin and outputs a commit message works:

```toml
# aichat
[commit-generation]
command = "aichat"
args = ["-m", "claude:claude-3-5-haiku-latest"]

# Custom script
[commit-generation]
command = "./scripts/generate-commit.sh"
```

## Fallback behavior

When no LLM is configured, worktrunk uses deterministic fallback messages:

- **Commit mode** generates a message based on changed filenames (e.g., "Changes to auth.rs & config.rs")
- **Squash mode** generates a message listing the squashed commits

Resources: [llm documentation](https://llm.datasette.io/) | [aichat](https://github.com/sigoden/aichat)
