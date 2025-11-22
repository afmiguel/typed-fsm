//! Integration tests for the finite_state_machine library

use typed_fsm::{state_machine, Transition};

// ============================================================================
// Test 1: Simple Toggle State Machine
// ============================================================================

#[derive(Debug)]
struct ToggleContext {
    toggle_count: u32,
}

#[derive(Debug)]
enum ToggleEvent {
    Toggle,
}

state_machine! {
    Name: ToggleFSM,
    Context: ToggleContext,
    Event: ToggleEvent,

    States: {
        Off => {
            entry: |ctx| {
                ctx.toggle_count += 1;
            }

            process: |_ctx, evt| {
                match evt {
                    ToggleEvent::Toggle => Transition::To(ToggleFSM::On),
                }
            }
        },

        On => {
            entry: |ctx| {
                ctx.toggle_count += 1;
            }

            process: |_ctx, evt| {
                match evt {
                    ToggleEvent::Toggle => Transition::To(ToggleFSM::Off),
                }
            }
        }
    }
}

#[test]
fn test_simple_toggle() {
    let mut ctx = ToggleContext { toggle_count: 0 };
    let mut fsm = ToggleFSM::Off;

    // Initialize should call entry
    fsm.init(&mut ctx);
    assert_eq!(ctx.toggle_count, 1);

    // Toggle to On
    fsm.dispatch(&mut ctx, &ToggleEvent::Toggle);
    assert_eq!(ctx.toggle_count, 2);

    // Toggle back to Off
    fsm.dispatch(&mut ctx, &ToggleEvent::Toggle);
    assert_eq!(ctx.toggle_count, 3);
}

// ============================================================================
// Test 2: State with Data
// ============================================================================

#[derive(Debug)]
struct CounterContext {
    value: i32,
}

#[derive(Debug)]
enum CounterEvent {
    Increment,
    Decrement,
    Reset,
}

state_machine! {
    Name: CounterFSM,
    Context: CounterContext,
    Event: CounterEvent,

    States: {
        Idle => {
            entry: |ctx| {
                ctx.value = 0;
            }

            process: |_ctx, evt| {
                match evt {
                    CounterEvent::Increment => Transition::To(CounterFSM::Active { step: 1 }),
                    CounterEvent::Decrement => Transition::To(CounterFSM::Active { step: -1 }),
                    CounterEvent::Reset => Transition::None
                }
            }
        },

        Active { step: i32 } => {
            entry: |ctx| {
                ctx.value += *step;
            }

            process: |_ctx, evt| {
                match evt {
                    CounterEvent::Reset => Transition::To(CounterFSM::Idle),
                    CounterEvent::Increment => Transition::To(CounterFSM::Active { step: 1 }),
                    CounterEvent::Decrement => Transition::To(CounterFSM::Active { step: -1 }),
                }
            }
        }
    }
}

#[test]
fn test_stateful_counter() {
    let mut ctx = CounterContext { value: 0 };
    let mut fsm = CounterFSM::Idle;

    fsm.init(&mut ctx);
    assert_eq!(ctx.value, 0);

    // Increment
    fsm.dispatch(&mut ctx, &CounterEvent::Increment);
    assert_eq!(ctx.value, 1);

    // Increment again
    fsm.dispatch(&mut ctx, &CounterEvent::Increment);
    assert_eq!(ctx.value, 2);

    // Decrement
    fsm.dispatch(&mut ctx, &CounterEvent::Decrement);
    assert_eq!(ctx.value, 1);

    // Reset
    fsm.dispatch(&mut ctx, &CounterEvent::Reset);
    assert_eq!(ctx.value, 0);
}

// ============================================================================
// Test 3: Exit Actions
// ============================================================================

#[derive(Debug)]
struct ResourceContext {
    acquired: bool,
    released: bool,
}

#[derive(Debug)]
enum ResourceEvent {
    Acquire,
    Release,
}

