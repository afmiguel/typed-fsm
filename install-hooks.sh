#!/usr/bin/env bash
# Install git hooks for this repository
#
# Usage: ./install-hooks.sh

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}Installing git hooks...${NC}"

# Check if .git directory exists
if [ ! -d ".git" ]; then
    echo "Error: .git directory not found. Are you in the project root?"
    exit 1
fi

# Install pre-commit hook
if [ -f "hooks/pre-commit" ]; then
    cp hooks/pre-commit .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit
    echo -e "${GREEN}✓${NC} Installed pre-commit hook"
else
    echo "Warning: hooks/pre-commit not found"
fi

echo ""
echo -e "${GREEN}Git hooks installed successfully!${NC}"
echo ""
echo "The pre-commit hook will run CI checks before each commit."
echo "To bypass the hook for a single commit, use:"
echo "  git commit --no-verify"
