//! Comprehensive coverage tests for the finite_state_machine library
//!
//! These tests aim to achieve near 100% code coverage by testing all possible
//! paths through the macro-generated code.

use typed_fsm::{state_machine, Transition};

// ============================================================================
// Test 1: State with all hooks (entry, process, exit)
// ============================================================================

#[derive(Debug, Clone)]
struct AllHooksContext {
    entry_called: bool,
    process_called: bool,
    exit_called: bool,
}

#[derive(Debug, Clone)]
enum AllHooksEvent {
    Next,
}

state_machine! {
    Name: AllHooksFSM,
    Context: AllHooksContext,
    Event: AllHooksEvent,

    States: {
        First => {
            entry: |ctx| {
                ctx.entry_called = true;
            }

            process: |ctx, _evt| {
                ctx.process_called = true;
                Transition::To(AllHooksFSM::Second)
            }

            exit: |ctx| {
                ctx.exit_called = true;
            }
        },

        Second => {
            process: |_ctx, _evt| {
                Transition::None
            }
        }
    }
}

#[test]
fn test_all_hooks_called_in_order() {
    let mut ctx = AllHooksContext {
        entry_called: false,
        process_called: false,
        exit_called: false,
    };
    let mut fsm = AllHooksFSM::First;

    // Init should call entry
    fsm.init(&mut ctx);
    assert!(ctx.entry_called);
    assert!(!ctx.process_called);
    assert!(!ctx.exit_called);

    // Reset flags
    ctx.entry_called = false;
    ctx.process_called = false;
    ctx.exit_called = false;

    // Dispatch should call process, then exit (old state), then entry (new state)
    fsm.dispatch(&mut ctx, &AllHooksEvent::Next);
    assert!(ctx.process_called);
    assert!(ctx.exit_called);
    // Note: Second state has no entry, so entry_called should still be false
    assert!(!ctx.entry_called);
}

// ============================================================================
// Test 2: State without entry hook
// ============================================================================

#[derive(Debug, Clone)]
struct NoEntryContext {
    counter: u32,
}

#[derive(Debug, Clone)]
enum NoEntryEvent {
    Increment,
}

state_machine! {
    Name: NoEntryFSM,
    Context: NoEntryContext,
    Event: NoEntryEvent,

    States: {
        Active => {
            process: |ctx, _evt| {
                ctx.counter += 1;
                Transition::None
            }
        }
    }
}

#[test]
fn test_state_without_entry() {
    let mut ctx = NoEntryContext { counter: 0 };
    let mut fsm = NoEntryFSM::Active;

    // Init should work even without entry hook
    fsm.init(&mut ctx);
    assert_eq!(ctx.counter, 0);

    // Process should still work
    fsm.dispatch(&mut ctx, &NoEntryEvent::Increment);
    assert_eq!(ctx.counter, 1);

    fsm.dispatch(&mut ctx, &NoEntryEvent::Increment);
    assert_eq!(ctx.counter, 2);
}

// ============================================================================
// Test 3: State without exit hook
// ============================================================================

#[derive(Debug, Clone)]
struct NoExitContext {
    transitions: u32,
}

#[derive(Debug, Clone)]
enum NoExitEvent {
    Switch,
}

state_machine! {
    Name: NoExitFSM,
    Context: NoExitContext,
    Event: NoExitEvent,

    States: {
        StateA => {
            entry: |ctx| {
                ctx.transitions += 1;
            }

            process: |_ctx, _evt| {
                Transition::To(NoExitFSM::StateB)
            }
        },

        StateB => {
            entry: |ctx| {
                ctx.transitions += 1;
            }

            process: |_ctx, _evt| {
                Transition::To(NoExitFSM::StateA)
            }
        }
    }
}

#[test]
fn test_state_without_exit() {
    let mut ctx = NoExitContext { transitions: 0 };
    let mut fsm = NoExitFSM::StateA;

    fsm.init(&mut ctx);
    assert_eq!(ctx.transitions, 1);

    // Transition to B
    fsm.dispatch(&mut ctx, &NoExitEvent::Switch);
    assert_eq!(ctx.transitions, 2);

    // Transition back to A
    fsm.dispatch(&mut ctx, &NoExitEvent::Switch);
    assert_eq!(ctx.transitions, 3);
}

// ============================================================================
// Test 4: Self-transition (state transitions to itself)
// ============================================================================

#[derive(Debug, Clone)]
struct SelfTransitionContext {
    reset_count: u32,
}

#[derive(Debug, Clone)]
enum SelfTransitionEvent {
    Reset,
    DoNothing,
}

