# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**typed-fsm** is a lightweight, zero-cost, **event-driven** Finite State Machine (FSM) microframework for Rust. It's designed for embedded systems (no-std compatible) and high-performance applications. The framework uses macros to generate type-safe, event-driven state machines with zero heap allocations.

## Development Commands

```bash
# Build the library
cargo build

# Build with release optimizations
cargo build --release

# Run the motor control example
cargo run --example motor

# Run the traffic light example
cargo run --example traffic_light

# Check for errors without building
cargo check

# Run all tests (30+ tests)
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test suites
cargo test --test integration_tests  # Integration tests
cargo test --test coverage_tests     # Coverage tests (lifecycle hooks, states)
cargo test --test edge_cases_tests   # Edge cases (patterns, early returns)

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
│   └── fsm.rs              # Core macro implementation + unit tests
├── examples/
│   ├── motor.rs            # Motor control system example (complex, event-driven)
│   └── traffic_light.rs    # Traffic light controller (simple, event-driven)
├── tests/
│   ├── integration_tests.rs  # Integration tests (toggle, counter, resources)
│   ├── coverage_tests.rs     # Comprehensive coverage tests (10 tests)
│   └── edge_cases_tests.rs   # Edge cases and special scenarios (8 tests)
├── Cargo.toml              # Package metadata and dependencies
├── README.md               # User-facing documentation
├── LICENSE-MIT             # MIT License
├── LICENSE-APACHE          # Apache 2.0 License
└── CLAUDE.md               # This file

Test Coverage: 30+ tests covering ~100% of code paths
Package Name: typed-fsm (crate name: typed_fsm)
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
