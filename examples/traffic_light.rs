//! Traffic Light Example
//!
//! This is a simple example demonstrating a traffic light controller.
//! It showcases:
//! - Simple state transitions without complex logic
//! - Timer-based events
//! - Entry actions for visual feedback

use typed_fsm::{state_machine, Transition};

// ============================================================================
// 1. Context (Shared State)
// ============================================================================

/// Represents the traffic light system context.
/// In a real system, this would control actual lights via GPIO pins.
#[derive(Debug)]
pub struct TrafficLightContext {
    /// Total cycles completed
    pub cycle_count: u32,
}

impl TrafficLightContext {
    fn log(&self, msg: &str) {
        println!("[Cycle #{}] {}", self.cycle_count, msg);
    }
}

// ============================================================================
// 2. Events
// ============================================================================

/// Events that drive the traffic light state machine.
#[derive(Debug)]
pub enum Event {
    /// Timer elapsed, move to next state
    TimerTick,
}

// ============================================================================
// 3. State Machine Definition
// ============================================================================

state_machine! {
    Name: TrafficLight,
    Context: TrafficLightContext,
    Event: Event,

    States: {
        // --------------------------------------------------------------------
        // State: GREEN
        // Description: Vehicles can proceed
        // --------------------------------------------------------------------
        Green => {
            entry: |ctx| {
                ctx.log("🟢 GREEN - Go!");
            }

            process: |_ctx, evt| {
                match evt {
                    Event::TimerTick => Transition::To(TrafficLight::Yellow),
                }
            }
        },

        // --------------------------------------------------------------------
        // State: YELLOW
        // Description: Vehicles should prepare to stop
        // --------------------------------------------------------------------
        Yellow => {
            entry: |ctx| {
                ctx.log("🟡 YELLOW - Caution!");
            }

            process: |_ctx, evt| {
                match evt {
                    Event::TimerTick => Transition::To(TrafficLight::Red),
                }
            }
        },

        // --------------------------------------------------------------------
        // State: RED
        // Description: Vehicles must stop
        // --------------------------------------------------------------------
        Red => {
            entry: |ctx| {
                ctx.log("🔴 RED - Stop!");
            }

            process: |ctx, evt| {
                match evt {
                    Event::TimerTick => {
                        // Increment cycle counter before transitioning back to Green
                        ctx.cycle_count += 1;
                        Transition::To(TrafficLight::Green)
                    }
                }
            }
        }
    }
}

// ============================================================================
// 4. Main Loop (Simulation)
// ============================================================================

fn main() {
    println!("=== Traffic Light Controller ===\n");

    // Initialize context
    let mut ctx = TrafficLightContext { cycle_count: 0 };

    // Start in Green state
    let mut light = TrafficLight::Green;

    // Initialize the state machine
    light.init(&mut ctx);

    println!("\n--- Starting traffic light cycle ---\n");

    // Simulate 10 timer ticks (more than 3 full cycles)
    for i in 1..=10 {
        println!("Timer tick #{}", i);
        light.dispatch(&mut ctx, &Event::TimerTick);
        println!();

        // Simulate delay between ticks
        std::thread::sleep(std::time::Duration::from_millis(800));
    }

    println!("--- Simulation Complete ---");
    println!("Total cycles completed: {}", ctx.cycle_count);
}
