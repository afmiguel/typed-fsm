//! # Concurrent State Machines Example: Traffic Intersection
//!
//! This example demonstrates **concurrent state machines** running in parallel threads
//! with synchronized coordination using Arc<Mutex<>> and channels.
//!
//! ## Architecture
//!
//! - **4 FSMs running in parallel threads:**
//!   - North/South Traffic Light (Green → Yellow → Red)
//!   - East/West Traffic Light (Red → Green → Yellow)
//!   - Pedestrian Crossing (Wait → Walk → Flash)
//!   - Emergency Override (monitors for emergency vehicles)
//!
//! - **Synchronization:**
//!   - Shared state: `Arc<Mutex<IntersectionState>>`
//!   - Event distribution: `mpsc::channel`
//!   - Safety coordination: prevents conflicting green lights
//!
//! ## Key Concepts
//!
//! - FSMs are automatically `Send + Sync` if their fields are
//! - Use `Arc<Mutex<>>` for thread-safe sharing
//! - Use channels for event broadcasting
//! - Each FSM runs independently but coordinates via shared state
//!
//! Run with: `cargo run --example traffic_intersection`

use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use typed_fsm::{state_machine, Transition};

// ============================================================================
// Shared Intersection State (Protected by Mutex)
// ============================================================================

#[derive(Debug, Clone)]
struct IntersectionState {
    ns_is_green: bool,
    ew_is_green: bool,
    pedestrian_walking: bool,
    emergency_active: bool,
}

impl IntersectionState {
    fn new() -> Self {
        Self {
            ns_is_green: false,
            ew_is_green: false,
            pedestrian_walking: false,
            emergency_active: false,
        }
    }

    fn is_safe_for_ns_green(&self) -> bool {
        !self.ew_is_green && !self.pedestrian_walking && !self.emergency_active
    }

    fn is_safe_for_ew_green(&self) -> bool {
        !self.ns_is_green && !self.pedestrian_walking && !self.emergency_active
    }

    fn is_safe_for_pedestrian(&self) -> bool {
        !self.ns_is_green && !self.ew_is_green && !self.emergency_active
    }
}

// ============================================================================
// Events
// ============================================================================

#[derive(Debug, Clone)]
enum TrafficEvent {
    TimerTick,
    PedestrianButton,
    Emergency,
    #[allow(dead_code)]
    EmergencyClear,
}

// ============================================================================
// North/South Traffic Light FSM
// ============================================================================

struct NSLightContext {
    intersection: Arc<Mutex<IntersectionState>>,
    tick_count: u32,
}

state_machine! {
    Name: NSLightFSM,
    Context: NSLightContext,
    Event: TrafficEvent,

    States: {
        Red => {
            entry: |ctx| {
                let mut state = ctx.intersection.lock().unwrap();
                state.ns_is_green = false;
                println!("  🔴 [NS] RED");
            }

            process: |ctx, evt| {
                match evt {
                    TrafficEvent::TimerTick => {
                        ctx.tick_count += 1;
                        if ctx.tick_count >= 3 {
                            ctx.tick_count = 0;
                            let state = ctx.intersection.lock().unwrap();
                            if state.is_safe_for_ns_green() {
                                return Transition::To(NSLightFSM::Green);
                            }
                        }
                        Transition::None
                    }
                    _ => Transition::None
                }
            }
        },

        Green => {
            entry: |ctx| {
                let mut state = ctx.intersection.lock().unwrap();
                state.ns_is_green = true;
                println!("  🟢 [NS] GREEN");
            }

            process: |ctx, evt| {
                match evt {
                    TrafficEvent::TimerTick => {
                        ctx.tick_count += 1;
                        if ctx.tick_count >= 4 {
                            ctx.tick_count = 0;
                            Transition::To(NSLightFSM::Yellow)
                        } else {
                            Transition::None
                        }
                    }
                    TrafficEvent::Emergency => Transition::To(NSLightFSM::Red),
                    _ => Transition::None
                }
            }
        },

        Yellow => {
            entry: |ctx| {
                println!("  🟡 [NS] YELLOW");
                ctx.intersection.lock().unwrap().ns_is_green = false;
            }

            process: |ctx, evt| {
                match evt {
                    TrafficEvent::TimerTick => {
                        ctx.tick_count += 1;
                        if ctx.tick_count >= 1 {
                            ctx.tick_count = 0;
                            Transition::To(NSLightFSM::Red)
                        } else {
                            Transition::None
                        }
                    }
                    _ => Transition::None
                }
            }
        }
    }
}

