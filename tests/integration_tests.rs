//! Integration tests for the finite_state_machine library

use typed_fsm::{state_machine, Transition};

// ============================================================================
// Test 1: Simple Toggle State Machine
// ============================================================================

#[derive(Debug, Clone)]
struct ToggleContext {
    toggle_count: u32,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
struct CounterContext {
    value: i32,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
struct ResourceContext {
    acquired: bool,
    released: bool,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
struct NoopContext {
    event_count: u32,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
struct MultiFieldContext {
    sum: i32,
}

#[derive(Debug, Clone)]
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

// ============================================================================
// Test 6: Init Behavior - With init() call
// ============================================================================

#[derive(Debug, Clone)]
struct InitContext {
    entry_called: bool,
    entry_call_count: u32,
}

#[derive(Debug, Clone)]
enum InitEvent {
    Trigger,
}

state_machine! {
    Name: InitFSM,
    Context: InitContext,
    Event: InitEvent,

    States: {
        Initial => {
            entry: |ctx| {
                ctx.entry_called = true;
                ctx.entry_call_count += 1;
            }

            process: |_ctx, evt| {
                match evt {
                    InitEvent::Trigger => Transition::To(InitFSM::Active),
                }
            }
        },

        Active => {
            entry: |ctx| {
                ctx.entry_call_count += 1;
            }

            process: |_ctx, _evt| {
                Transition::None
            }
        }
    }
}

#[test]
fn test_init_called_executes_initial_entry() {
    let mut ctx = InitContext {
        entry_called: false,
        entry_call_count: 0,
    };
    let mut fsm = InitFSM::Initial;

    // Before init, entry should not have been called
    assert!(!ctx.entry_called);
    assert_eq!(ctx.entry_call_count, 0);

    // Call init - this SHOULD execute the entry action
    fsm.init(&mut ctx);

    // After init, entry action should have executed
    assert!(ctx.entry_called);
    assert_eq!(ctx.entry_call_count, 1);

    // Dispatch event to transition
    fsm.dispatch(&mut ctx, &InitEvent::Trigger);
    assert_eq!(ctx.entry_call_count, 2); // Initial entry + Active entry
}

#[test]
fn test_init_not_called_skips_initial_entry() {
    let mut ctx = InitContext {
        entry_called: false,
        entry_call_count: 0,
    };
    let mut fsm = InitFSM::Initial;

    // Skip init() call - this is the WRONG way to use the FSM

    // Dispatch event without calling init first
    fsm.dispatch(&mut ctx, &InitEvent::Trigger);

    // The Initial state's entry was NEVER called
    // Only Active state's entry was called
    assert!(!ctx.entry_called); // Initial entry never executed!
    assert_eq!(ctx.entry_call_count, 1); // Only Active entry was called
}

// ============================================================================
// Test 7: Blink Pattern (like the blink.rs example)
// ============================================================================

#[derive(Debug, Clone)]
struct BlinkContext {
    led_on_count: u32,
    led_off_count: u32,
    tick_count: u32,
}

#[derive(Debug, Clone)]
enum BlinkEvent {
    Tick,
}

state_machine! {
    Name: BlinkFSM,
    Context: BlinkContext,
    Event: BlinkEvent,

    States: {
        On => {
            entry: |ctx| {
                ctx.led_on_count += 1;
                ctx.tick_count += 1;
            }

            process: |_ctx, evt| {
                match evt {
                    BlinkEvent::Tick => Transition::To(BlinkFSM::Off),
                }
            }
        },

        Off => {
            entry: |ctx| {
                ctx.led_off_count += 1;
                ctx.tick_count += 1;
            }

            process: |_ctx, evt| {
                match evt {
                    BlinkEvent::Tick => Transition::To(BlinkFSM::On),
                }
            }
        }
    }
}

#[test]
fn test_blink_alternates_states() {
    let mut ctx = BlinkContext {
        led_on_count: 0,
        led_off_count: 0,
        tick_count: 0,
    };
    let mut fsm = BlinkFSM::On;

    // Initialize - calls On entry
    fsm.init(&mut ctx);
    assert_eq!(ctx.led_on_count, 1);
    assert_eq!(ctx.led_off_count, 0);
    assert_eq!(ctx.tick_count, 1);

    // Tick: On → Off
    fsm.dispatch(&mut ctx, &BlinkEvent::Tick);
    assert_eq!(ctx.led_on_count, 1);
    assert_eq!(ctx.led_off_count, 1);
    assert_eq!(ctx.tick_count, 2);

    // Tick: Off → On
    fsm.dispatch(&mut ctx, &BlinkEvent::Tick);
    assert_eq!(ctx.led_on_count, 2);
    assert_eq!(ctx.led_off_count, 1);
    assert_eq!(ctx.tick_count, 3);

    // Tick: On → Off
    fsm.dispatch(&mut ctx, &BlinkEvent::Tick);
    assert_eq!(ctx.led_on_count, 2);
    assert_eq!(ctx.led_off_count, 2);
    assert_eq!(ctx.tick_count, 4);

    // Tick: Off → On
    fsm.dispatch(&mut ctx, &BlinkEvent::Tick);
    assert_eq!(ctx.led_on_count, 3);
    assert_eq!(ctx.led_off_count, 2);
    assert_eq!(ctx.tick_count, 5);
}

#[test]
fn test_blink_multiple_cycles() {
    let mut ctx = BlinkContext {
        led_on_count: 0,
        led_off_count: 0,
        tick_count: 0,
    };
    let mut fsm = BlinkFSM::On;

    fsm.init(&mut ctx);

    // Run 10 ticks (5 complete cycles)
    for _ in 0..10 {
        fsm.dispatch(&mut ctx, &BlinkEvent::Tick);
    }

    // Should have 6 On counts (1 from init + 5 from ticks) and 5 Off counts
    assert_eq!(ctx.led_on_count, 6);
    assert_eq!(ctx.led_off_count, 5);
    assert_eq!(ctx.tick_count, 11); // 1 from init + 10 from ticks
}

// ============================================================================
// Test 8: Hierarchical State Machine Pattern
// ============================================================================

#[derive(Debug, Clone)]
struct NestedVolumeContext {
    volume_level: u8,
}

#[derive(Debug, Clone)]
enum VolumeCommand {
    Up,
    Down,
}

state_machine! {
    Name: NestedVolumeFSM,
    Context: NestedVolumeContext,
    Event: VolumeCommand,

    States: {
        Low => {
            entry: |ctx| {
                ctx.volume_level = 25;
            }

            process: |_ctx, evt| {
                match evt {
                    VolumeCommand::Up => Transition::To(NestedVolumeFSM::High),
                    VolumeCommand::Down => Transition::None
                }
            }
        },

        High => {
            entry: |ctx| {
                ctx.volume_level = 75;
            }

            process: |_ctx, evt| {
                match evt {
                    VolumeCommand::Down => Transition::To(NestedVolumeFSM::Low),
                    VolumeCommand::Up => Transition::None
                }
            }
        }
    }
}

#[derive(Debug)]
struct HierarchicalContext {
    is_active: bool,
    nested_fsm: Option<NestedVolumeFSM>,
    volume_ctx: NestedVolumeContext,
}

#[derive(Debug, Clone)]
enum HierarchicalEvent {
    Activate,
    Deactivate,
    VolumeEvent(VolumeCommand),
}

state_machine! {
    Name: HierarchicalFSM,
    Context: HierarchicalContext,
    Event: HierarchicalEvent,

    States: {
        Inactive => {
            entry: |ctx| {
                ctx.is_active = false;
                ctx.nested_fsm = None;
            }

            process: |_ctx, evt| {
                match evt {
                    HierarchicalEvent::Activate => Transition::To(HierarchicalFSM::Active),
                    _ => Transition::None
                }
            }
        },

        Active => {
            entry: |ctx| {
                ctx.is_active = true;
                // Initialize nested FSM when entering Active state
                let mut nested = NestedVolumeFSM::Low;
                nested.init(&mut ctx.volume_ctx);
                ctx.nested_fsm = Some(nested);
            }

            process: |ctx, evt| {
                match evt {
                    HierarchicalEvent::Deactivate => Transition::To(HierarchicalFSM::Inactive),

                    // Delegate volume events to nested FSM
                    HierarchicalEvent::VolumeEvent(vol_cmd) => {
                        if let Some(ref mut nested_fsm) = ctx.nested_fsm {
                            nested_fsm.dispatch(&mut ctx.volume_ctx, vol_cmd);
                        }
                        Transition::None
                    },

                    _ => Transition::None
                }
            }

            exit: |ctx| {
                // Cleanup nested FSM when leaving Active state
                ctx.nested_fsm = None;
            }
        }
    }
}

#[test]
fn test_hierarchical_nested_fsm_lifecycle() {
    let mut ctx = HierarchicalContext {
        is_active: false,
        nested_fsm: None,
        volume_ctx: NestedVolumeContext { volume_level: 0 },
    };
    let mut fsm = HierarchicalFSM::Inactive;

    fsm.init(&mut ctx);

    // Initially inactive, no nested FSM
    assert!(!ctx.is_active);
    assert!(ctx.nested_fsm.is_none());

    // Activate - should create nested FSM
    fsm.dispatch(&mut ctx, &HierarchicalEvent::Activate);
    assert!(ctx.is_active);
    assert!(ctx.nested_fsm.is_some());
    assert_eq!(ctx.volume_ctx.volume_level, 25); // Low volume initialized

    // Test nested FSM functionality
    fsm.dispatch(&mut ctx, &HierarchicalEvent::VolumeEvent(VolumeCommand::Up));
    assert_eq!(ctx.volume_ctx.volume_level, 75); // High volume

    fsm.dispatch(
        &mut ctx,
        &HierarchicalEvent::VolumeEvent(VolumeCommand::Down),
    );
    assert_eq!(ctx.volume_ctx.volume_level, 25); // Back to Low

    // Deactivate - should cleanup nested FSM
    fsm.dispatch(&mut ctx, &HierarchicalEvent::Deactivate);
    assert!(!ctx.is_active);
    assert!(ctx.nested_fsm.is_none());

    // Volume events have no effect when inactive
    let volume_before = ctx.volume_ctx.volume_level;
    fsm.dispatch(&mut ctx, &HierarchicalEvent::VolumeEvent(VolumeCommand::Up));
    assert_eq!(ctx.volume_ctx.volume_level, volume_before); // No change
}

#[test]
fn test_hierarchical_nested_fsm_reinitialization() {
    let mut ctx = HierarchicalContext {
        is_active: false,
        nested_fsm: None,
        volume_ctx: NestedVolumeContext { volume_level: 0 },
    };
    let mut fsm = HierarchicalFSM::Inactive;

    fsm.init(&mut ctx);

    // Activate first time
    fsm.dispatch(&mut ctx, &HierarchicalEvent::Activate);
    assert_eq!(ctx.volume_ctx.volume_level, 25); // Low

    // Change volume
    fsm.dispatch(&mut ctx, &HierarchicalEvent::VolumeEvent(VolumeCommand::Up));
    assert_eq!(ctx.volume_ctx.volume_level, 75); // High

    // Deactivate (cleans up nested FSM)
    fsm.dispatch(&mut ctx, &HierarchicalEvent::Deactivate);

    // Activate again - nested FSM should be reinitialized to Low
    fsm.dispatch(&mut ctx, &HierarchicalEvent::Activate);
    assert_eq!(ctx.volume_ctx.volume_level, 25); // Reset to Low, not 75
}

// ============================================================================
// Test 10-11: Concurrent State Machines
// ============================================================================

use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone)]
struct ConcurrentContext {
    counter: u32,
}

#[derive(Debug, Clone)]
enum ConcurrentEvent {
    Increment,
    #[allow(dead_code)]
    Reset,
}

state_machine! {
    Name: ConcurrentFSM,
    Context: ConcurrentContext,
    Event: ConcurrentEvent,

    States: {
        Active => {
            process: |ctx, evt| {
                match evt {
                    ConcurrentEvent::Increment => {
                        ctx.counter += 1;
                        Transition::None
                    }
                    ConcurrentEvent::Reset => Transition::To(ConcurrentFSM::Idle)
                }
            }
        },

        Idle => {
            entry: |ctx| {
                ctx.counter = 0;
            }

            process: |ctx, evt| {
                match evt {
                    ConcurrentEvent::Increment => {
                        ctx.counter += 1;
                        Transition::To(ConcurrentFSM::Active)
                    }
                    _ => Transition::None
                }
            }
        }
    }
}

#[test]
fn test_concurrent_fsm_basic() {
    // Demonstrates that FSMs can be safely sent to threads
    let ctx = ConcurrentContext { counter: 0 };
    let fsm = ConcurrentFSM::Idle;

    // Spawn FSM in a separate thread
    let handle = thread::spawn(move || {
        let mut ctx = ctx;
        let mut fsm = fsm;
        fsm.init(&mut ctx);

        // Process events
        fsm.dispatch(&mut ctx, &ConcurrentEvent::Increment);
        fsm.dispatch(&mut ctx, &ConcurrentEvent::Increment);
        fsm.dispatch(&mut ctx, &ConcurrentEvent::Increment);

        ctx.counter
    });

    let result = handle.join().unwrap();
    assert_eq!(result, 3);
}

#[test]
fn test_concurrent_fsm_synchronization() {
    // Demonstrates Arc<Mutex<>> pattern for shared FSM access
    use std::sync::mpsc::channel;

    let fsm = Arc::new(Mutex::new(ConcurrentFSM::Idle));
    let ctx = Arc::new(Mutex::new(ConcurrentContext { counter: 0 }));

    // Initialize
    {
        let mut fsm_lock = fsm.lock().unwrap();
        let mut ctx_lock = ctx.lock().unwrap();
        fsm_lock.init(&mut ctx_lock);
    }

    let (tx, rx) = channel();

    // Spawn 3 threads that increment concurrently
    for i in 0..3 {
        let fsm_clone = Arc::clone(&fsm);
        let ctx_clone = Arc::clone(&ctx);
        let tx_clone = tx.clone();

        thread::spawn(move || {
            for _ in 0..10 {
                let mut fsm_lock = fsm_clone.lock().unwrap();
                let mut ctx_lock = ctx_clone.lock().unwrap();
                fsm_lock.dispatch(&mut ctx_lock, &ConcurrentEvent::Increment);
            }
            tx_clone.send(i).unwrap();
        });
    }

    // Wait for all threads to complete
    drop(tx);
    while rx.recv().is_ok() {}

    // Verify final counter value
    let ctx_lock = ctx.lock().unwrap();
    assert_eq!(ctx_lock.counter, 30); // 3 threads * 10 increments = 30
}
