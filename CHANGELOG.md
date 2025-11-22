# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[0.1.0]: https://github.com/afmiguel/typed-fsm/releases/tag/v0.1.0
