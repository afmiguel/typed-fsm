//! # Logging Example: Automatic State Machine Instrumentation
//!
//! This example demonstrates the optional **logging** feature that automatically
//! instruments state machines with log output for debugging and monitoring.
//!
//! ## What is Logging?
//!
//! When enabled via feature flags, typed-fsm automatically logs:
//! - State machine initialization
//! - State entry actions
//! - State exit actions
//! - State transitions with events
//! - Events that don't trigger transitions
//!
//! ## How to Enable
//!
//! Add the `logging` feature to your Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! typed-fsm = { version = "0.3", features = ["logging"] }
//! log = "0.4"
//! ```
//!
//! Run this example with:
//! ```bash
//! cargo run --example logging --features logging
//! ```
//!
//! Without the feature flag, no logging code is compiled (zero-cost).

use typed_fsm::{state_machine, Transition};

// Initialize logger (env_logger in this example)
fn init_logger() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp_millis()
        .init();
}

// ============================================================================
// Example: Payment Processing FSM
// ============================================================================

struct PaymentContext {
    amount: f32,
    transaction_id: String,
}

#[derive(Debug, Clone)]
enum PaymentEvent {
    Process,
    Approve,
    Reject,
    Timeout,
}

state_machine! {
    Name: Payment,
    Context: PaymentContext,
    Event: PaymentEvent,

    States: {
        Pending => {
            entry: |ctx| {
                println!("  User: Initiating payment of ${:.2}", ctx.amount);
            }

            process: |_ctx, evt| {
                match evt {
                    PaymentEvent::Process => Transition::To(Payment::Processing),
                    _ => Transition::None
                }
            }
        },

        Processing => {
            entry: |ctx| {
                println!("  User: Processing transaction {}", ctx.transaction_id);
            }

            process: |_ctx, evt| {
                match evt {
                    PaymentEvent::Approve => Transition::To(Payment::Approved),
                    PaymentEvent::Reject => Transition::To(Payment::Rejected),
                    PaymentEvent::Timeout => Transition::To(Payment::Failed),
                    _ => Transition::None
                }
            }

            exit: |_ctx| {
                println!("  User: Transaction processing completed");
            }
        },

        Approved => {
            entry: |ctx| {
                println!("  User: Payment approved! ID: {}", ctx.transaction_id);
            }

            process: |_ctx, _evt| {
                Transition::None
            }
        },

        Rejected => {
            entry: |_ctx| {
                println!("  User: Payment rejected by bank");
            }

            process: |_ctx, _evt| {
                Transition::None
            }
        },

        Failed => {
            entry: |_ctx| {
                println!("  User: Payment failed (timeout)");
            }

            process: |_ctx, _evt| {
                Transition::None
            }
        }
    }
}

// ============================================================================
// Main: Demonstrates logging output
// ============================================================================

fn main() {
    println!("=== Logging Example: Automatic FSM Instrumentation ===\n");

    // Initialize logging (required for 'log' crate)
    init_logger();

    println!("Legend:");
    println!("  [Payment] = FSM log output (from typed-fsm)");
    println!("  User:     = Application log output\n");

    println!("--- Scenario 1: Successful Payment ---\n");
    run_successful_payment();

    println!("\n--- Scenario 2: Rejected Payment ---\n");
    run_rejected_payment();

    println!("\n--- Scenario 3: Timeout ---\n");
    run_timeout_payment();

    println!("\n=== Key Takeaways ===");
    println!("1. Logging is enabled via feature flags (zero-cost when disabled)");
    println!("2. All state transitions are logged automatically");
    println!("3. Log format: [FSM] State + Event -> NewState");
    println!("4. Compatible with 'log' and 'tracing' crates");
    println!("5. Useful for debugging, monitoring, and audit trails");
    println!("\nTo see logs, run with:");
    println!("  cargo run --example logging --features logging");
}

fn run_successful_payment() {
    let mut ctx = PaymentContext {
        amount: 99.99,
        transaction_id: "TXN-001".to_string(),
    };

    let mut payment = Payment::Pending;
    payment.init(&mut ctx);

    payment.dispatch(&mut ctx, &PaymentEvent::Process);
    payment.dispatch(&mut ctx, &PaymentEvent::Approve);
}

fn run_rejected_payment() {
    let mut ctx = PaymentContext {
        amount: 1500.00,
        transaction_id: "TXN-002".to_string(),
    };

    let mut payment = Payment::Pending;
    payment.init(&mut ctx);

    payment.dispatch(&mut ctx, &PaymentEvent::Process);
    payment.dispatch(&mut ctx, &PaymentEvent::Reject);
}

fn run_timeout_payment() {
    let mut ctx = PaymentContext {
        amount: 50.00,
        transaction_id: "TXN-003".to_string(),
    };

    let mut payment = Payment::Pending;
    payment.init(&mut ctx);

    payment.dispatch(&mut ctx, &PaymentEvent::Process);
    payment.dispatch(&mut ctx, &PaymentEvent::Timeout);
}
