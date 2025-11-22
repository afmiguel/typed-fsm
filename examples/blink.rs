//! # Blink Example - The Simplest State Machine
//!
//! This example demonstrates the absolute minimum needed to create
//! a working state machine with typed-fsm. It models a simple LED
//! that alternates between On and Off states.
//!
//! **Key Learning Points:**
//! - Minimal 2-state FSM (On ↔ Off)
//! - Entry actions for state transitions
//! - Event-driven transitions
//! - **CRITICAL:** Must call `.init()` before event loop!
//!
//! Run with: `cargo run --example blink`

use typed_fsm::{state_machine, Transition};

// Context: Represents the LED hardware and tick counter
struct LedContext {
    tick_count: u32,
}

// Event: Simple tick event to trigger state changes
enum Event {
    Tick,
}

// Define the state machine with two states: On and Off
state_machine! {
    Name: BlinkFSM,
    Context: LedContext,
    Event: Event,

    States: {
        On => {
            entry: |ctx| {
                ctx.tick_count += 1;
                println!("💡 LED ON  (tick {})", ctx.tick_count);
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
                println!("   LED OFF (tick {})", ctx.tick_count);
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
    println!("=== Blink Example: Simplest State Machine ===\n");

    // Create context and state machine
    let mut ctx = LedContext { tick_count: 0 };
    let mut led = BlinkFSM::On;

    // ⚠️ CRITICAL: Must call init() before the event loop!
    // Without this, the entry action of the initial state (On) will NOT execute!
    led.init(&mut ctx);

    println!("\nStarting blink sequence...\n");

    // Event loop: Send 10 tick events
    for _ in 0..10 {
        led.dispatch(&mut ctx, &Event::Tick);
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    println!("\n=== Blink sequence complete ===");
    println!("Total ticks: {}", ctx.tick_count);
}