state_machine! {
    Name: SelfTransitionFSM,
    Context: SelfTransitionContext,
    Event: SelfTransitionEvent,

    States: {
        Running { iteration: u32 } => {
            entry: |ctx| {
                ctx.reset_count += 1;
            }

            process: |_ctx, evt| {
                match evt {
                    SelfTransitionEvent::Reset => {
                        // Self-transition: same state, new data
                        Transition::To(SelfTransitionFSM::Running { iteration: 0 })
                    }
                    SelfTransitionEvent::DoNothing => Transition::None
                }
            }

            exit: |_ctx| {
                // Exit is called even for self-transitions
            }
        }
    }
}

#[test]
fn test_self_transition() {
    let mut ctx = SelfTransitionContext { reset_count: 0 };
    let mut fsm = SelfTransitionFSM::Running { iteration: 5 };

    fsm.init(&mut ctx);
    assert_eq!(ctx.reset_count, 1);

    // Self-transition should call exit then entry again
    fsm.dispatch(&mut ctx, &SelfTransitionEvent::Reset);
    assert_eq!(ctx.reset_count, 2);

    // No transition should not call entry/exit
    fsm.dispatch(&mut ctx, &SelfTransitionEvent::DoNothing);
    assert_eq!(ctx.reset_count, 2);
}

// ============================================================================
// Test 5: Multiple transitions in sequence
// ============================================================================

#[derive(Debug, Clone)]
struct ChainContext {
    path: Vec<&'static str>,
}

#[derive(Debug, Clone)]
enum ChainEvent {
    Next,
}

state_machine! {
    Name: ChainFSM,
    Context: ChainContext,
    Event: ChainEvent,

    States: {
        S1 => {
            entry: |ctx| {
                ctx.path.push("S1_entry");
            }

            process: |ctx, _evt| {
                ctx.path.push("S1_process");
                Transition::To(ChainFSM::S2)
            }

            exit: |ctx| {
                ctx.path.push("S1_exit");
            }
        },

        S2 => {
            entry: |ctx| {
                ctx.path.push("S2_entry");
            }

            process: |ctx, _evt| {
                ctx.path.push("S2_process");
                Transition::To(ChainFSM::S3)
            }

            exit: |ctx| {
                ctx.path.push("S2_exit");
            }
        },

        S3 => {
            entry: |ctx| {
                ctx.path.push("S3_entry");
            }

            process: |ctx, _evt| {
                ctx.path.push("S3_process");
                Transition::None
            }
        }
    }
}

#[test]
fn test_multiple_transitions_sequence() {
    let mut ctx = ChainContext { path: Vec::new() };
    let mut fsm = ChainFSM::S1;

    // Init
    fsm.init(&mut ctx);
    assert_eq!(ctx.path, vec!["S1_entry"]);

    // First transition: S1 -> S2
    fsm.dispatch(&mut ctx, &ChainEvent::Next);
    assert_eq!(
        ctx.path,
        vec!["S1_entry", "S1_process", "S1_exit", "S2_entry"]
    );

    // Second transition: S2 -> S3
    fsm.dispatch(&mut ctx, &ChainEvent::Next);
    assert_eq!(
        ctx.path,
        vec![
            "S1_entry",
            "S1_process",
            "S1_exit",
            "S2_entry",
            "S2_process",
            "S2_exit",
            "S3_entry"
        ]
    );

    // No transition: S3 stays in S3
    fsm.dispatch(&mut ctx, &ChainEvent::Next);
    assert_eq!(
        ctx.path,
        vec![
            "S1_entry",
            "S1_process",
            "S1_exit",
            "S2_entry",
            "S2_process",
            "S2_exit",
            "S3_entry",
            "S3_process"
        ]
    );
}

// ============================================================================
// Test 6: State with multiple fields and complex data
// ============================================================================

#[derive(Debug, Clone)]
struct ComplexContext {
    last_config: Option<(u32, u32, u32)>,
}

#[derive(Debug, Clone)]
enum ComplexEvent {
    Configure(u32, u32, u32),
    Clear,
}

