# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**typed-fsm** is a lightweight, zero-cost, **event-driven** Finite State Machine (FSM) microframework for Rust. It's designed for embedded systems (no-std compatible) and high-performance applications. The framework uses macros to generate type-safe, event-driven state machines with zero heap allocations.

**Version:** 0.4.0-dev (on branch develop/v0.4.0)

**New in v0.3.0:**
- Guards (conditional transitions)
- Logging support (optional via feature flags)
- Timeouts (Timer trait abstraction pattern)

**New in v0.4.0:**
- Concurrency support (ISR and multithreading safe dispatch)
- Dropped events monitoring (`dropped_events_count()`, `reset_dropped_count()`)
- Configurable queue capacity per FSM (`QueueCapacity` parameter)

## CI Validation (Run Before Committing)

**IMPORTANT:** Always validate changes locally before committing to avoid CI failures.

```bash
# Run all CI checks locally (recommended before every commit)
./ci-local.sh

# Run with verbose output to see detailed errors
./ci-local.sh --verbose

# Optional: Install git pre-commit hook (runs CI checks automatically)
./install-hooks.sh

# If hook is installed, bypass it for a single commit with:
git commit --no-verify
```

### What ci-local.sh checks:
- ✓ Code formatting (rustfmt)
- ✓ Linting (clippy with -D warnings)
- ✓ All tests (unit, integration, doc tests)
- ✓ Documentation build
- ✓ MSRV compatibility (Rust 1.75.0)
- ✓ Build with all features

**MSRV Setup (required for full validation):**
```bash
# Install the MSRV toolchain for complete local validation
rustup install 1.75.0

# The script will automatically use it to verify MSRV compatibility
```

## Development Commands

```bash
# Build the library
cargo build

# Build with release optimizations
cargo build --release

# Run examples
cargo run --example motor          # Motor control system
cargo run --example traffic_light  # Traffic light controller
cargo run --example guards         # Guards (conditional transitions)
cargo run --example timeouts       # Timeouts (timer trait pattern)

# Run logging example with feature flag
RUST_LOG=info cargo run --example logging --features logging

# Check for errors without building
cargo check

# Run all tests (100 tests with ~100% coverage)
cargo test --all-features -- --test-threads=1

# Run tests with output
cargo test --all-features -- --nocapture --test-threads=1

# Run specific test suites
cargo test --test integration_tests  # Integration tests (13 tests)
cargo test --test coverage_tests     # Coverage tests (10 tests)
cargo test --test edge_cases_tests   # Edge cases (8 tests)
cargo test --test guards_tests       # Guards tests (14 tests) - v0.3.0
cargo test --test logging_tests      # Logging tests (9 tests) - v0.3.0
cargo test --test timeouts_tests     # Timeouts tests (11 tests) - v0.3.0
cargo test --test concurrent_tests --features concurrent -- --test-threads=1  # Concurrent tests (21 tests) - v0.4.0

# Run only unit tests
cargo test --lib

# Run only doc tests
cargo test --doc

# Build documentation
cargo doc --open

# Format code
cargo fmt

# Run clippy for linting
cargo clippy

# Run clippy with all features
cargo clippy --all-targets --all-features
```

## Project Structure

```
typed-fsm/
├── src/
│   ├── lib.rs              # Library entry point, public API
│   └── fsm.rs              # Core macro implementation + unit tests + logging + concurrency
├── examples/
│   ├── motor.rs            # Motor control system (complex, event-driven)
│   ├── traffic_light.rs    # Traffic light controller (simple, event-driven)
│   ├── guards.rs           # Guards: ATM, door lock, order processing (v0.3.0)
│   ├── logging.rs          # Logging: payment FSM with instrumentation (v0.3.0)
│   ├── timeouts.rs         # Timeouts: WiFi, session, button debouncing (v0.3.0)
│   ├── concurrent_isr.rs   # Concurrent: ISR-safe dispatch (v0.4.0)
│   └── concurrent_threads.rs # Concurrent: Multi-threaded dispatch (v0.4.0)
├── tests/
│   ├── integration_tests.rs  # Integration tests (13 tests)
│   ├── coverage_tests.rs     # Coverage tests (10 tests)
│   ├── edge_cases_tests.rs   # Edge cases (8 tests)
│   ├── guards_tests.rs       # Guards tests (14 tests) - v0.3.0
│   ├── logging_tests.rs      # Logging tests (9 tests) - v0.3.0
│   ├── timeouts_tests.rs     # Timeouts tests (11 tests) - v0.3.0
│   └── concurrent_tests.rs   # Concurrent tests (21 tests) - v0.4.0
├── Cargo.toml              # Package metadata, dependencies, features
├── README.md               # User-facing documentation
├── CHANGELOG.md            # Version history and release notes
├── ROADMAP.md              # Development roadmap and advanced features
├── LICENSE-MIT             # MIT License
├── LICENSE-APACHE          # Apache 2.0 License
└── CLAUDE.md               # This file

Test Coverage: 100 tests covering ~100% of code paths
  - Core: 3 unit + 10 coverage + 8 edge cases + 13 integration = 34 tests
  - v0.3.0: 14 guards + 9 logging + 11 timeouts = 34 tests
  - v0.4.0: 21 concurrent tests = 21 tests
  - Docs: 11 doc tests
Package Name: typed-fsm (crate name: typed_fsm)
Version: 0.4.0-dev
```

