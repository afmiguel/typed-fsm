#!/usr/bin/env bash
# Local CI Validation Script
# Run this before committing to ensure all CI checks will pass
#
# Usage: ./ci-local.sh
# Or for verbose output: ./ci-local.sh --verbose

set -e  # Exit on first error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
MSRV="1.75.0"
VERBOSE=false

# Parse arguments
if [[ "$1" == "--verbose" ]]; then
    VERBOSE=true
fi

# Helper functions
print_step() {
    echo -e "${BLUE}==>${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

run_check() {
    local name="$1"
    local cmd="$2"

    print_step "$name"

    if $VERBOSE; then
        if eval "$cmd"; then
            print_success "$name passed"
        else
            print_error "$name failed"
            exit 1
        fi
    else
        if eval "$cmd" > /dev/null 2>&1; then
            print_success "$name passed"
        else
            print_error "$name failed"
            echo "Run with --verbose to see detailed errors"
            exit 1
        fi
    fi
}

echo "========================================="
echo "  Running Local CI Validation"
echo "========================================="
echo ""

# 1. Check formatting
run_check "Rustfmt (code formatting)" "cargo fmt --all -- --check"

# 2. Run clippy
run_check "Clippy (linting)" "cargo clippy --all-targets --all-features -- -D warnings"

# 3. Run tests
# Note: Tests must run with --test-threads=1 when concurrent feature is enabled
# because concurrent_tests.rs uses shared global static variables
print_step "Test Suite (all tests)"
if $VERBOSE; then
    cargo test --all-features -- --test-threads=1
    print_success "Test Suite passed"
else
    if ! cargo test --all-features --quiet -- --test-threads=1 > /dev/null 2>&1; then
        print_error "Test Suite failed"
        echo "Run with --verbose to see detailed errors"
        exit 1
    fi
    print_success "Test Suite passed"
fi

# 4. Build documentation
run_check "Documentation (doc build)" "RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features"

# 5. Check MSRV (if toolchain available)
print_step "MSRV Check (Rust $MSRV)"

# Check if rustup is available
if command -v rustup &> /dev/null; then
    # Check if MSRV toolchain is installed
    if rustup toolchain list | grep -q "$MSRV"; then
        if $VERBOSE; then
            if cargo +$MSRV check --all-features; then
                print_success "MSRV Check passed with Rust $MSRV"
            else
                print_error "MSRV Check failed"
                exit 1
            fi
        else
            if cargo +$MSRV check --all-features > /dev/null 2>&1; then
                print_success "MSRV Check passed with Rust $MSRV"
            else
                print_error "MSRV Check failed"
                echo "Run with --verbose to see detailed errors"
                exit 1
            fi
        fi
    else
        print_warning "MSRV toolchain ($MSRV) not installed, skipping MSRV check"
        echo "           Install with: rustup install $MSRV"
    fi
else
    print_warning "rustup not found, skipping MSRV check"
fi

# 6. Build with all features
run_check "Build (all features)" "cargo build --all-features"

echo ""
echo "========================================="
echo -e "${GREEN}✓ All CI checks passed!${NC}"
echo "========================================="
echo ""
echo "Safe to commit and push."
