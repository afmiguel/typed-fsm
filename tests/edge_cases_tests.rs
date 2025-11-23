//! Edge cases and special scenarios tests

use typed_fsm::{state_machine, Transition};

// ============================================================================
// Test 1: Early return in process block
// ============================================================================

#[derive(Debug, Clone)]
struct EarlyReturnContext {
    error_handled: bool,
    normal_flow: bool,
}

#[derive(Debug, Clone)]
enum EarlyReturnEvent {
    ErrorCondition,
    NormalEvent,
}

state_machine! {
    Name: EarlyReturnFSM,
    Context: EarlyReturnContext,
    Event: EarlyReturnEvent,

    States: {
        Processing => {
            process: |ctx, evt| {
                match evt {
                    EarlyReturnEvent::ErrorCondition => {
                        ctx.error_handled = true;
                        // Early return
                        return Transition::To(EarlyReturnFSM::Error);
                    }
                    EarlyReturnEvent::NormalEvent => {
                        ctx.normal_flow = true;
                    }
                }
                Transition::None
            }
        },

        Error => {
            process: |_ctx, _evt| {
                Transition::None
            }
        }
    }
}

#[test]
fn test_early_return_in_process() {
    let mut ctx = EarlyReturnContext {
        error_handled: false,
        normal_flow: false,
    };
    let mut fsm = EarlyReturnFSM::Processing;

    fsm.init(&mut ctx);

    // Normal event doesn't trigger early return
    fsm.dispatch(&mut ctx, &EarlyReturnEvent::NormalEvent);
    assert!(ctx.normal_flow);
    assert!(!ctx.error_handled);

    // Error event triggers early return
    fsm.dispatch(&mut ctx, &EarlyReturnEvent::ErrorCondition);
    assert!(ctx.error_handled);
}

// ============================================================================
// Test 2: Unused context and event parameters
// ============================================================================

#[derive(Debug, Clone)]
struct UnusedParamsContext {
    value: u32,
}

#[derive(Debug, Clone)]
enum UnusedParamsEvent {
    Event1,
    Event2,
}

state_machine! {
    Name: UnusedParamsFSM,
    Context: UnusedParamsContext,
    Event: UnusedParamsEvent,

    States: {
        StateA => {
            entry: |_ctx| {
                // Context intentionally unused
            }

            process: |_ctx, _evt| {
                // Both parameters intentionally unused
                Transition::To(UnusedParamsFSM::StateB)
            }

            exit: |_ctx| {
                // Context intentionally unused
            }
        },

        StateB => {
            process: |ctx, _evt| {
                // Only event is unused
                ctx.value = 42;
                Transition::None
            }
        }
    }
}

#[test]
fn test_unused_parameters() {
    let mut ctx = UnusedParamsContext { value: 0 };
    let mut fsm = UnusedParamsFSM::StateA;

    fsm.init(&mut ctx);
    fsm.dispatch(&mut ctx, &UnusedParamsEvent::Event1);
    assert_eq!(ctx.value, 0);

    fsm.dispatch(&mut ctx, &UnusedParamsEvent::Event2);
    assert_eq!(ctx.value, 42);
}

// ============================================================================
// Test 3: State fields not used in some hooks
// ============================================================================

#[derive(Debug, Clone)]
struct PartialFieldContext {
    last_id: u32,
}

#[derive(Debug, Clone)]
enum PartialFieldEvent {
    Next,
}

state_machine! {
    Name: PartialFieldFSM,
    Context: PartialFieldContext,
    Event: PartialFieldEvent,

    States: {
        WithData { id: u32, name: &'static str } => {
            entry: |ctx| {
                // Only 'id' is used, 'name' is not
                ctx.last_id = *id;
            }

            process: |_ctx, _evt| {
                // State fields not accessed at all
                Transition::To(PartialFieldFSM::Other)
            }

            exit: |_ctx| {
                // Neither ctx nor state fields used
            }
        },

        Other => {
            process: |_ctx, _evt| {
                Transition::None
            }
        }
    }
}

