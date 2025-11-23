//! # typed-fsm: Event-Driven Finite State Machine Microframework
//!
//! A lightweight, zero-cost, **event-driven** FSM generator for Rust using macros.
//! Designed for embedded systems (no-std compatible) and high-performance applications.
//!
//! ## Features
//!
//! - **Event-Driven Architecture** - Built from the ground up for event-based systems
//! - **Zero-cost abstraction** - Compiles to efficient jump tables with no runtime overhead
//! - **Type-safe** - Compile-time validation of state transitions and events
//! - **No allocations** - Uses enums and static dispatch (no `Box`, `dyn`, or heap)
//! - **Embedded-ready** - `#![no_std]` compatible with zero dependencies
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
//! #[derive(Debug)]
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
//! #[derive(Debug)]
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
//! # #[derive(Debug)]
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
//! # #[derive(Debug)]
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
//! # #[derive(Debug)]
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
//! ## Examples
//!
//! See the `examples/` directory for complete examples:
//! - `blink.rs` - The simplest possible FSM (LED On/Off) - **start here!**
//! - `traffic_light.rs` - Traffic light controller with timing
//! - `hierarchical.rs` - Nested state machines (audio player with volume control)
//! - `traffic_intersection.rs` - Concurrent FSMs with thread synchronization
//! - `motor.rs` - Motor control system with safety checks and stateful states

#![no_std]

// The state_machine! macro is automatically available at the crate root
// due to #[macro_export] in fsm.rs
mod fsm;

// Re-export the core types
pub use fsm::Transition;