state_machine! {
    Name: ComplexFSM,
    Context: ComplexContext,
    Event: ComplexEvent,

    States: {
        Idle => {
            process: |_ctx, evt| {
                match evt {
                    ComplexEvent::Configure(a, b, c) => {
                        Transition::To(ComplexFSM::Configured {
                            param_a: *a,
                            param_b: *b,
                            param_c: *c,
                        })
                    }
                    _ => Transition::None
                }
            }
        },

        Configured {
            param_a: u32,
            param_b: u32,
            param_c: u32
        } => {
            entry: |ctx| {
                ctx.last_config = Some((*param_a, *param_b, *param_c));
            }

            process: |_ctx, evt| {
                match evt {
                    ComplexEvent::Clear => Transition::To(ComplexFSM::Idle),
                    ComplexEvent::Configure(a, b, c) => {
                        Transition::To(ComplexFSM::Configured {
                            param_a: *a,
                            param_b: *b,
                            param_c: *c,
                        })
                    }
                }
            }

            exit: |_ctx| {
                // Cleanup could happen here
            }
        }
    }
}

#[test]
fn test_complex_state_with_multiple_fields() {
    let mut ctx = ComplexContext { last_config: None };
    let mut fsm = ComplexFSM::Idle;

    fsm.init(&mut ctx);
    assert!(ctx.last_config.is_none());

    // Configure with specific values
    fsm.dispatch(&mut ctx, &ComplexEvent::Configure(10, 20, 30));
    assert_eq!(ctx.last_config, Some((10, 20, 30)));

    // Reconfigure
    fsm.dispatch(&mut ctx, &ComplexEvent::Configure(5, 15, 25));
    assert_eq!(ctx.last_config, Some((5, 15, 25)));

    // Clear
    fsm.dispatch(&mut ctx, &ComplexEvent::Clear);
    assert_eq!(ctx.last_config, Some((5, 15, 25))); // Still has last config

    // Configure again
    fsm.dispatch(&mut ctx, &ComplexEvent::Configure(1, 2, 3));
    assert_eq!(ctx.last_config, Some((1, 2, 3)));
}

// ============================================================================
// Test 7: Verify Transition::None doesn't trigger exit/entry
// ============================================================================

#[derive(Debug, Clone)]
struct NoTransitionContext {
    entry_count: u32,
    exit_count: u32,
    event_count: u32,
}

#[derive(Debug, Clone)]
enum NoTransitionEvent {
    Process,
    Switch,
}

state_machine! {
    Name: NoTransitionFSM,
    Context: NoTransitionContext,
    Event: NoTransitionEvent,

    States: {
        Main => {
            entry: |ctx| {
                ctx.entry_count += 1;
            }

            process: |ctx, evt| {
                ctx.event_count += 1;
                match evt {
                    NoTransitionEvent::Process => Transition::None,
                    NoTransitionEvent::Switch => Transition::To(NoTransitionFSM::Other),
                }
            }

            exit: |ctx| {
                ctx.exit_count += 1;
            }
        },

        Other => {
            entry: |ctx| {
                ctx.entry_count += 1;
            }

            process: |ctx, _evt| {
                ctx.event_count += 1;
                Transition::None
            }

            exit: |ctx| {
                ctx.exit_count += 1;
            }
        }
    }
}

#[test]
fn test_transition_none_no_exit_entry() {
    let mut ctx = NoTransitionContext {
        entry_count: 0,
        exit_count: 0,
        event_count: 0,
    };
    let mut fsm = NoTransitionFSM::Main;

    // Init calls entry once
    fsm.init(&mut ctx);
    assert_eq!(ctx.entry_count, 1);
    assert_eq!(ctx.exit_count, 0);
    assert_eq!(ctx.event_count, 0);

    // Process events that return Transition::None
    fsm.dispatch(&mut ctx, &NoTransitionEvent::Process);
    assert_eq!(ctx.entry_count, 1); // No change
    assert_eq!(ctx.exit_count, 0); // No change
    assert_eq!(ctx.event_count, 1); // Incremented

    fsm.dispatch(&mut ctx, &NoTransitionEvent::Process);
    assert_eq!(ctx.entry_count, 1);
    assert_eq!(ctx.exit_count, 0);
    assert_eq!(ctx.event_count, 2);

    // Now switch states
    fsm.dispatch(&mut ctx, &NoTransitionEvent::Switch);
    assert_eq!(ctx.entry_count, 2); // Other's entry called
    assert_eq!(ctx.exit_count, 1); // Main's exit called
    assert_eq!(ctx.event_count, 3);

    // Process in Other state (no transition)
    fsm.dispatch(&mut ctx, &NoTransitionEvent::Process);
    assert_eq!(ctx.entry_count, 2); // No change
    assert_eq!(ctx.exit_count, 1); // No change
    assert_eq!(ctx.event_count, 4);
}

// ============================================================================
// Test 8: Single state FSM (edge case)
// ============================================================================

#[derive(Debug, Clone)]
struct SingleStateContext {
    value: u32,
}