#[test]
fn test_partial_field_usage() {
    let mut ctx = PartialFieldContext { last_id: 0 };
    let mut fsm = PartialFieldFSM::WithData {
        id: 123,
        name: "test",
    };

    fsm.init(&mut ctx);
    assert_eq!(ctx.last_id, 123);

    fsm.dispatch(&mut ctx, &PartialFieldEvent::Next);
    // Verify transition happened
    fsm.dispatch(&mut ctx, &PartialFieldEvent::Next);
    assert_eq!(ctx.last_id, 123); // No change in Other state
}

// ============================================================================
// Test 4: Nested match patterns in process
// ============================================================================

#[derive(Debug, PartialEq, Clone)]
enum Status {
    Active,
    Inactive,
}

#[derive(Debug, Clone)]
struct NestedMatchContext {
    status_changes: u32,
    last_value: Option<u32>,
}

#[derive(Debug, Clone)]
enum NestedMatchEvent {
    Update { status: Status, value: Option<u32> },
    Clear,
}

state_machine! {
    Name: NestedMatchFSM,
    Context: NestedMatchContext,
    Event: NestedMatchEvent,

    States: {
        Running => {
            process: |ctx, evt| {
                match evt {
                    NestedMatchEvent::Update { status, value } => {
                        match status {
                            Status::Active => {
                                ctx.status_changes += 1;
                                match value {
                                    Some(v) => {
                                        ctx.last_value = Some(*v);
                                    }
                                    None => {
                                        ctx.last_value = None;
                                    }
                                }
                            }
                            Status::Inactive => {
                                ctx.status_changes += 10;
                            }
                        }
                        Transition::None
                    }
                    NestedMatchEvent::Clear => {
                        ctx.last_value = None;
                        ctx.status_changes = 0;
                        Transition::None
                    }
                }
            }
        }
    }
}

#[test]
fn test_nested_match_patterns() {
    let mut ctx = NestedMatchContext {
        status_changes: 0,
        last_value: None,
    };
    let mut fsm = NestedMatchFSM::Running;

    fsm.init(&mut ctx);

    // Active with Some value
    fsm.dispatch(
        &mut ctx,
        &NestedMatchEvent::Update {
            status: Status::Active,
            value: Some(42),
        },
    );
    assert_eq!(ctx.status_changes, 1);
    assert_eq!(ctx.last_value, Some(42));

    // Active with None
    fsm.dispatch(
        &mut ctx,
        &NestedMatchEvent::Update {
            status: Status::Active,
            value: None,
        },
    );
    assert_eq!(ctx.status_changes, 2);
    assert_eq!(ctx.last_value, None);

    // Inactive
    fsm.dispatch(
        &mut ctx,
        &NestedMatchEvent::Update {
            status: Status::Inactive,
            value: Some(100),
        },
    );
    assert_eq!(ctx.status_changes, 12);

    // Clear
    fsm.dispatch(&mut ctx, &NestedMatchEvent::Clear);
    assert_eq!(ctx.status_changes, 0);
    assert_eq!(ctx.last_value, None);
}

// ============================================================================
// Test 5: Multiple consecutive self-transitions
// ============================================================================

#[derive(Debug, Clone)]
struct MultiSelfContext {
    iteration: u32,
    resets: u32,
}

#[derive(Debug, Clone)]
enum MultiSelfEvent {
    Iterate,
}

state_machine! {
    Name: MultiSelfFSM,
    Context: MultiSelfContext,
    Event: MultiSelfEvent,

    States: {
        Loop { counter: u32 } => {
            entry: |ctx| {
                ctx.iteration += 1;
                if *counter == 0 {
                    ctx.resets += 1;
                }
            }

            process: |_ctx, _evt| {
                let next_counter = if *counter < 5 {
                    *counter + 1
                } else {
                    0
                };
                Transition::To(MultiSelfFSM::Loop {
                    counter: next_counter,
                })
            }
        }
    }
}

