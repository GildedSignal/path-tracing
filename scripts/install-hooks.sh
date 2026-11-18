#!/usr/bin/env bash
# Copies the committed git hooks into .git/hooks (pre-push stays local: it names a private string).
cd "$(git rev-parse --show-toplevel)" && cp scripts/git-hooks/pre-commit scripts/git-hooks/commit-msg .git/hooks/ && chmod +x .git/hooks/pre-commit .git/hooks/commit-msg && echo "hooks installed"