state_machine! {
    Name: ResourceFSM,
    Context: ResourceContext,
    Event: ResourceEvent,

    States: {
        Free => {
            process: |_ctx, evt| {
                match evt {
                    ResourceEvent::Acquire => Transition::To(ResourceFSM::Locked),
                    _ => Transition::None
                }
            }
        },

        Locked => {
            entry: |ctx| {
                ctx.acquired = true;
            }

            process: |_ctx, evt| {
                match evt {
                    ResourceEvent::Release => Transition::To(ResourceFSM::Free),
                    _ => Transition::None
                }
            }

            exit: |ctx| {
                ctx.released = true;
            }
        }
    }
}

#[test]
fn test_exit_actions() {
    let mut ctx = ResourceContext {
        acquired: false,
        released: false,
    };
    let mut fsm = ResourceFSM::Free;

    fsm.init(&mut ctx);

    // Acquire resource
    fsm.dispatch(&mut ctx, &ResourceEvent::Acquire);
    assert!(ctx.acquired);
    assert!(!ctx.released);

    // Release resource
    fsm.dispatch(&mut ctx, &ResourceEvent::Release);
    assert!(ctx.acquired);
    assert!(ctx.released); // Exit action was called
}

// ============================================================================
// Test 4: No State Change
// ============================================================================

#[derive(Debug)]
struct NoopContext {
    event_count: u32,
}

#[derive(Debug)]
enum NoopEvent {
    DoNothing,
}

state_machine! {
    Name: NoopFSM,
    Context: NoopContext,
    Event: NoopEvent,

    States: {
        Idle => {
            process: |ctx, _evt| {
                ctx.event_count += 1;
                Transition::None
            }
        }
    }
}

#[test]
fn test_no_state_change() {
    let mut ctx = NoopContext { event_count: 0 };
    let mut fsm = NoopFSM::Idle;

    fsm.init(&mut ctx);

    // Process events that don't change state
    fsm.dispatch(&mut ctx, &NoopEvent::DoNothing);
    assert_eq!(ctx.event_count, 1);

    fsm.dispatch(&mut ctx, &NoopEvent::DoNothing);
    assert_eq!(ctx.event_count, 2);

    fsm.dispatch(&mut ctx, &NoopEvent::DoNothing);
    assert_eq!(ctx.event_count, 3);
}

// ============================================================================
// Test 5: Multiple Fields in State
// ============================================================================

#[derive(Debug)]
struct MultiFieldContext {
    sum: i32,
}

#[derive(Debug)]
enum MathEvent {
    Add(i32, i32),
    Done,
}

state_machine! {
    Name: MathFSM,
    Context: MultiFieldContext,
    Event: MathEvent,

    States: {
        Idle => {
            process: |_ctx, evt| {
                match evt {
                    MathEvent::Add(a, b) => Transition::To(MathFSM::Computing { x: *a, y: *b }),
                    _ => Transition::None
                }
            }
        },

        Computing { x: i32, y: i32 } => {
            entry: |ctx| {
                ctx.sum = *x + *y;
            }

            process: |_ctx, evt| {
                match evt {
                    MathEvent::Done => Transition::To(MathFSM::Idle),
                    MathEvent::Add(a, b) => Transition::To(MathFSM::Computing { x: *a, y: *b }),
                }
            }
        }
    }
}

#[test]
fn test_multiple_state_fields() {
    let mut ctx = MultiFieldContext { sum: 0 };
    let mut fsm = MathFSM::Idle;

    fsm.init(&mut ctx);

    // Add 10 + 20
    fsm.dispatch(&mut ctx, &MathEvent::Add(10, 20));
    assert_eq!(ctx.sum, 30);

    // Add 5 + 7
    fsm.dispatch(&mut ctx, &MathEvent::Add(5, 7));
    assert_eq!(ctx.sum, 12);

    // Done
    fsm.dispatch(&mut ctx, &MathEvent::Done);
    assert_eq!(ctx.sum, 12); // No change in sum
}