#[derive(Debug, Clone)]
enum SingleStateEvent {
    Increment,
    Decrement,
}

state_machine! {
    Name: SingleStateFSM,
    Context: SingleStateContext,
    Event: SingleStateEvent,

    States: {
        Only => {
            entry: |ctx| {
                ctx.value = 100;
            }

            process: |ctx, evt| {
                match evt {
                    SingleStateEvent::Increment => {
                        ctx.value += 1;
                    }
                    SingleStateEvent::Decrement => {
                        ctx.value -= 1;
                    }
                }
                Transition::None
            }

            exit: |ctx| {
                ctx.value = 0;
            }
        }
    }
}

#[test]
fn test_single_state_machine() {
    let mut ctx = SingleStateContext { value: 0 };
    let mut fsm = SingleStateFSM::Only;

    fsm.init(&mut ctx);
    assert_eq!(ctx.value, 100);

    fsm.dispatch(&mut ctx, &SingleStateEvent::Increment);
    assert_eq!(ctx.value, 101);

    fsm.dispatch(&mut ctx, &SingleStateEvent::Decrement);
    assert_eq!(ctx.value, 100);

    fsm.dispatch(&mut ctx, &SingleStateEvent::Increment);
    fsm.dispatch(&mut ctx, &SingleStateEvent::Increment);
    assert_eq!(ctx.value, 102);
}

// ============================================================================
// Test 9: Match patterns in process blocks
// ============================================================================

#[derive(Debug, PartialEq, Clone)]
enum Color {
    Red,
    Green,
    Blue,
}

#[derive(Debug, Clone)]
struct PatternContext {
    current_color: Color,
    changes: u32,
}

#[derive(Debug, Clone)]
enum PatternEvent {
    SetColor(Color),
    Reset,
}

state_machine! {
    Name: PatternFSM,
    Context: PatternContext,
    Event: PatternEvent,

    States: {
        Active { color: Color } => {
            entry: |ctx| {
                ctx.current_color = match color {
                    Color::Red => Color::Red,
                    Color::Green => Color::Green,
                    Color::Blue => Color::Blue,
                };
                ctx.changes += 1;
            }

            process: |_ctx, evt| {
                match evt {
                    PatternEvent::SetColor(c) => {
                        Transition::To(PatternFSM::Active {
                            color: match c {
                                Color::Red => Color::Red,
                                Color::Green => Color::Green,
                                Color::Blue => Color::Blue,
                            },
                        })
                    }
                    PatternEvent::Reset => Transition::To(PatternFSM::Active { color: Color::Red }),
                }
            }
        }
    }
}

#[test]
fn test_pattern_matching_in_process() {
    let mut ctx = PatternContext {
        current_color: Color::Red,
        changes: 0,
    };
    let mut fsm = PatternFSM::Active { color: Color::Red };

    fsm.init(&mut ctx);
    assert_eq!(ctx.current_color, Color::Red);
    assert_eq!(ctx.changes, 1);

    // Change to Green
    fsm.dispatch(&mut ctx, &PatternEvent::SetColor(Color::Green));
    assert_eq!(ctx.current_color, Color::Green);
    assert_eq!(ctx.changes, 2);

    // Change to Blue
    fsm.dispatch(&mut ctx, &PatternEvent::SetColor(Color::Blue));
    assert_eq!(ctx.current_color, Color::Blue);
    assert_eq!(ctx.changes, 3);

    // Reset to Red
    fsm.dispatch(&mut ctx, &PatternEvent::Reset);
    assert_eq!(ctx.current_color, Color::Red);
    assert_eq!(ctx.changes, 4);
}

// ============================================================================
// Test 10: Minimal state (only process, no entry/exit)
// ============================================================================

#[derive(Debug, Clone)]
struct MinimalContext {
    processed: bool,
}

#[derive(Debug, Clone)]
enum MinimalEvent {
    Trigger,
}

state_machine! {
    Name: MinimalFSM,
    Context: MinimalContext,
    Event: MinimalEvent,

    States: {
        State => {
            process: |ctx, _evt| {
                ctx.processed = true;
                Transition::None
            }
        }
    }
}

#[test]
fn test_minimal_state_only_process() {
    let mut ctx = MinimalContext { processed: false };
    let mut fsm = MinimalFSM::State;

    // Init should work even with no entry
    fsm.init(&mut ctx);
    assert!(!ctx.processed);

    // Process should work
    fsm.dispatch(&mut ctx, &MinimalEvent::Trigger);
    assert!(ctx.processed);
}