## Architecture

### Core Components

**Library (`src/lib.rs`)**
- Public API exports: `Transition` enum and `state_machine!` macro
- Marked as `#![no_std]` for embedded compatibility
- Contains usage examples in documentation

**State Machine Macro (`src/fsm.rs`)**
- The `state_machine!` macro generates a complete FSM implementation from a declarative specification
- Produces a `pub enum` with states as variants, where each state can carry typed data
- Implements three lifecycle hooks per state:
  - `entry`: Executed once when entering the state
  - `process`: Handles events and returns `Transition<S>`
  - `exit`: Executed once when leaving the state
- The `dispatch` method orchestrates the full transition lifecycle: Process → Exit → Update → Entry

**Transition Model**
- `Transition::None`: Stay in current state
- `Transition::To(S)`: Move to a new state (can carry payload data)

**Zero-Cost Design Principles**
- Uses static dispatch via enums (no `Box`, `dyn`, or heap allocations)
- Relies on Rust's type system to prevent invalid state transitions at compile time
- The `dispatch` method is marked `#[inline(always)]` to enable compiler optimization into jump tables

### Example Application (`examples/motor.rs`)

Demonstrates a motor control system with three states:
- `Idle`: Waiting for user command
- `Running { target_speed: u32 }`: Motor active with PID control and overspeed protection
- `Error { code: u16 }`: Safety latch requiring manual reset

The example shows:
- Context pattern: `MotorContext` holds hardware state (sensors/actuators)
- Event-driven architecture: `Input` enum represents system events (buttons, sensor ticks)
- State data: `Running` and `Error` states carry typed payloads
- Safety-critical transitions: Overspeed detection triggers automatic transition to `Error` state

### Key Patterns

**Initialization**
State machines must call `.init(&mut ctx)` before the event loop to execute the entry action of the initial state.

**State Variables**
States can hold data (e.g., `Running { target_speed: u32 }`). These fields are automatically available in entry/process/exit blocks.

**Context vs State**
- Context (`MotorContext`): Shared hardware/system state accessible across all states
- State data: Specific to individual state variants, carries transition payloads

## Test Coverage

The project has near 100% code coverage with comprehensive test suites:

### Unit Tests (`src/fsm.rs`)
- Tests for `Transition` enum (3 tests)
- Validates pattern matching and value carrying

### Integration Tests (`tests/integration_tests.rs`)
- Simple toggle state machine
- Counter with stateful states
- Resource management with exit actions
- Events without state transitions
- Multiple fields in states
Total: 5 tests

### Coverage Tests (`tests/coverage_tests.rs`)
Comprehensive tests covering all macro-generated code paths:
- All lifecycle hooks (entry, process, exit)
- States without entry hooks
- States without exit hooks
- Self-transitions
- Multiple consecutive transitions
- Complex multi-field states
- Transition::None behavior verification
- Single-state FSMs
- Pattern matching in process blocks
- Minimal states (process-only)
Total: 10 tests

### Edge Case Tests (`tests/edge_cases_tests.rs`)
Advanced scenarios and special patterns:
- Early returns in process blocks
- Unused context and event parameters
- Partial field usage
- Nested match patterns
- Multiple self-transitions
- States with varying field counts (0-3 fields)
- Wildcard patterns in match
- If-let patterns in process
Total: 8 tests

### Doc Tests
- All code examples in documentation are tested (4 tests)

**Test Strategy**: Tests validate not just correct behavior but also:
- All branches of generated code
- Optional entry/exit actions
- State transitions (including self-transitions)
- Early returns and complex control flow
- Different match patterns and idioms
