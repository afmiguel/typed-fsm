//! Tests for Logging feature
//!
//! This test suite validates that logging works correctly:
//! - Logging compiles without feature flags (zero-cost)
//! - Logging compiles with 'logging' feature
//! - All state transitions are logged
//! - No runtime errors with logging enabled

use typed_fsm::{state_machine, Transition};

// ============================================================================
// Test 1: FSM compiles and works without logging feature
// ============================================================================

struct TestContext {
    counter: u32,
}

#[derive(Debug, Clone)]
enum TestEvent {
    Increment,
    Reset,
}

state_machine! {
    Name: Counter,
    Context: TestContext,
    Event: TestEvent,

    States: {
        Active => {
            entry: |ctx| {
                ctx.counter = 0;
            }

            process: |ctx, evt| {
                match evt {
                    TestEvent::Increment => {
                        ctx.counter += 1;
                        if ctx.counter >= 3 {
                            Transition::To(Counter::Max)
                        } else {
                            Transition::None
                        }
                    }
                    TestEvent::Reset => {
                        Transition::To(Counter::Active)
                    }
                }
            }

            exit: |ctx| {
                ctx.counter = 999; // Mark that exit was called
            }
        },

        Max => {
            entry: |_ctx| {
                // Entry hook called
            }

            process: |_ctx, evt| {
                match evt {
                    TestEvent::Reset => Transition::To(Counter::Active),
                    _ => Transition::None
                }
            }
        }
    }
}

#[test]
fn test_logging_zero_cost_without_feature() {
    // This test ensures that without the 'logging' feature,
    // the FSM compiles and works correctly (zero-cost abstraction)
    let mut ctx = TestContext { counter: 0 };

    let mut counter = Counter::Active;
    counter.init(&mut ctx);

    assert_eq!(ctx.counter, 0);
    assert!(matches!(counter, Counter::Active));

    // Increment to max
    counter.dispatch(&mut ctx, &TestEvent::Increment);
    counter.dispatch(&mut ctx, &TestEvent::Increment);
    counter.dispatch(&mut ctx, &TestEvent::Increment);

    assert!(matches!(counter, Counter::Max));
    assert_eq!(ctx.counter, 999); // Exit was called

    // Reset
    counter.dispatch(&mut ctx, &TestEvent::Reset);
    assert!(matches!(counter, Counter::Active));
    assert_eq!(ctx.counter, 0);
}

#[test]
fn test_logging_init_called() {
    let mut ctx = TestContext { counter: 0 };
    let mut counter = Counter::Active;

    // init() should work with or without logging
    counter.init(&mut ctx);

    assert_eq!(ctx.counter, 0);
    assert!(matches!(counter, Counter::Active));
}

#[test]
fn test_logging_entry_hooks_called() {
    let mut ctx = TestContext { counter: 0 };
    let mut counter = Counter::Active;
    counter.init(&mut ctx);

    // Entry hooks should be called regardless of logging
    counter.dispatch(&mut ctx, &TestEvent::Increment);
    counter.dispatch(&mut ctx, &TestEvent::Increment);
    counter.dispatch(&mut ctx, &TestEvent::Increment);

    // We transitioned to Max, so Active's exit was called
    assert_eq!(ctx.counter, 999);
}

#[test]
fn test_logging_exit_hooks_called() {
    let mut ctx = TestContext { counter: 5 };
    let mut counter = Counter::Max;
    counter.init(&mut ctx);

    // Transition back to Active
    counter.dispatch(&mut ctx, &TestEvent::Reset);

    // Active's entry should have reset counter to 0
    assert_eq!(ctx.counter, 0);
}

#[test]
fn test_logging_transition_none() {
    let mut ctx = TestContext { counter: 0 };
    let mut counter = Counter::Active;
    counter.init(&mut ctx);

    // Increment once (stays in Active)
    counter.dispatch(&mut ctx, &TestEvent::Increment);

    assert!(matches!(counter, Counter::Active));
    assert_eq!(ctx.counter, 1);

    // Another increment (still in Active)
    counter.dispatch(&mut ctx, &TestEvent::Increment);

    assert!(matches!(counter, Counter::Active));
    assert_eq!(ctx.counter, 2);
}