#[test]
fn test_multiple_self_transitions() {
    let mut ctx = MultiSelfContext {
        iteration: 0,
        resets: 0,
    };
    let mut fsm = MultiSelfFSM::Loop { counter: 0 };

    fsm.init(&mut ctx);
    assert_eq!(ctx.iteration, 1);
    assert_eq!(ctx.resets, 1);

    // Perform multiple self-transitions
    for i in 1..=10 {
        fsm.dispatch(&mut ctx, &MultiSelfEvent::Iterate);
        assert_eq!(ctx.iteration, i + 1);
    }

    // Should have reset twice (at counter 0 initially and after reaching 5)
    assert_eq!(ctx.resets, 2);
}

// ============================================================================
// Test 6: States with different numbers of fields
// ============================================================================

#[derive(Debug, Clone)]
struct MixedFieldsContext {
    state_name: &'static str,
}

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
enum MixedFieldsEvent {
    ToZero,
    ToOne(u32),
    ToTwo(u32, u32),
    ToThree(u32, u32, u32),
}

state_machine! {
    Name: MixedFieldsFSM,
    Context: MixedFieldsContext,
    Event: MixedFieldsEvent,

    States: {
        NoFields => {
            entry: |ctx| {
                ctx.state_name = "NoFields";
            }

            process: |_ctx, evt| {
                match evt {
                    MixedFieldsEvent::ToOne(a) => {
                        Transition::To(MixedFieldsFSM::OneField { a: *a })
                    }
                    _ => Transition::None
                }
            }
        },

        OneField { a: u32 } => {
            entry: |ctx| {
                ctx.state_name = "OneField";
            }

            process: |_ctx, evt| {
                match evt {
                    MixedFieldsEvent::ToTwo(x, y) => {
                        Transition::To(MixedFieldsFSM::TwoFields { x: *x, y: *y })
                    }
                    MixedFieldsEvent::ToZero => Transition::To(MixedFieldsFSM::NoFields),
                    _ => Transition::None
                }
            }
        },

        TwoFields { x: u32, y: u32 } => {
            entry: |ctx| {
                ctx.state_name = "TwoFields";
            }

            process: |_ctx, evt| {
                match evt {
                    MixedFieldsEvent::ToThree(a, b, c) => {
                        Transition::To(MixedFieldsFSM::ThreeFields {
                            a: *a,
                            b: *b,
                            c: *c,
                        })
                    }
                    MixedFieldsEvent::ToZero => Transition::To(MixedFieldsFSM::NoFields),
                    _ => Transition::None
                }
            }
        },

        ThreeFields { a: u32, b: u32, c: u32 } => {
            entry: |ctx| {
                ctx.state_name = "ThreeFields";
            }

            process: |_ctx, evt| {
                match evt {
                    MixedFieldsEvent::ToZero => Transition::To(MixedFieldsFSM::NoFields),
                    _ => Transition::None
                }
            }
        }
    }
}

#[test]
fn test_states_with_mixed_fields() {
    let mut ctx = MixedFieldsContext { state_name: "" };
    let mut fsm = MixedFieldsFSM::NoFields;

    fsm.init(&mut ctx);
    assert_eq!(ctx.state_name, "NoFields");

    // NoFields -> OneField
    fsm.dispatch(&mut ctx, &MixedFieldsEvent::ToOne(10));
    assert_eq!(ctx.state_name, "OneField");

    // OneField -> TwoFields
    fsm.dispatch(&mut ctx, &MixedFieldsEvent::ToTwo(20, 30));
    assert_eq!(ctx.state_name, "TwoFields");

    // TwoFields -> ThreeFields
    fsm.dispatch(&mut ctx, &MixedFieldsEvent::ToThree(1, 2, 3));
    assert_eq!(ctx.state_name, "ThreeFields");

    // ThreeFields -> NoFields
    fsm.dispatch(&mut ctx, &MixedFieldsEvent::ToZero);
    assert_eq!(ctx.state_name, "NoFields");

    // Test direct transitions
    fsm.dispatch(&mut ctx, &MixedFieldsEvent::ToOne(99));
    assert_eq!(ctx.state_name, "OneField");

    fsm.dispatch(&mut ctx, &MixedFieldsEvent::ToZero);
    assert_eq!(ctx.state_name, "NoFields");
}

