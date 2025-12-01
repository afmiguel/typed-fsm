# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.4] - 2025-11-30

### Documentation
- **Added Raspberry Pi Pico 2 W Example**: Included a reference to the [typed-fsm-pico-test](https://github.com/afmiguel/typed-fsm-pico-test) repository in README and lib.rs. This demonstrates real-world usage on RP2350 with hardware integration (LED, ADC, Timer).

## [0.4.3] - 2025-11-30

### Fixed
- **Fixed `unexpected cfg condition` warnings** in consumer crates by conditionally defining `__fsm_log!` macro based on the `logging` feature flag.
- Removed `cfg` check inside the macro expansion to prevent leakage into consumer crate configuration checks.

## [0.4.2] - 2025-11-23

### Added
- **Comprehensive FAQ Section** - 18 new questions covering v0.3.0 and v0.4.0 features
  - Guards & Conditional Transitions (3 questions with code examples)
  - Logging & Observability (3 questions on zero-cost abstraction)
  - Timeouts & Time-Based Transitions (2 questions on timer patterns)
  - ISR & Interrupt Safety (4 questions on ISR-safe dispatch)
  - Concurrency & Thread Safety (6 questions on queue, overflow, performance)
  - **Total: 29 FAQ entries** covering all features from v0.1.0 through v0.4.0

This is a documentation-only release to help users understand and adopt the
new features introduced in v0.3.0 and v0.4.0.

## [0.4.1] - 2025-11-23

### Changed
- **README.md**: Enhanced documentation to prominently feature ISR-safe dispatch
  - "When to Choose Each" section now highlights ISR/concurrency as primary differentiator
  - Added notes to competitor options about lack of native ISR support
  - Added "Don't choose typed-fsm if you need" section for transparency
  - Updated "Key Advantages" to lead with ISR-safe concurrency
  - Fixed Quick Start version from 0.1.0 to 0.4

This is a documentation-only release to improve discoverability of the unique
ISR-safe dispatch feature on crates.io and docs.rs.

## [0.4.0] - 2025-11-23

### Added
- **Concurrency Support (Feature: `concurrent`)** - ISR and multithreading safe dispatch
  - Atomic protection against re-entrant dispatch calls
  - Lock-free event queuing with FIFO ordering
  - Safe dispatch from:
    - Interrupt Service Routines (ISRs) in embedded systems
    - Multiple threads in concurrent applications
    - Combined ISR + Thread scenarios (RTOS environments)
  - Implementation:
    - `AtomicBool` for dispatch lock management
    - `critical_section::Mutex` for queue protection
    - `heapless::Deque` for event queue (configurable capacity, default: 16 events)
    - Automatic queue processing before lock release
    - `AtomicUsize` counter for tracking dropped events
  - Performance:
    - ~10-15% overhead when feature enabled (with no contention)
    - ~100 cycles for ISR enqueue (fast and deterministic)
    - Zero overhead when feature disabled (standard implementation)
  - New dependencies (optional):
    - `critical-section` v1.1 - Portable critical sections
    - `heapless` v0.8 - No-alloc data structures
    - `paste` v1.0 - Macro hygiene for static names
  - **Dropped Events Monitoring** - Track queue overflow events
    - `dropped_events_count()` - Static method to query dropped events count
    - `reset_dropped_count()` - Static method to reset counter to zero
    - Debug mode protection: Panics on queue overflow during development
    - Release mode: Silent drop with atomic counter increment
    - Helps detect queue sizing issues early in development
  - **Configurable Queue Capacity** - Customize queue size per FSM
    - Optional `QueueCapacity` parameter in macro (e.g., `QueueCapacity: 64`)
    - Default capacity: 16 events (if not specified)
    - Each FSM can have different capacity based on its needs
    - Example: `QueueCapacity: 4` for low-throughput FSMs, `QueueCapacity: 64` for high-throughput
  - New examples:
    - `concurrent_isr.rs` - Simulated ISR with event queuing and atomic dispatch
    - `concurrent_threads.rs` - Multi-threaded task processor demonstrating thread-safe dispatch
  - Comprehensive test suite:
    - `tests/concurrent_tests.rs` - 21 exhaustive tests covering:
      - Single-threaded operation with no contention
      - Multi-threaded concurrent dispatch
      - FIFO event ordering verification (basic and strict)
      - State transitions under concurrency
      - High contention stress testing (with and without delays)
      - Event loss prevention with unique values
      - Reset during concurrent processing
      - Basic safety guarantees
      - **Critical scenarios**:
        - Queue overflow (>16 events) - verifies silent drop behavior
        - Immediate execution when dispatch is free
        - Queue then immediate execution transition
        - Complete queue processing before lock release
        - Extreme contention without delays (20 threads × 10 events)
        - Events dispatched during slow entry/exit hooks
        - Multiple rapid state transitions
        - Queue overflow drops events silently (no panic)

### Changed
- Updated `Cargo.toml`:
  - Added `concurrent` feature flag
  - Added optional dependencies: `critical-section`, `heapless`, `paste`
  - Added new example entries: `concurrent_isr`, `concurrent_threads`
- Updated `src/fsm.rs`:
  - Dual implementation with `#[cfg(feature = "concurrent")]`
  - Original zero-cost implementation preserved as default
  - Concurrent implementation with atomic protection
- Updated documentation:
  - `src/lib.rs` - New section on ISR and multithreading safety
  - `README.md` - Comprehensive concurrency support section with examples
  - Added usage patterns for ISR and multithreading scenarios
  - Performance characteristics and trade-offs
- Updated CLAUDE.md:
  - New test commands for concurrent feature
  - Updated test count (88+ tests)
  - New examples in project structure

### Tests
- **Concurrent Tests** (`tests/concurrent_tests.rs`) - 21 exhaustive tests:
  - **Basic scenarios** (8 tests):
    - Single-threaded operation with no contention
    - Multiple threads dispatching events concurrently
    - FIFO ordering verification under contention
    - State transitions with concurrency
    - High contention stress test (10 threads × 20 events)
    - Event loss prevention with unique values
    - Reset during concurrent processing
    - Basic safety guarantees
  - **Critical edge cases** (9 tests):
    - Queue overflow (>16 events) - verifies silent drop behavior
    - Immediate execution when dispatch is free
    - Queue then immediate execution transition
    - Complete queue processing before lock release
    - Extreme contention without delays (20 threads × 10 events = 200 total)
    - Strict FIFO ordering with barriers
    - Events dispatched during slow entry/exit hooks
    - Multiple rapid state transitions
    - Queue overflow drops silently (no panic)
  - **Dropped events & queue capacity** (4 tests):
    - Dropped events counter API verification
    - Dropped events counter reset functionality
    - Custom large queue capacity (64 events)
    - Custom small queue capacity (4 events)
  - **Note**: Tests must run sequentially (`--test-threads=1`) due to shared global state

**Total:** 100 tests (45 core tests + 34 v0.3.0 tests + 21 concurrent tests + 11+ doc tests)
**Coverage:** ~100% of code paths including all critical concurrency scenarios

### Known Limitations
- **Queue capacity**: Configurable per FSM (default: 16 events). Events are dropped when queue is full, but tracked via `dropped_events_count()` API. Use debug builds to catch overflow issues during development (will panic).
- **Event cloning**: Event types must implement `Clone` when using the `concurrent` feature.
- **Shared statics**: All FSMs of the same type share the same global lock and queue. In practice, each FSM has a unique type name, so this is rarely an issue.

### Internal
- Macro-level conditional compilation for concurrent/non-concurrent implementations
- Static variable generation with unique names per FSM (using `paste` crate)
- Atomic operations with appropriate memory orderings (Acquire/Release)
- Critical section abstraction for portable interrupt safety

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

[0.4.2]: https://github.com/afmiguel/typed-fsm/releases/tag/v0.4.2
[0.4.1]: https://github.com/afmiguel/typed-fsm/releases/tag/v0.4.1
[0.4.0]: https://github.com/afmiguel/typed-fsm/releases/tag/v0.4.0
[0.3.0]: https://github.com/afmiguel/typed-fsm/releases/tag/v0.3.0
[0.2.0]: https://github.com/afmiguel/typed-fsm/releases/tag/v0.2.0
[0.1.0]: https://github.com/afmiguel/typed-fsm/releases/tag/v0.1.0
