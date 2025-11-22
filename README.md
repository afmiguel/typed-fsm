# typed-fsm

[![Crates.io](https://img.shields.io/crates/v/typed-fsm.svg)](https://crates.io/crates/typed-fsm)
[![Documentation](https://docs.rs/typed-fsm/badge.svg)](https://docs.rs/typed-fsm)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

A lightweight, zero-cost, **event-driven** finite state machine microframework for Rust.

**typed-fsm** provides a declarative macro-based approach to building type-safe, event-driven state machines. Perfect for embedded systems, protocol implementations, game logic, and any application requiring robust state management.

## Features

- ⚡ **Event-Driven Architecture** - Built from the ground up for event-based systems
- 🚀 **Zero-cost abstraction** - Compiles to efficient jump tables with no runtime overhead
- 🔒 **Type-safe** - Compile-time validation of state transitions and events
- 🎯 **Declarative** - Clean, readable syntax using macros
- 💾 **No allocations** - Uses enums and static dispatch (no `Box`, `dyn`, or heap)
- 🔧 **Embedded-ready** - `#![no_std]` compatible
- 📦 **Stateful states** - States can carry typed data
- 🔄 **Lifecycle hooks** - `entry`, `process`, and `exit` actions per state

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
typed-fsm = "0.1.0"
```

### Simple Example

```rust
use typed_fsm::{state_machine, Transition};

// Define your context (shared state)
struct LightContext {
    brightness: u8,
}

// Define your events
enum LightEvent {
    TurnOn,
    TurnOff,
}

// Create your state machine
state_machine! {
    Name: LightFSM,
    Context: LightContext,
    Event: LightEvent,

    States: {
        Off => {
            entry: |ctx| {
                ctx.brightness = 0;
            }

            process: |_ctx, evt| {
                match evt {
                    LightEvent::TurnOn => Transition::To(LightFSM::On),
                    _ => Transition::None
                }
            }
        },

        On => {
            entry: |ctx| {
                ctx.brightness = 100;
            }

            process: |_ctx, evt| {
                match evt {
                    LightEvent::TurnOff => Transition::To(LightFSM::Off),
                    _ => Transition::None
                }
            }
        }
    }
}

fn main() {
    let mut ctx = LightContext { brightness: 0 };
    let mut fsm = LightFSM::Off;

    // Initialize the state machine
    fsm.init(&mut ctx);

    // Dispatch events
    fsm.dispatch(&mut ctx, &LightEvent::TurnOn);
    assert_eq!(ctx.brightness, 100);

    fsm.dispatch(&mut ctx, &LightEvent::TurnOff);
    assert_eq!(ctx.brightness, 0);
}
```

## Advanced Features

### Stateful States

States can carry typed data:

```rust
state_machine! {
    Name: MotorFSM,
    Context: MotorContext,
    Event: MotorEvent,

    States: {
        Running { target_speed: u32 } => {
            entry: |_ctx| {
                println!("Target speed: {} RPM", target_speed);
            }

            process: |ctx, evt| {
                // Access target_speed with *target_speed
                match evt {
                    MotorEvent::SetSpeed(speed) => {
                        *target_speed = *speed;
                        Transition::None
                    },
                    _ => Transition::None
                }
            }
        }
    }
}
```

### Lifecycle Hooks

Each state supports three optional hooks:

- **`entry`** - Executed once when entering the state
- **`process`** - Handles events, returns `Transition<S>`
- **`exit`** - Executed once when leaving the state

```rust
state_machine! {
    Name: MyFSM,
    Context: MyContext,
    Event: MyEvent,

    States: {
        Active => {
            entry: |ctx| {
                // Initialize resources
            }

            process: |ctx, evt| {
                // Handle events
                Transition::None
            }

            exit: |ctx| {
                // Cleanup resources
            }
        }
    }
}
```

## Testing

This library has comprehensive test coverage (near 100%) with 30+ tests covering:

- **Unit tests** - Core `Transition` enum functionality
- **Integration tests** - Complete FSM scenarios (toggle, counter, resources)
- **Coverage tests** - All lifecycle hooks, optional entry/exit, self-transitions, multi-field states
- **Edge case tests** - Early returns, nested patterns, wildcard matches, if-let patterns
- **Doc tests** - All documentation examples are tested

Run all tests:

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run a specific test suite
cargo test --test integration_tests
cargo test --test coverage_tests
cargo test --test edge_cases_tests
```

## Examples

Run the included examples:

```bash
# Motor control system with safety checks
cargo run --example motor

# Traffic light controller
cargo run --example traffic_light
```

## How It Works

The `state_machine!` macro generates:

1. A `pub enum` with your states as variants
2. Implementation of `init()`, `dispatch()`, and internal lifecycle methods
3. Type-safe state transitions with compile-time validation

The generated code uses:
- Static dispatch (no `dyn Trait`)
- Move semantics for state transitions
- `#[inline(always)]` for zero-cost abstraction

## Performance

This library is designed for performance-critical applications:

- **Zero heap allocations** - All state data is stack-allocated
- **Optimal codegen** - Compiles to efficient jump tables
- **No runtime overhead** - State transitions are simple enum assignments
- **Embedded-friendly** - No dependencies, `#![no_std]` compatible

## Use Cases

- Embedded systems and firmware
- Protocol implementations
- Game logic
- UI state management
- Robotics and control systems
- Workflow engines

## Documentation

Full API documentation is available at [docs.rs/typed-fsm](https://docs.rs/typed-fsm).

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