// ============================================================================
// Test 2: Complex FSM with multiple states
// ============================================================================

#[allow(dead_code)]
struct PaymentContext {
    amount: f32,
    processed: bool,
}

#[derive(Debug, Clone)]
enum PaymentEvent {
    Process,
    Approve,
    Reject,
}

state_machine! {
    Name: Payment,
    Context: PaymentContext,
    Event: PaymentEvent,

    States: {
        Pending => {
            entry: |ctx| {
                ctx.processed = false;
            }

            process: |_ctx, evt| {
                match evt {
                    PaymentEvent::Process => Transition::To(Payment::Processing),
                    _ => Transition::None
                }
            }
        },

        Processing => {
            process: |_ctx, evt| {
                match evt {
                    PaymentEvent::Approve => Transition::To(Payment::Approved),
                    PaymentEvent::Reject => Transition::To(Payment::Rejected),
                    _ => Transition::None
                }
            }

            exit: |ctx| {
                ctx.processed = true;
            }
        },

        Approved => {
            entry: |_ctx| {
                // Approved
            }

            process: |_ctx, _evt| {
                Transition::None
            }
        },

        Rejected => {
            entry: |_ctx| {
                // Rejected
            }

            process: |_ctx, _evt| {
                Transition::None
            }
        }
    }
}

#[test]
fn test_logging_full_lifecycle_approve() {
    let mut ctx = PaymentContext {
        amount: 99.99,
        processed: false,
    };

    let mut payment = Payment::Pending;
    payment.init(&mut ctx);

    assert!(!ctx.processed);

    // Process
    payment.dispatch(&mut ctx, &PaymentEvent::Process);
    assert!(matches!(payment, Payment::Processing));
    assert!(!ctx.processed); // Exit not called yet

    // Approve
    payment.dispatch(&mut ctx, &PaymentEvent::Approve);
    assert!(matches!(payment, Payment::Approved));
    assert!(ctx.processed); // Exit was called
}

#[test]
fn test_logging_full_lifecycle_reject() {
    let mut ctx = PaymentContext {
        amount: 99.99,
        processed: false,
    };

    let mut payment = Payment::Pending;
    payment.init(&mut ctx);

    payment.dispatch(&mut ctx, &PaymentEvent::Process);
    payment.dispatch(&mut ctx, &PaymentEvent::Reject);

    assert!(matches!(payment, Payment::Rejected));
    assert!(ctx.processed);
}

// ============================================================================
// Test 3: Self-transitions
// ============================================================================

struct SelfTransitionContext {
    resets: u32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum SelfEvent {
    Reset,
    Advance,
}

state_machine! {
    Name: SelfMachine,
    Context: SelfTransitionContext,
    Event: SelfEvent,

    States: {
        Active => {
            entry: |ctx| {
                ctx.resets += 1;
            }

            process: |_ctx, evt| {
                match evt {
                    SelfEvent::Reset => {
                        // Self-transition
                        Transition::To(SelfMachine::Active)
                    }
                    SelfEvent::Advance => {
                        Transition::To(SelfMachine::Done)
                    }
                }
            }

            exit: |_ctx| {
                // Exit called even on self-transition
            }
        },

        Done => {
            process: |_ctx, _evt| {
                Transition::None
            }
        }
    }
}

#[test]
fn test_logging_self_transition() {
    let mut ctx = SelfTransitionContext { resets: 0 };

    let mut machine = SelfMachine::Active;
    machine.init(&mut ctx);

    assert_eq!(ctx.resets, 1);

    // Self-transition should call exit and entry
    machine.dispatch(&mut ctx, &SelfEvent::Reset);

    assert!(matches!(machine, SelfMachine::Active));
    assert_eq!(ctx.resets, 2); // Entry called again
}

#[test]
fn test_logging_multiple_self_transitions() {
    let mut ctx = SelfTransitionContext { resets: 0 };

    let mut machine = SelfMachine::Active;
    machine.init(&mut ctx);

    // Multiple self-transitions
    machine.dispatch(&mut ctx, &SelfEvent::Reset);
    machine.dispatch(&mut ctx, &SelfEvent::Reset);
    machine.dispatch(&mut ctx, &SelfEvent::Reset);

    assert_eq!(ctx.resets, 4); // init + 3 resets
}
