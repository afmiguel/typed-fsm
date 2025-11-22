# typed-fsm

[![Crates.io](https://img.shields.io/crates/v/typed-fsm.svg)](https://crates.io/crates/typed-fsm)
[![Documentation](https://docs.rs/typed-fsm/badge.svg)](https://docs.rs/typed-fsm)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![CI](https://github.com/afmiguel/typed-fsm/workflows/CI/badge.svg)](https://github.com/afmiguel/typed-fsm/actions)
[![Downloads](https://img.shields.io/crates/d/typed-fsm.svg)](https://crates.io/crates/typed-fsm)

A lightweight, zero-cost, **event-driven** finite state machine microframework for Rust.

**typed-fsm** provides a declarative macro-based approach to building type-safe, event-driven state machines. Perfect for embedded systems, protocol implementations, game logic, and any application requiring robust state management.

## Features

### Core
- **Event-Driven Architecture** - Built from the ground up for event-based systems
- **Zero-cost abstraction** - Compiles to efficient jump tables with no runtime overhead
- **Type-safe** - Compile-time validation of state transitions and events
- **Declarative** - Clean, readable syntax using macros
- **No allocations** - Uses enums and static dispatch (no `Box`, `dyn`, or heap)
- **Embedded-ready** - `#![no_std]` compatible
- **Stateful states** - States can carry typed data
- **Lifecycle hooks** - `entry`, `process`, and `exit` actions per state

### Advanced (v0.3.0)
- **Guards** - Conditional transitions with boolean logic (security, validation, business rules)
- **Logging** - Optional instrumentation via `log` or `tracing` crates (zero-cost when disabled)
- **Timeouts** - Timer trait abstraction pattern for time-based transitions (platform-agnostic)

## Why typed-fsm?

### Comparison with Alternatives

| Feature | [typed-fsm][t] | [statig][st] | [smlang][sl] | [rust-fsm][rf] |
|---------|----------------|--------------|--------------|----------------|
| **Event-driven** | ✓ | ✓ | ✓ | ✓ |
| **Zero-cost** | ✓ | ~ | ~ | ~ |
| **no_std** | ✓ | ✓ | ✓ | ✓ |
| **Stateful states** | ✓ | ✓ | ✓ | ✗¹ |
| **Lifecycle hooks** | ✓ | ✓ | ~² | ✗ |
| **Hierarchical FSM** | ~³ | ✓ | ✗ | ✗ |
| **Thread-safe (Send+Sync)** | ✓ | ? | ? | ? |
| **Macro-based DSL** | ✓ | ✓ | ✓ | ✓ |
| **Type-safe** | ✓ | ✓ | ✓ | ✓ |
| **Dependencies** | 0 | 3⁴ | 1 | 2⁴ |
| **Async support** | ✗⁵ | ✓ | ✓ | ✗ |
| **Diagram generation** | ✗ | ✗ | ✗ | ✓ |

[t]: https://crates.io/crates/typed-fsm
[st]: https://crates.io/crates/statig
[sl]: https://crates.io/crates/smlang
[rf]: https://crates.io/crates/rust-fsm

¹ rust-fsm: States cannot carry data in DSL (manual implementation possible)
² smlang: Has guards/actions, but not explicit entry/exit hooks per state
³ typed-fsm: Via nested FSMs in context (compositional, not native like statig)
⁴ Optional dependencies (can be disabled with feature flags)
⁵ typed-fsm: Can be used within async code, but hooks are synchronous (no async fn support)

### When to Choose Each

**Choose typed-fsm if you need:**
- Absolute zero dependencies (embedded, security-critical)
- Guaranteed zero-cost abstraction with no runtime overhead
- Explicit lifecycle hooks (entry/process/exit)
- Clear thread-safety guarantees (auto Send+Sync)
- Simple API without hierarchical complexity

**Choose statig if you need:**
- Native hierarchical state machines with superstates
- Async/await support for concurrent state machines
- State-local storage (data bound to specific states)
- More mature ecosystem with extensive documentation

**Choose smlang if you need:**
- Async state machines out of the box
- Guards (conditional transitions) and actions
- DSL-first approach with procedural macros

**Choose rust-fsm if you need:**
- Automatic Mermaid diagram generation
- Classical Mealy/Moore machine patterns
- Large community and battle-tested in production
- Flexible transition specifications

### Key Advantages of typed-fsm

1. **True Zero Dependencies** - The only FSM library with absolutely no dependencies, perfect for security-critical applications
2. **Genuine Zero-Cost Abstraction** - Compiles to optimal code without procedural macro overhead
3. **Thread-Safety by Design** - Automatic Send+Sync when applicable, explicitly documented and tested
4. **Complete Lifecycle Model** - Clean entry/process/exit pattern without DSL limitations
5. **Embedded-First** - Designed for resource-constrained environments from day one

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
typed-fsm = "0.1.0"
```

### Simplest Example: Blink

A minimal LED blink state machine:

```rust
use typed_fsm::{state_machine, Transition};

// Context: Shared state
struct LedContext {
    tick_count: u32,
}

// Event: Simple tick
enum Event {
    Tick,a 
}

// State machine: On ↔ Off
state_machine! {
    Name: BlinkFSM,
    Context: LedContext,
    Event: Event,

    States: {
        On => {
            entry: |ctx| {
                ctx.tick_count += 1;
                println!("LED ON");
            }

            process: |_ctx, event| {
                match event {
                    Event::Tick => Transition::To(BlinkFSM::Off),
                }
            }
        },

        Off => {
            entry: |ctx| {
                ctx.tick_count += 1;
                println!("LED OFF");
            }

            process: |_ctx, event| {
                match event {
                    Event::Tick => Transition::To(BlinkFSM::On),
                }
            }
        }
    }
}

fn main() {
    let mut ctx = LedContext { tick_count: 0 };
    let mut led = BlinkFSM::On;

    // ⚠️ CRITICAL: Must call init() before event loop!
    led.init(&mut ctx);

    // Dispatch events
    led.dispatch(&mut ctx, &Event::Tick);  // On → Off
    led.dispatch(&mut ctx, &Event::Tick);  // Off → On

    println!("Total ticks: {}", ctx.tick_count);
}
```

## 🔄 Understanding Transitions

The `process` hook **must** return a `Transition` enum to tell the state machine what to do next:

### `Transition::None` - Stay in Current State

Use when an event should be handled but doesn't require changing states:

```rust
process: |ctx, evt| {
    match evt {
        MyEvent::UpdateData(value) => {
            ctx.data = *value;  // Update context
            Transition::None     // Stay in same state
        }
    }
}
```

**When to use:**
- Event updates context but state remains the same
- Event should be ignored in this state
- Processing an event that doesn't affect state flow

**What happens:**
- ✅ `process` executes
- ❌ `exit` does NOT execute (no state change)
- ❌ `entry` does NOT execute (no state change)
- ✅ State remains unchanged

### `Transition::To(State)` - Move to New State

Use when an event should trigger a state change:

```rust
process: |ctx, evt| {
    match evt {
        MyEvent::Start => {
            Transition::To(MyFSM::Running { speed: 100 })
        }
    }
}
```

**When to use:**
- Event triggers a state change
- Conditions are met for transitioning
- Need to move to a different state (including self-transitions)

**What happens:**
1. ✅ `process` executes and returns new state
2. ✅ Current state's `exit` executes (if defined)
3. ✅ New state's `entry` executes (if defined)
4. ✅ State updates to the new state

### Example: Combining Both

```rust
state_machine! {
    Name: DoorFSM,
    Context: DoorContext,
    Event: DoorEvent,

    States: {
        Closed => {
            process: |ctx, evt| {
                match evt {
                    DoorEvent::Open => {
                        // Change state
                        Transition::To(DoorFSM::Open)
                    },
                    DoorEvent::Lock => {
                        // Stay in same state but update context
                        ctx.locked = true;
                        Transition::None
                    },
                    DoorEvent::Close => {
                        // Already closed, do nothing
                        Transition::None
                    }
                }
            }
        },

        Open => {
            process: |ctx, evt| {
                match evt {
                    DoorEvent::Close => {
                        // Change state
                        Transition::To(DoorFSM::Closed)
                    },
                    DoorEvent::Open | DoorEvent::Lock => {
                        // Invalid in this state, ignore
                        Transition::None
                    }
                }
            }
        }
    }
}
```

**Key Points:**
- **Every `process` block must return a `Transition`**
- Use `Transition::None` for events that don't change state
- Use `Transition::To(State)` for events that trigger transitions
- You can update context in `process` before returning
- The transition type determines whether `exit`/`entry` hooks run

## ⚠️ Important: Initialization

**You MUST call `.init(&mut ctx)` before dispatching any events!**

### Why is init() required?

The `init()` method executes the `entry` action of the initial state. Forgetting to call it will cause:

- The entry action of the initial state will NEVER execute
- The state machine will still process events, but initialization is skipped
- This can lead to incorrect behavior that is difficult to debug

### Correct Usage Pattern

```rust
// 1. Create context
let mut ctx = MyContext { /* ... */ };

// 2. Create state machine
let mut fsm = MyFSM::InitialState;

// 3. ⚠️ CRITICAL: Initialize BEFORE event loop
fsm.init(&mut ctx);

// 4. Now safe to dispatch events
loop {
    fsm.dispatch(&mut ctx, &event);
}
```

### Common Mistake (Don't Do This!)

```rust
let mut ctx = MyContext { /* ... */ };
let mut fsm = MyFSM::InitialState;

// ❌ WRONG: Forgot to call init()!
// Entry action will NEVER execute!
fsm.dispatch(&mut ctx, &event);  // Silent failure
```

See the [blink example](examples/blink.rs) for a complete demonstration.

## 📋 Quick Start Template

Copy and paste this template to start building your state machine. Replace the `UPPERCASE` placeholders with your actual names:

```rust
use typed_fsm::{state_machine, Transition};

// 1. Define your context (shared state across all states)
struct MY_CONTEXT {
    MY_FIELD: MY_TYPE,
}

// 2. Define your events (what can happen to trigger transitions)
enum MY_EVENT {
    MY_EVENT_1,
    MY_EVENT_2,
}

// 3. Create your state machine
state_machine! {
    Name: MY_FSM,
    Context: MY_CONTEXT,
    Event: MY_EVENT,

    States: {
        MY_STATE_1 => {
            entry: |ctx| {
                // Runs once when entering this state
            }

            process: |ctx, evt| {
                match evt {
                    MY_EVENT::MY_EVENT_1 => Transition::To(MY_FSM::MY_STATE_2),
                    MY_EVENT::MY_EVENT_2 => Transition::None
                }
            }

            exit: |ctx| {
                // Runs once when leaving this state
            }
        },

        MY_STATE_2 => {
            process: |ctx, evt| {
                match evt {
                    MY_EVENT::MY_EVENT_1 => Transition::To(MY_FSM::MY_STATE_1),
                    _ => Transition::None
                }
            }
        }
    }
}

fn main() {
    // 1. Create context
    let mut ctx = MY_CONTEXT {
        MY_FIELD: MY_VALUE,
    };

    // 2. Create state machine (start in initial state)
    let mut fsm = MY_FSM::MY_STATE_1;

    // 3. ⚠️ CRITICAL: Initialize before event loop!
    fsm.init(&mut ctx);

    // 4. Event loop - dispatch events
    fsm.dispatch(&mut ctx, &MY_EVENT::MY_EVENT_1);
    fsm.dispatch(&mut ctx, &MY_EVENT::MY_EVENT_2);
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

Each state supports three lifecycle hooks:

- **`entry`** (optional) - Executed once when entering the state
- **`process`** (**required**) - Handles events, returns `Transition<S>`
- **`exit`** (optional) - Executed once when leaving the state

```rust
state_machine! {
    Name: MyFSM,
    Context: MyContext,
    Event: MyEvent,

    States: {
        Active => {
            entry: |ctx| {
                ctx.resource.acquire();
            }

            process: |ctx, evt| {
                Transition::None
            }

            exit: |ctx| {
                ctx.resource.release();
            }
        }
    }
}
```

## Concurrency Support

typed-fsm supports concurrent state machines through composition with Rust's standard concurrency primitives.

### Thread Safety

FSMs are automatically `Send + Sync` if their fields are `Send + Sync`. This allows safe sharing across threads using `Arc<Mutex<>>`.

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use typed_fsm::{state_machine, Transition};

// FSM is automatically Send + Sync
let fsm = Arc::new(Mutex::new(MyFSM::Initial));
let ctx = Arc::new(Mutex::new(MyContext { /* ... */ }));

// Clone for another thread
let fsm_clone = Arc::clone(&fsm);
let ctx_clone = Arc::clone(&ctx);

thread::spawn(move || {
    let mut fsm_lock = fsm_clone.lock().unwrap();
    let mut ctx_lock = ctx_clone.lock().unwrap();
    fsm_lock.dispatch(&mut *ctx_lock, &event);
});
```

### Event Broadcasting

Use channels to distribute events to multiple FSMs:

```rust
use std::sync::mpsc::channel;

let (tx, rx1) = channel();
let rx2 = tx.clone();

// Broadcast events to multiple FSMs
tx.send(Event::Tick).unwrap();
```

### Coordinated FSMs

Multiple FSMs can coordinate through shared state:

```rust
struct SharedState {
    lock_a: bool,
    lock_b: bool,
}

let shared = Arc::new(Mutex::new(SharedState {
    lock_a: false,
    lock_b: false
}));

// FSM A and FSM B coordinate via shared state
// See examples/traffic_intersection.rs for complete example
```

### Important: no_std Compatibility

The **core framework** remains `#![no_std]` compatible. Concurrency examples use `std::sync` and `std::thread`, but the generated FSM code has zero dependencies and works in no_std environments.

For embedded systems without std:
- Use `spin::Mutex` instead of `std::sync::Mutex`
- Use `alloc::sync::Arc` instead of `std::sync::Arc`
- Implement custom event distribution (e.g., interrupt-based)

## Using with Async Code

While typed-fsm does not have native async/await support in lifecycle hooks, it **can be used within async contexts**. The state machine methods are synchronous but can be called from async functions.

### Pattern: Async Wrapper

```rust
use typed_fsm::{state_machine, Transition};

// Standard synchronous FSM
state_machine! {
    Name: MyFSM,
    Context: MyContext,
    Event: MyEvent,

    States: {
        Active => {
            entry: |ctx| {
                ctx.status = "active".to_string();
            }

            process: |ctx, evt| {
                match evt {
                    MyEvent::Stop => Transition::To(MyFSM::Idle),
                    _ => Transition::None
                }
            }
        },

        Idle => {
            process: |ctx, evt| {
                match evt {
                    MyEvent::Start => Transition::To(MyFSM::Active),
                    _ => Transition::None
                }
            }
        }
    }
}

// Use within async context
async fn process_events(mut fsm: MyFSM, mut ctx: MyContext) {
    fsm.init(&mut ctx);

    loop {
        // Async operations between dispatches
        let event = receive_event_async().await;

        // Synchronous dispatch
        fsm.dispatch(&mut ctx, &event);

        // More async work
        if matches!(fsm, MyFSM::Active) {
            send_status_update(&ctx).await;
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
```

### Why No Native Async?

This is an intentional design decision to:
- **Maintain zero-cost abstraction** - Async has inherent overhead (futures, polling, state machines)
- **Preserve no_std compatibility** - Async requires a runtime (tokio, async-std)
- **Keep API simple** - Synchronous hooks are easier to reason about
- **Support embedded systems** - Many embedded environments don't use async

### When You Need Async Hooks

If your use case requires `async fn` in entry/exit/process hooks, consider:
- **statig** - Native hierarchical async state machines
- **smlang** - Async actions and guards built-in

These crates sacrifice zero-cost abstraction and require a runtime, but provide first-class async support.

## Guards (Conditional Transitions)

Guards are boolean conditions that must evaluate to true for a state transition to occur. They act as gatekeepers, validating data or checking preconditions before allowing state changes.

### What are Guards?

Guards allow you to implement conditional logic that determines whether an event should trigger a state transition. This is essential for:

- **Security checks** - PIN verification, authentication
- **Resource validation** - Check availability before allocation
- **Business rules** - Enforce constraints and policies
- **Data validation** - Verify input before accepting

### How Guards Work

Guards are implemented using standard Rust conditionals (`if/else`) within the `process` block. No special syntax is needed - just return `Transition::None` when the guard condition fails.

### Example: ATM PIN Verification

```rust
use typed_fsm::{state_machine, Transition};

struct ATMContext {
    correct_pin: u32,
    attempts: u32,
}

enum ATMEvent {
    EnterPIN { pin: u32 },
}

state_machine! {
    Name: ATM,
    Context: ATMContext,
    Event: ATMEvent,

    States: {
        WaitingPIN => {
            process: |ctx, evt| {
                match evt {
                    ATMEvent::EnterPIN { pin } => {
                        // Guard 1: Check if PIN is correct
                        if *pin == ctx.correct_pin {
                            println!("PIN accepted");
                            Transition::To(ATM::Authenticated)
                        } else {
                            ctx.attempts += 1;

                            // Guard 2: Block after 3 attempts
                            if ctx.attempts >= 3 {
                                Transition::To(ATM::Blocked)
                            } else {
                                Transition::None
                            }
                        }
                    }
                }
            }
        },

        Authenticated => {
            process: |_ctx, _evt| { Transition::None }
        },

        Blocked => {
            process: |_ctx, _evt| { Transition::None }
        }
    }
}
```

### Multiple Guard Conditions

Guards can combine multiple conditions:

```rust
process: |ctx, evt| {
    match evt {
        OrderEvent::Submit => {
            // Guard 1: Check stock
            if !ctx.items_in_stock {
                return Transition::None;
            }

            // Guard 2: Check credit limit
            if ctx.order_value > ctx.customer_credit {
                return Transition::None;
            }

            // All guards passed
            Transition::To(Order::Submitted)
        }
    }
}
```

### Guard Best Practices

1. **Early Returns** - Return immediately when guard fails for clarity
2. **Logging** - Log why guard failed for debugging
3. **Context Updates** - Update context before returning (e.g., increment attempt counter)
4. **Clear Messages** - Provide user feedback when guard blocks transition

### Complete Example

See [examples/guards.rs](examples/guards.rs) for a comprehensive example demonstrating:
- ATM security guards (PIN verification)
- Door lock access control
- Order processing business rules

Run with:
```bash
cargo run --example guards
```

## Timeouts (Time-Based Transitions)

typed-fsm supports time-based state transitions through the **Timer trait abstraction pattern**. This pattern maintains `no_std` compatibility while providing a flexible, platform-agnostic way to implement timeouts, retries, and time-based behaviors.

### What are Timeouts?

Timeouts allow states to automatically transition after a specified time duration. They're essential for:

- Connection timeouts
- Retry mechanisms with exponential backoff
- Button debouncing
- Watchdog timers in embedded systems
- Session expiration
- Idle detection

### Implementation Pattern

Unlike some FSM libraries that provide built-in timer functionality (which would break `no_std` compatibility), typed-fsm uses a **trait abstraction pattern**:

1. Define a `Timer` trait (user-provided or use the example)
2. Store timer instances in your Context
3. Check timeouts in your process blocks
4. Reset timers in entry/exit hooks as needed

This pattern is:
- **Zero-cost** - No overhead if you don't use it
- **no_std compatible** - Users implement for their platform
- **Completely optional** - Ignore if you don't need timeouts
- **Platform-agnostic** - Works with any time source

### Timer Trait

```rust
pub trait Timer {
    fn start(&mut self, duration_ms: u64);
    fn is_expired(&self) -> bool;
    fn reset(&mut self);
}
```

### Platform Implementations

**For std (Desktop/Server):**
```rust
use std::time::{Duration, Instant};

struct StdTimer {
    start_time: Option<Instant>,
    duration: Duration,
}

impl Timer for StdTimer {
    fn start(&mut self, duration_ms: u64) {
        self.start_time = Some(Instant::now());
        self.duration = Duration::from_millis(duration_ms);
    }

    fn is_expired(&self) -> bool {
        if let Some(start) = self.start_time {
            start.elapsed() >= self.duration
        } else {
            false
        }
    }

    fn reset(&mut self) {
        self.start_time = None;
    }
}
```

**For Embedded (no_std):**
```rust
// Example for embedded HAL timer
struct EmbeddedTimer<'a> {
    timer: &'a mut dyn embedded_hal::timer::CountDown,
    is_running: bool,
}

impl Timer for EmbeddedTimer<'_> {
    fn start(&mut self, duration_ms: u64) {
        self.timer.start(duration_ms.millis());
        self.is_running = true;
    }

    fn is_expired(&self) -> bool {
        self.is_running && self.timer.wait().is_ok()
    }

    fn reset(&mut self) {
        self.is_running = false;
    }
}
```

**For Testing (Mock):**
```rust
struct MockTimer {
    remaining_ms: u64,
}

impl Timer for MockTimer {
    fn start(&mut self, duration_ms: u64) {
        self.remaining_ms = duration_ms;
    }

    fn is_expired(&self) -> bool {
        self.remaining_ms == 0
    }

    fn reset(&mut self) {
        self.remaining_ms = 0;
    }
}

// In tests, manually decrement remaining_ms to simulate time
```

### Example: WiFi Connection with Timeout

```rust
use typed_fsm::{state_machine, Transition};

struct WiFiContext {
    timer: StdTimer,
    retry_count: u32,
    connection_timeout_ms: u64,
}

enum WiFiEvent {
    Connect,
    Connected,
    CheckTimeout,  // Polled event to check timeout
}

state_machine! {
    Name: WiFi,
    Context: WiFiContext,
    Event: WiFiEvent,

    States: {
        Connecting => {
            entry: |ctx| {
                println!("Connecting... (timeout: {}ms)", ctx.connection_timeout_ms);
                // Start timeout timer
                ctx.timer.start(ctx.connection_timeout_ms);
            }

            process: |ctx, evt| {
                match evt {
                    WiFiEvent::Connected => {
                        println!("Connected!");
                        ctx.timer.reset();
                        Transition::To(WiFi::Active)
                    }
                    WiFiEvent::CheckTimeout => {
                        // Check if timeout expired
                        if ctx.timer.is_expired() {
                            println!("Timeout!");
                            ctx.timer.reset();
                            Transition::To(WiFi::Failed)
                        } else {
                            Transition::None
                        }
                    }
                    _ => Transition::None
                }
            }

            exit: |ctx| {
                ctx.timer.reset();
            }
        },

        Active => { /* ... */ },
        Failed => { /* ... */ }
    }
}
```

### Usage Pattern

```rust
let mut ctx = WiFiContext {
    timer: StdTimer::new(),
    retry_count: 0,
    connection_timeout_ms: 5000,
};

let mut wifi = WiFi::Connecting;
wifi.init(&mut ctx);

// In your event loop:
loop {
    // Poll for events
    if let Some(event) = get_event() {
        wifi.dispatch(&mut ctx, &event);
    }

    // Periodically check for timeouts
    wifi.dispatch(&mut ctx, &WiFiEvent::CheckTimeout);

    thread::sleep(Duration::from_millis(100));
}
```

### Best Practices

1. **Store timers in Context** - Not in state variants (they get moved during transitions)
2. **Use polling pattern** - Check timeouts via a `CheckTimeout` event in your event loop
3. **Reset on exit** - Always reset timers in exit hooks to prevent stale timeouts
4. **Start on entry** - Initialize timers when entering the state that needs them
5. **Platform abstraction** - Implement Timer trait for your specific platform

### Complete Example

See [examples/timeouts.rs](examples/timeouts.rs) for comprehensive examples demonstrating:
- WiFi connection with timeout and retry logic
- Session timeout with idle detection
- Button debouncing with time delays

Run with:
```bash
cargo run --example timeouts
```

## Testing

This library has comprehensive test coverage (~100%) with **79 tests** covering:

- **Unit tests** (3 tests) - Core `Transition` enum functionality
- **Coverage tests** (10 tests) - All lifecycle hooks, optional entry/exit, self-transitions, multi-field states
- **Edge case tests** (8 tests) - Early returns, nested patterns, wildcard matches, if-let patterns
- **Integration tests** (13 tests) - Complete FSM scenarios (toggle, counter, resources, concurrency)
- **Guards tests** (14 tests) - Conditional transitions (PIN verification, multiple guards, range checks)
- **Logging tests** (9 tests) - Zero-cost abstraction, lifecycle hooks, self-transitions
- **Timeouts tests** (11 tests) - Timer trait pattern, retry logic, session timeouts
- **Doc tests** (11 tests) - All documentation examples are tested

Run all tests:

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run a specific test suite
cargo test --test integration_tests  # 13 tests
cargo test --test coverage_tests     # 10 tests
cargo test --test edge_cases_tests   # 8 tests
cargo test --test guards_tests       # 14 tests (v0.3.0)
cargo test --test logging_tests      # 9 tests (v0.3.0)
cargo test --test timeouts_tests     # 11 tests (v0.3.0)
```

## Examples

Run the included examples (ordered by complexity):

```bash
# Simplest example: LED blink (start here!)
cargo run --example blink

# Traffic light controller
cargo run --example traffic_light

# Hierarchical state machine: Audio player with nested volume control
cargo run --example hierarchical

# Concurrent state machines: Traffic intersection with synchronized lights
cargo run --example traffic_intersection

# Motor control system with safety checks
cargo run --example motor
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

## FAQ

### General Questions

**Q: Can I use typed-fsm in no_std environments?**
A: Yes! typed-fsm is `#![no_std]` compatible and has zero dependencies, making it perfect for embedded systems.

**Q: What's the performance overhead?**
A: Zero! The macro compiles to simple enum pattern matching. State transitions are just enum assignments. No heap allocations, no dynamic dispatch.

**Q: Can states hold data?**
A: Yes! States can carry typed fields, for example: `Running { speed: u32, mode: Mode }`.

**Q: How does this compare to manual enum matching?**
A: It generates the same code you would write manually, but with better organization, lifecycle hooks, and less boilerplate.

### Technical Questions

**Q: Does this work with async/await?**
A: Yes! The context and events can be async-friendly. The state machine itself is synchronous, but you can use async operations in your entry/exit/process handlers.

**Q: Can I serialize the state machine?**
A: The generated enum can derive `Serialize`/`Deserialize` if you enable the `serde` feature (future enhancement).

**Q: How do I handle errors in state transitions?**
A: You can include error information in events or state data. For example: `Error { code: u32, message: String }`.

**Q: Can I have nested state machines?**
A: Yes! A state's context can contain another state machine. This allows hierarchical state machines.

### Safety Questions

**Q: Is this library safe?**
A: Yes! The library contains zero `unsafe` blocks and has zero dependencies. It's been thoroughly tested with 30+ tests covering 100% of code paths.

**Q: Can invalid states occur?**
A: No! Rust's type system prevents invalid states at compile time. If it compiles, the state transitions are valid.

**Q: Is this production-ready?**
A: Yes! The library has comprehensive tests, documentation, and follows Rust best practices. It's ready for production use.

## Documentation

Full API documentation is available at [docs.rs/typed-fsm](https://docs.rs/typed-fsm).

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