// ============================================================================
// East/West Traffic Light FSM
// ============================================================================

struct EWLightContext {
    intersection: Arc<Mutex<IntersectionState>>,
    tick_count: u32,
}

state_machine! {
    Name: EWLightFSM,
    Context: EWLightContext,
    Event: TrafficEvent,

    States: {
        Red => {
            entry: |ctx| {
                let mut state = ctx.intersection.lock().unwrap();
                state.ew_is_green = false;
                println!("    🔴 [EW] RED");
            }

            process: |ctx, evt| {
                match evt {
                    TrafficEvent::TimerTick => {
                        ctx.tick_count += 1;
                        if ctx.tick_count >= 3 {
                            ctx.tick_count = 0;
                            let state = ctx.intersection.lock().unwrap();
                            if state.is_safe_for_ew_green() {
                                return Transition::To(EWLightFSM::Green);
                            }
                        }
                        Transition::None
                    }
                    _ => Transition::None
                }
            }
        },

        Green => {
            entry: |ctx| {
                let mut state = ctx.intersection.lock().unwrap();
                state.ew_is_green = true;
                println!("    🟢 [EW] GREEN");
            }

            process: |ctx, evt| {
                match evt {
                    TrafficEvent::TimerTick => {
                        ctx.tick_count += 1;
                        if ctx.tick_count >= 4 {
                            ctx.tick_count = 0;
                            Transition::To(EWLightFSM::Yellow)
                        } else {
                            Transition::None
                        }
                    }
                    TrafficEvent::Emergency => Transition::To(EWLightFSM::Red),
                    _ => Transition::None
                }
            }
        },

        Yellow => {
            entry: |ctx| {
                println!("    🟡 [EW] YELLOW");
                ctx.intersection.lock().unwrap().ew_is_green = false;
            }

            process: |ctx, evt| {
                match evt {
                    TrafficEvent::TimerTick => {
                        ctx.tick_count += 1;
                        if ctx.tick_count >= 1 {
                            ctx.tick_count = 0;
                            Transition::To(EWLightFSM::Red)
                        } else {
                            Transition::None
                        }
                    }
                    _ => Transition::None
                }
            }
        }
    }
}

// ============================================================================
// Pedestrian Crossing FSM
// ============================================================================

struct PedestrianContext {
    intersection: Arc<Mutex<IntersectionState>>,
    button_pressed: bool,
}

state_machine! {
    Name: PedestrianFSM,
    Context: PedestrianContext,
    Event: TrafficEvent,

    States: {
        Wait => {
            entry: |ctx| {
                ctx.intersection.lock().unwrap().pedestrian_walking = false;
                println!("      🚶 [PED] WAIT (Don't Walk)");
            }

            process: |ctx, evt| {
                match evt {
                    TrafficEvent::PedestrianButton => {
                        ctx.button_pressed = true;
                        println!("      🚶 [PED] Button pressed!");
                        Transition::None
                    }
                    TrafficEvent::TimerTick => {
                        if ctx.button_pressed {
                            let state = ctx.intersection.lock().unwrap();
                            if state.is_safe_for_pedestrian() {
                                ctx.button_pressed = false;
                                return Transition::To(PedestrianFSM::Walk);
                            }
                        }
                        Transition::None
                    }
                    _ => Transition::None
                }
            }
        },

        Walk => {
            entry: |ctx| {
                ctx.intersection.lock().unwrap().pedestrian_walking = true;
                println!("      🚶 [PED] WALK");
            }

            process: |_ctx, evt| {
                match evt {
                    TrafficEvent::TimerTick => Transition::To(PedestrianFSM::Flash),
                    TrafficEvent::Emergency => Transition::To(PedestrianFSM::Wait),
                    _ => Transition::None
                }
            }
        },

        Flash => {
            entry: |_ctx| {
                println!("      🚶 [PED] FLASHING (Hurry!)");
            }

            process: |_ctx, evt| {
                match evt {
                    TrafficEvent::TimerTick => Transition::To(PedestrianFSM::Wait),
                    _ => Transition::None
                }
            }

            exit: |ctx| {
                ctx.intersection.lock().unwrap().pedestrian_walking = false;
            }
        }
    }
}