// ============================================================================
// Test 7: Wildcard patterns in match
// ============================================================================

#[derive(Debug, Clone)]
struct WildcardContext {
    default_count: u32,
    specific_count: u32,
}

#[derive(Debug, Clone)]
enum WildcardEvent {
    Specific,
    Other1,
    Other2,
    Other3,
}

state_machine! {
    Name: WildcardFSM,
    Context: WildcardContext,
    Event: WildcardEvent,

    States: {
        Active => {
            process: |ctx, evt| {
                match evt {
                    WildcardEvent::Specific => {
                        ctx.specific_count += 1;
                    }
                    _ => {
                        ctx.default_count += 1;
                    }
                }
                Transition::None
            }
        }
    }
}

#[test]
fn test_wildcard_pattern_in_match() {
    let mut ctx = WildcardContext {
        default_count: 0,
        specific_count: 0,
    };
    let mut fsm = WildcardFSM::Active;

    fsm.init(&mut ctx);

    fsm.dispatch(&mut ctx, &WildcardEvent::Specific);
    assert_eq!(ctx.specific_count, 1);
    assert_eq!(ctx.default_count, 0);

    fsm.dispatch(&mut ctx, &WildcardEvent::Other1);
    assert_eq!(ctx.specific_count, 1);
    assert_eq!(ctx.default_count, 1);

    fsm.dispatch(&mut ctx, &WildcardEvent::Other2);
    assert_eq!(ctx.specific_count, 1);
    assert_eq!(ctx.default_count, 2);

    fsm.dispatch(&mut ctx, &WildcardEvent::Other3);
    assert_eq!(ctx.specific_count, 1);
    assert_eq!(ctx.default_count, 3);

    fsm.dispatch(&mut ctx, &WildcardEvent::Specific);
    assert_eq!(ctx.specific_count, 2);
    assert_eq!(ctx.default_count, 3);
}

// ============================================================================
// Test 8: If-let patterns in process
// ============================================================================

#[derive(Debug, Clone)]
struct IfLetContext {
    some_count: u32,
    none_count: u32,
}

#[derive(Debug, Clone)]
enum IfLetEvent {
    MaybeValue(Option<u32>),
}

state_machine! {
    Name: IfLetFSM,
    Context: IfLetContext,
    Event: IfLetEvent,

    States: {
        Processing => {
            process: |ctx, evt| {
                if let IfLetEvent::MaybeValue(Some(val)) = evt {
                    ctx.some_count += val;
                } else if let IfLetEvent::MaybeValue(None) = evt {
                    ctx.none_count += 1;
                }
                Transition::None
            }
        }
    }
}

#[test]
fn test_if_let_patterns() {
    let mut ctx = IfLetContext {
        some_count: 0,
        none_count: 0,
    };
    let mut fsm = IfLetFSM::Processing;

    fsm.init(&mut ctx);

    fsm.dispatch(&mut ctx, &IfLetEvent::MaybeValue(Some(5)));
    assert_eq!(ctx.some_count, 5);
    assert_eq!(ctx.none_count, 0);

    fsm.dispatch(&mut ctx, &IfLetEvent::MaybeValue(Some(10)));
    assert_eq!(ctx.some_count, 15);
    assert_eq!(ctx.none_count, 0);

    fsm.dispatch(&mut ctx, &IfLetEvent::MaybeValue(None));
    assert_eq!(ctx.some_count, 15);
    assert_eq!(ctx.none_count, 1);

    fsm.dispatch(&mut ctx, &IfLetEvent::MaybeValue(None));
    assert_eq!(ctx.some_count, 15);
    assert_eq!(ctx.none_count, 2);
}
