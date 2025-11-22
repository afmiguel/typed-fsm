# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2025-11-22

### Added
- **Guards (Conditional Transitions)** - Pattern for implementing guard conditions
  - Documentation with comprehensive examples
  - 3 real-world scenarios:
    - ATM security guards (PIN verification, retry limits)
    - Door lock access control (authorized codes)
    - Order processing business rules (stock checks, credit limits)
  - New example: `guards.rs` with complete demonstrations
  - README section explaining guards pattern and best practices

- **Logging Support** - Optional instrumentation via feature flags
  - Zero-cost when disabled (no code generation)
  - Support for both `log` and `tracing` crates
  - Automatic logging of:
    - FSM initialization (`init()`)
    - State entry actions (`entry()`)
    - State exit actions (`exit()`)
    - State transitions with events
    - Events that don't trigger transitions
  - New example: `logging.rs` demonstrating payment processing with logs
  - Feature flags: `logging` (for log crate) and `tracing` (for tracing-core crate)

- **Timeouts (Time-Based Transitions)** - Timer trait abstraction pattern
  - Platform-agnostic Timer trait for timeout handling
  - Maintains `no_std` compatibility (user-provided implementations)
  - Zero impact on users who don't need timeouts
  - Complete documentation with platform-specific examples:
    - `std`: Using `std::time::Instant`
    - Embedded: Using HAL timer peripherals
    - Testing: Using mock time
  - New example: `timeouts.rs` with 3 comprehensive scenarios:
    - WiFi connection with timeout and retry logic
    - Session timeout with idle detection
    - Button debouncing with time delays
  - Best practices and usage patterns

- Comprehensive ROADMAP.md documenting 10 advanced features
  - 3 development phases (Basic, Intermediate, Advanced)
  - Feature comparison matrix (priority, complexity, no_std compatibility)
  - Version roadmap (v0.3.0 through v0.6.0)

### Changed
- Updated README.md with new sections:
  - Guards (Conditional Transitions)
  - Timeouts (Time-Based Transitions)
  - Logging patterns and examples
- Enhanced documentation in CLAUDE.md
- Updated Cargo.toml:
  - Version: 0.3.0-dev
  - New optional dependencies: `log`, `tracing-core`
  - New dev-dependency: `log` for examples
  - New example entries: `guards`, `logging`, `timeouts`

### Tests
- **Guards Tests** (`tests/guards_tests.rs`) - 14 comprehensive tests:
  - PIN verification with retry limits
  - Multiple guard conditions (AND logic)
  - Range checks (temperature monitoring)
  - List membership checks (access control)
- **Logging Tests** (`tests/logging_tests.rs`) - 9 comprehensive tests:
  - Zero-cost abstraction verification
  - Lifecycle hooks validation
  - Self-transitions
  - Full payment processing lifecycle
- **Timeouts Tests** (`tests/timeouts_tests.rs`) - 11 comprehensive tests:
  - MockTimer implementation
  - Timeout detection and transitions
  - Retry logic with delays
  - Session timeout with inactivity detection
  - Timer reset behavior

**Total:** 79 tests (45 core tests + 34 v0.3.0 tests + 11 doc tests)
**Coverage:** ~100% of code paths

### Internal
- Added `__fsm_log!` internal macro for logging instrumentation
- Instrumented all lifecycle hooks (init, entry, exit, dispatch)
- Conditional compilation for logging features

## [0.2.0] - 2025-01-22

### Added
- Thread safety and concurrency support
  - Documentation for `Send + Sync` auto-traits
  - `Arc<Mutex<>>` pattern examples
  - 2 new concurrency integration tests
- New examples:
  - `blink.rs` - Simplest possible FSM (LED On/Off)
  - `hierarchical.rs` - Nested state machines (audio player with volume control)
  - `traffic_intersection.rs` - Concurrent FSMs with 3 parallel state machines
- Comprehensive async documentation
  - "Using with Async Code" section explaining async patterns
  - Clarification that FSMs can be used within async contexts
  - Examples of async wrapper patterns
- Honest comparison table with real Rust FSM crates
  - Replaced placeholder crates with statig, smlang, and rust-fsm
  - Added "When to Choose Each" guidance
  - Links to crates.io for each alternative
  - New comparison rows: Hierarchical FSM, Thread-safe, Async support, Diagram generation

### Changed
- Improved README.md with honest feature comparisons
- Updated documentation to clarify async usage patterns
- Enhanced concurrency examples and patterns

### Removed
- Emojis from core code documentation (src/fsm.rs)
- Placeholder/non-existent crates from comparison table

## [0.1.0] - 2025-01-21

### Added
- Initial public release of typed-fsm
- Event-driven finite state machine macro `state_machine!`
- Support for stateful states with typed data fields
- Lifecycle hooks: `entry`, `process`, and `exit`
- Type-safe state transitions with `Transition<S>` enum
- Zero-cost abstraction with static dispatch
- `#![no_std]` compatibility for embedded systems
- Comprehensive test suite with 30+ tests covering ~100% of code paths
  - Unit tests for `Transition` enum
  - Integration tests (toggle, counter, resource management)
  - Coverage tests (all lifecycle hooks, optional entry/exit, self-transitions)
  - Edge case tests (early returns, nested patterns, wildcard matches)
- Examples:
  - Motor control system with safety checks
  - Traffic light controller
- Professional documentation:
  - README.md with quick start and advanced features
  - CLAUDE.md for AI-assisted development
  - Dual licensing (MIT/Apache-2.0)

[0.3.0]: https://github.com/afmiguel/typed-fsm/releases/tag/v0.3.0
[0.2.0]: https://github.com/afmiguel/typed-fsm/releases/tag/v0.2.0
[0.1.0]: https://github.com/afmiguel/typed-fsm/releases/tag/v0.1.0
