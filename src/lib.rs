//! # typed-fsm: Event-Driven Finite State Machine Microframework
//!
//! A lightweight, zero-cost, **event-driven** FSM generator for Rust using macros.
//! Designed for embedded systems (no-std compatible) and high-performance applications.
//!
//! ## Design Philosophy
//!
//! - **Event-Driven Architecture:** Built from the ground up for event-based systems
//! - **Zero Allocations:** Uses `enums` and static dispatch. No `Box`, `dyn`, or heap allocations.
//! - **Memory Safety:** Leverages Rust's type system to prevent invalid state transitions.
//! - **Compile-Time Guarantees:** All state logic is validated at compile time.
//! - **Performance:** Optimized for embedded and performance-critical applications.
//!
//! ## Quick Start
//!
//! ```rust
//! use typed_fsm::{state_machine, Transition};
//!
//! // Define your context (shared state)
//! struct LightContext {
//!     brightness: u8,
//! }
//!
//! // Define your events
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
//! fsm.init(&mut ctx);
//!
//! fsm.dispatch(&mut ctx, &LightEvent::TurnOn);
//! assert_eq!(ctx.brightness, 100);
//! ```
//!
//! ## Features
//!
//! - **Stateful States:** States can carry data (e.g., `Running { speed: u32 }`)
//! - **Lifecycle Hooks:** Define `entry`, `process`, and `exit` actions for each state
//! - **Type-Safe Transitions:** Compile-time validation of state transitions
//! - **Zero Runtime Overhead:** Compiles to efficient jump tables
//!
//! ## Examples
//!
//! See the `examples/` directory for complete examples:
//! - `motor.rs` - Motor control system with safety checks

#![no_std]

// The state_machine! macro is automatically available at the crate root
// due to #[macro_export] in fsm.rs
mod fsm;

// Re-export the core types
pub use fsm::Transition;
