# Local CI Validation

This directory contains scripts to validate changes locally before committing, ensuring all GitHub Actions CI checks will pass.

## Quick Start

```bash
# 1. Run all CI checks locally
./ci-local.sh

# 2. (Optional) Install git pre-commit hook to run checks automatically
./install-hooks.sh
```

## Scripts

### `ci-local.sh`

Runs all CI checks that GitHub Actions will perform:

- ✓ **Rustfmt** - Code formatting validation
- ✓ **Clippy** - Linting with warnings as errors
- ✓ **Test Suite** - All unit, integration, and doc tests
- ✓ **Documentation** - Doc build with warnings as errors
- ✓ **MSRV Check** - Compatibility with Rust 1.75.0
- ✓ **Build** - Compilation with all features

**Usage:**
```bash
# Standard run (quiet mode)
./ci-local.sh

# Verbose mode (shows all output)
./ci-local.sh --verbose
```

### `install-hooks.sh`

Installs a git pre-commit hook that automatically runs `ci-local.sh` before each commit.

**Usage:**
```bash
./install-hooks.sh
```

After installation, CI checks run automatically before every commit. To bypass for a single commit:
```bash
git commit --no-verify
```

### `hooks/pre-commit`

The actual pre-commit hook that gets installed into `.git/hooks/`.

## MSRV Validation

To run the complete MSRV (Minimum Supported Rust Version) check, install Rust 1.75.0:

```bash
rustup install 1.75.0
```

The `ci-local.sh` script will automatically use this toolchain to verify compatibility. If not installed, it will skip the MSRV check with a warning.

## Why Use This?

Running CI checks locally before committing:

1. **Saves time** - Catch errors before pushing
2. **Faster feedback** - Local checks are faster than waiting for CI
3. **Better workflow** - Fix issues immediately while context is fresh
4. **Prevents broken commits** - Ensures CI will pass

## Workflow Recommendation

### Option 1: Manual validation (conservative)
```bash
# Make changes...
git add .
./ci-local.sh        # Validate before committing
git commit -m "..."  # Commit if all checks pass
git push
```

### Option 2: Pre-commit hook (automated)
```bash
./install-hooks.sh   # One-time setup

# Then just commit normally:
git add .
git commit -m "..."  # Hook runs automatically
git push
```

### Option 3: Quick commits (when you're confident)
```bash
# Make changes...
git add .
git commit -m "..."
./ci-local.sh        # Validate after commit
git push             # Only push if checks pass
```

## Troubleshooting

### "MSRV toolchain not installed"
```bash
rustup install 1.75.0
```

### "permission denied"
```bash
chmod +x ci-local.sh install-hooks.sh hooks/pre-commit
```

### Checks fail locally
1. Run with `--verbose` to see detailed errors
2. Fix the issues
3. Run again until all checks pass

### Need to commit despite failing checks
```bash
# NOT recommended, but possible in emergencies:
git commit --no-verify
```

## Integration with Claude Code

When working with Claude Code, always mention these scripts:

> "Before committing, run ./ci-local.sh to validate all CI checks locally"

This ensures Claude validates changes before creating commits, preventing CI failures.
