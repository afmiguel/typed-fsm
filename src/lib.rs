//! # typed-fsm: Event-Driven Finite State Machine Microframework
//!
//! A lightweight, zero-cost, **event-driven** FSM generator for Rust with **ISR and concurrency support**.
//! Designed for embedded systems (no-std compatible) and high-performance applications.
//!
//! ## Features
//!
//! - **Event-Driven Architecture** - Built from the ground up for event-based systems
//! - **ISR-Safe Dispatch** - Call `dispatch()` from interrupt service routines (optional `concurrent` feature)
//! - **Thread-Safe Concurrency** - Safe concurrent access from multiple threads with atomic protection
//! - **Zero-cost abstraction** - Compiles to efficient jump tables with no runtime overhead
//! - **Type-safe** - Compile-time validation of state transitions and events
//! - **No allocations** - Uses enums and static dispatch (no `Box`, `dyn`, or heap)
//! - **Embedded-ready** - `#![no_std]` compatible with zero dependencies by default
//! - **Stateful states** - States can carry typed data
//! - **Lifecycle hooks** - `entry`, `process`, and `exit` actions per state
//!
//! ## Quick Start: Simplest Example (Blink)
//!
//! The simplest state machine alternates between two states:
//!
//! ```rust
//! use typed_fsm::{state_machine, Transition};
//!
//! // Context: Shared state across all states
//! struct LedContext {
//!     tick_count: u32,
//! }
//!
//! // Event: Simple tick event
//! #[derive(Debug, Clone)]
//! enum Event {
//!     Tick,
//! }
//!
//! // State machine with two states: On and Off
//! state_machine! {
//!     Name: BlinkFSM,
//!     Context: LedContext,
//!     Event: Event,
//!
//!     States: {
//!         On => {
//!             entry: |ctx| {
//!                 ctx.tick_count += 1;
//!             }
//!
//!             process: |_ctx, event| {
//!                 match event {
//!                     Event::Tick => Transition::To(BlinkFSM::Off),
//!                 }
//!             }
//!         },
//!
//!         Off => {
//!             entry: |ctx| {
//!                 ctx.tick_count += 1;
//!             }
//!
//!             process: |_ctx, event| {
//!                 match event {
//!                     Event::Tick => Transition::To(BlinkFSM::On),
//!                 }
//!             }
//!         }
//!     }
//! }
//!
//! // Usage
//! let mut ctx = LedContext { tick_count: 0 };
//! let mut led = BlinkFSM::On;
//!
//! // ⚠️ CRITICAL: Must call init() before event loop!
//! led.init(&mut ctx);
//!
//! // Dispatch events
//! led.dispatch(&mut ctx, &Event::Tick);  // On → Off
//! led.dispatch(&mut ctx, &Event::Tick);  // Off → On
//! assert_eq!(ctx.tick_count, 3); // Initial entry + 2 transitions
//! ```
//!
//! ## More Complex Example: Light with Brightness
//!
//! States can handle multiple events and modify context:
//!
//! ```rust
//! # use typed_fsm::{state_machine, Transition};
//! // Define your context (shared state)
//! struct LightContext {
//!     brightness: u8,
//! }
//!
//! // Define your events
//! #[derive(Debug, Clone)]
//! enum LightEvent {
//!     TurnOn,
//!     TurnOff,
//! }
//!
//! // Create your state machine
//! state_machine! {
//!     Name: LightFSM,
//!     Context: LightContext,
//!     Event: LightEvent,
//!
//!     States: {
//!         Off => {
//!             entry: |ctx| {
//!                 ctx.brightness = 0;
//!             }
//!
//!             process: |_ctx, evt| {
//!                 match evt {
//!                     LightEvent::TurnOn => Transition::To(LightFSM::On),
//!                     _ => Transition::None
//!                 }
//!             }
//!         },
//!
//!         On => {
//!             entry: |ctx| {
//!                 ctx.brightness = 100;
//!             }
//!
//!             process: |_ctx, evt| {
//!                 match evt {
//!                     LightEvent::TurnOff => Transition::To(LightFSM::Off),
//!                     _ => Transition::None
//!                 }
//!             }
//!         }
//!     }
//! }
//!
//! // Use the state machine
//! let mut ctx = LightContext { brightness: 0 };
//! let mut fsm = LightFSM::Off;
//!
//! // ⚠️ CRITICAL: Always call init() before dispatching events!
//! fsm.init(&mut ctx);
//!
//! fsm.dispatch(&mut ctx, &LightEvent::TurnOn);
//! assert_eq!(ctx.brightness, 100);
//! ```
//!
//! ## Understanding Transitions
//!
//! The `process` hook **must return** a `Transition` enum to tell the state machine what to do:
//!
//! ### `Transition::None` - Stay in Current State
//!
//! Use when an event should be handled but doesn't change the state:
//!
//! ```rust
//! # use typed_fsm::{state_machine, Transition};
//! # struct Context { data: u32 }
//! # #[derive(Debug, Clone)]
//! # enum Event { Update(u32), Ignore }
//! # state_machine! {
//! #     Name: FSM,
//! #     Context: Context,
//! #     Event: Event,
//! #     States: {
//! #         Active => {
//! process: |ctx, evt| {
//!     match evt {
//!         Event::Update(value) => {
//!             ctx.data = *value;  // Update context
//!             Transition::None    // Stay in same state
//!         },
//!         Event::Ignore => Transition::None
//!     }
//! }
//! #         }
//! #     }
//! # }
//! ```
//!
//! **When `Transition::None` is returned:**
//! - ✅ `process` executes
//! - ❌ `exit` does NOT execute (no state change)
//! - ❌ `entry` does NOT execute (no state change)
//! - ✅ State remains unchanged
//!
//! ### `Transition::To(State)` - Move to New State
//!
//! Use when an event should trigger a state change:
//!
//! ```rust
//! # use typed_fsm::{state_machine, Transition};
//! # struct Context { }
//! # #[derive(Debug, Clone)]
//! # enum Event { Start, Stop }
//! # state_machine! {
//! #     Name: Machine,
//! #     Context: Context,
//! #     Event: Event,
//! #     States: {
//! #         Idle => {
//! process: |ctx, evt| {
//!     match evt {
//!         Event::Start => {
//!             Transition::To(Machine::Running { speed: 100 })
//!         },
//!         Event::Stop => Transition::None
//!     }
//! }
//! #         },
//! #         Running { speed: u32 } => {
//! #             process: |ctx, evt| { Transition::None }
//! #         }
//! #     }
//! # }
//! ```
//!
//! **When `Transition::To(State)` is returned:**
//! 1. ✅ `process` executes and returns new state
//! 2. ✅ Current state's `exit` executes (if defined)
//! 3. ✅ New state's `entry` executes (if defined)
//! 4. ✅ State updates to the new state
//!
//! **Key Points:**
//! - Every `process` block **must** return a `Transition`
//! - Use `Transition::None` for events that don't change state
//! - Use `Transition::To(State)` for events that trigger transitions
//! - You can update context in `process` before returning
//! - The transition type determines whether `exit`/`entry` hooks run
//!
//! ## Thread Safety and Concurrency
//!
//! FSMs are automatically `Send + Sync` if their fields are `Send + Sync`.
//! This enables safe concurrent usage through Rust's standard concurrency primitives.
//!
//! ### Arc<Mutex<>> Pattern
//!
//! ```rust,no_run
//! # use typed_fsm::{state_machine, Transition};
//! # use std::sync::{Arc, Mutex};
//! # use std::thread;
//! # struct Context { }
//! # #[derive(Debug, Clone)]
//! # enum Event { Tick }
//! # state_machine! {
//! #     Name: FSM,
//! #     Context: Context,
//! #     Event: Event,
//! #     States: {
//! #         Active => {
//! #             process: |ctx, evt| { Transition::None }
//! #         }
//! #     }
//! # }
//! let fsm = Arc::new(Mutex::new(FSM::Active));
//! let ctx = Arc::new(Mutex::new(Context { }));
//!
//! let fsm_clone = Arc::clone(&fsm);
//! let ctx_clone = Arc::clone(&ctx);
//!
//! thread::spawn(move || {
//!     let mut fsm = fsm_clone.lock().unwrap();
//!     let mut ctx = ctx_clone.lock().unwrap();
//!     fsm.dispatch(&mut *ctx, &Event::Tick);
//! });
//! ```
//!
//! See `examples/traffic_intersection.rs` for a complete concurrent FSM example.
//!
//! **Note:** The core framework is `#![no_std]` compatible. Concurrency examples
//! use std, but FSMs work in no_std environments with alternatives like `spin::Mutex`.
//!
//! ## ISR and Multithreading Safety (Feature: `concurrent`)
//!
//! For **interrupt service routines (ISRs)** and true **concurrent multithreading**,
//! enable the optional `concurrent` feature. This adds protection against re-entrant
//! dispatch calls using atomic operations and lock-free queues.
//!
//! This feature supports all architectures (including AVR and ARM Cortex-M) by automatically
//! adapting to the target platform via the `portable-atomic` crate.
//!
//! ### When to Use
//!
//! Enable `concurrent` when:
//! - **ISRs call `dispatch()`**: Interrupt handlers need to generate events
//! - **Multiple threads call `dispatch()`**: Concurrent access from different threads
//! - **ISRs + Threads**: Combined scenario (e.g., RTOS environments)
//!
//! ### How It Works
//!
//! 1. **Immediate execution**: If no dispatch is active, executes immediately
//! 2. **Queue if busy**: If dispatch is already active, event is queued (capacity: 16 events)
//! 3. **FIFO processing**: Queued events are processed in order before releasing lock
//! 4. **Atomic protection**: Uses `portable_atomic::AtomicBool` with compare-exchange and `critical_section::Mutex`
//!
//! ### Requirements
//!
//! - **Event type must be `Clone`**: Events are cloned when enqueued
//! - **critical-section implementation**: Requires a `critical-section` provider for your platform
//!   - For `std`: Use `critical-section = { version = "1.1", features = ["std"] }`
//!   - For embedded: Use your HAL's critical-section implementation
//!
//! ### Important Limitations
//!
//! - **Queue capacity**: Fixed at 16 events. Events are silently dropped when queue is full.
//! - **Shared statics**: All FSMs of the same type share global static variables (lock + queue).
//!   This is normally not an issue as each FSM type has a unique name.
//!
//! ### Usage
//!
//! ```toml
//! [dependencies]
//! typed-fsm = { version = "0.4", features = ["concurrent"] }
//! ```
//!
//! ### Complete Examples
//!
//! - `examples/concurrent_isr.rs` - Simulated ISR with event queuing
//! - `examples/concurrent_threads.rs` - Multithreading with concurrent dispatch//!
//! **Performance:** ~10-15% overhead when enabled, zero overhead when disabled.
//!
//! ## Examples
//!
//! See the `examples/` directory for complete examples:
//! - \*\*\[New\]\*\* Raspberry Pi Pico 2 W Demo: \[typed-fsm-pico-test\](<https://github.com/afmiguel/typed-fsm-pico-test>) - Real-world usage on RP2350 interacting with Hardware (LED, ADC, Timer).
//! - \*\*\[New\]\*\* Arduino Uno Demo: \[typed-fsm-arduino-test\](<https://github.com/afmiguel/typed-fsm-arduino-test>) - Real-world usage on ATmega328P (AVR) with concurrent ISRs.
//! - `motor.rs` - Motor control (complex, event-driven) - **start here!**
//! - `traffic_light.rs` - Traffic light controller (simple, event-driven)
//! - `guards.rs` - Conditional transitions (ATM, door lock, orders)
//! - `logging.rs` - FSM with instrumentation
//! - `timeouts.rs` - Timer pattern (WiFi, session, debouncing)
//! - `concurrent_isr.rs` - ISR-safe dispatch (requires `concurrent` feature)
//! - `concurrent_threads.rs` - Thread-safe dispatch (requires `concurrent` feature)

#![no_std]

// The state_machine! macro is automatically available at the crate root
// due to #[macro_export] in fsm.rs
mod fsm;

// Re-export the core types
pub use fsm::Transition;