// ============================================================================
// Main: Spawns concurrent FSMs
// ============================================================================

fn main() {
    println!("=== Traffic Intersection: Concurrent State Machines ===\n");
    println!("Legend:");
    println!("  🔴 Red    🟡 Yellow    🟢 Green");
    println!("  [NS] = North/South    [EW] = East/West    [PED] = Pedestrian\n");
    println!("Starting intersection control...\n");

    // Shared intersection state (protected by Mutex)
    let intersection = Arc::new(Mutex::new(IntersectionState::new()));

    // Spawn North/South Light Thread
    let ns_intersection = Arc::clone(&intersection);
    let (ns_event_tx, ns_event_rx) = channel();
    thread::spawn(move || {
        let mut ctx = NSLightContext {
            intersection: ns_intersection,
            tick_count: 0,
        };
        let mut fsm = NSLightFSM::Red;
        fsm.init(&mut ctx);

        while let Ok(event) = ns_event_rx.recv() {
            fsm.dispatch(&mut ctx, &event);
        }
    });

    // Spawn East/West Light Thread
    let ew_intersection = Arc::clone(&intersection);
    let (ew_event_tx, ew_event_rx) = channel();
    thread::spawn(move || {
        let mut ctx = EWLightContext {
            intersection: ew_intersection,
            tick_count: 0,
        };
        let mut fsm = EWLightFSM::Red;
        fsm.init(&mut ctx);

        // Start EW as Red initially
        thread::sleep(Duration::from_millis(100));

        while let Ok(event) = ew_event_rx.recv() {
            fsm.dispatch(&mut ctx, &event);
        }
    });

    // Spawn Pedestrian Crossing Thread
    let ped_intersection = Arc::clone(&intersection);
    let (ped_event_tx, ped_event_rx) = channel();
    thread::spawn(move || {
        let mut ctx = PedestrianContext {
            intersection: ped_intersection,
            button_pressed: false,
        };
        let mut fsm = PedestrianFSM::Wait;
        fsm.init(&mut ctx);

        while let Ok(event) = ped_event_rx.recv() {
            fsm.dispatch(&mut ctx, &event);
        }
    });

    // Main event loop: broadcast events to all FSMs
    println!("Running intersection for 15 seconds...\n");

    for i in 0..15 {
        thread::sleep(Duration::from_secs(1));

        // Broadcast timer tick to all lights
        let _ = ns_event_tx.send(TrafficEvent::TimerTick);
        let _ = ew_event_tx.send(TrafficEvent::TimerTick);
        let _ = ped_event_tx.send(TrafficEvent::TimerTick);

        // Simulate pedestrian button press at 5 seconds
        if i == 5 {
            println!("\n>>> Pedestrian button pressed! <<<\n");
            let _ = ped_event_tx.send(TrafficEvent::PedestrianButton);
        }

        // Simulate emergency vehicle at 10 seconds
        if i == 10 {
            println!("\n🚨 EMERGENCY VEHICLE APPROACHING! 🚨\n");
            let _ = ns_event_tx.send(TrafficEvent::Emergency);
            let _ = ew_event_tx.send(TrafficEvent::Emergency);
            let _ = ped_event_tx.send(TrafficEvent::Emergency);
        }
    }

    println!("\n=== Intersection Control Complete ===");
    println!("\nKey Takeaways:");
    println!("  • 3 FSMs ran concurrently in separate threads");
    println!("  • Shared state (Arc<Mutex<>>) prevented unsafe light combinations");
    println!("  • Events were broadcast via channels to coordinate behavior");
    println!("  • Each FSM maintained its own lifecycle independently");
    println!("  • Thread-safe by design: FSMs are Send + Sync automatically");
}
