//! # Hierarchical State Machine Example
//!
//! This example demonstrates a **nested (hierarchical) state machine** pattern
//! where one state machine contains another as part of its context.
//!
//! ## Use Case: Audio Player with Volume Control
//!
//! - **Player FSM** (top-level): Controls playback (Stopped, Playing, Paused)
//! - **Volume FSM** (nested): Controls volume levels (Low, Medium, High) - only active when Playing
//!
//! This pattern is useful when:
//! - Substates only make sense within a parent state
//! - You want to encapsulate related state logic
//! - Different contexts require different state machines
//!
//! Run with: `cargo run --example hierarchical`

use typed_fsm::{state_machine, Transition};

// ============================================================================
// Volume FSM (Nested State Machine)
// ============================================================================

#[derive(Debug)]
struct VolumeContext {
    level: u8,
}

#[derive(Debug)]
enum VolumeEvent {
    VolumeUp,
    VolumeDown,
}

state_machine! {
    Name: VolumeFSM,
    Context: VolumeContext,
    Event: VolumeEvent,

    States: {
        Low => {
            entry: |ctx| {
                ctx.level = 30;
                println!("  🔉 Volume: LOW ({}%)", ctx.level);
            }

            process: |_ctx, evt| {
                match evt {
                    VolumeEvent::VolumeUp => Transition::To(VolumeFSM::Medium),
                    VolumeEvent::VolumeDown => Transition::None,
                }
            }
        },

        Medium => {
            entry: |ctx| {
                ctx.level = 60;
                println!("  🔊 Volume: MEDIUM ({}%)", ctx.level);
            }

            process: |_ctx, evt| {
                match evt {
                    VolumeEvent::VolumeUp => Transition::To(VolumeFSM::High),
                    VolumeEvent::VolumeDown => Transition::To(VolumeFSM::Low),
                }
            }
        },

        High => {
            entry: |ctx| {
                ctx.level = 100;
                println!("  🔊 Volume: HIGH ({}%)", ctx.level);
            }

            process: |_ctx, evt| {
                match evt {
                    VolumeEvent::VolumeUp => Transition::None,
                    VolumeEvent::VolumeDown => Transition::To(VolumeFSM::Medium),
                }
            }
        }
    }
}

// ============================================================================
// Player FSM (Top-Level State Machine)
// ============================================================================

#[derive(Debug)]
struct PlayerContext {
    track_name: String,
    position: u32,
    // Nested state machine: only used when Playing
    volume_fsm: Option<VolumeFSM>,
    volume_ctx: VolumeContext,
}

#[derive(Debug)]
enum PlayerEvent {
    Play,
    Pause,
    Stop,
    VolumeChange(VolumeEvent),
}

state_machine! {
    Name: PlayerFSM,
    Context: PlayerContext,
    Event: PlayerEvent,

    States: {
        Stopped => {
            entry: |ctx| {
                ctx.position = 0;
                ctx.volume_fsm = None;
                println!("⏹️  Player STOPPED");
            }

            process: |_ctx, evt| {
                match evt {
                    PlayerEvent::Play => Transition::To(PlayerFSM::Playing),
                    _ => Transition::None
                }
            }
        },

        Playing => {
            entry: |ctx| {
                println!("▶️  Playing: {}", ctx.track_name);

                // Initialize nested volume FSM when entering Playing state
                let mut volume_fsm = VolumeFSM::Medium;
                volume_fsm.init(&mut ctx.volume_ctx);
                ctx.volume_fsm = Some(volume_fsm);
            }

            process: |ctx, evt| {
                match evt {
                    PlayerEvent::Pause => Transition::To(PlayerFSM::Paused),
                    PlayerEvent::Stop => Transition::To(PlayerFSM::Stopped),

                    // Delegate volume events to nested FSM
                    PlayerEvent::VolumeChange(vol_evt) => {
                        if let Some(ref mut volume_fsm) = ctx.volume_fsm {
                            volume_fsm.dispatch(&mut ctx.volume_ctx, vol_evt);
                        }
                        Transition::None
                    },

                    _ => Transition::None
                }
            }

            exit: |ctx| {
                // Cleanup nested FSM when leaving Playing state
                ctx.volume_fsm = None;
                println!("  (Volume controls disabled)");
            }
        },

        Paused => {
            entry: |ctx| {
                println!("⏸️  Player PAUSED at position {}", ctx.position);
            }

            process: |_ctx, evt| {
                match evt {
                    PlayerEvent::Play => Transition::To(PlayerFSM::Playing),
                    PlayerEvent::Stop => Transition::To(PlayerFSM::Stopped),
                    _ => Transition::None
                }
            }
        }
    }
}

fn main() {
    println!("=== Hierarchical State Machine Example ===");
    println!("Audio Player with Nested Volume Control\n");

    let mut ctx = PlayerContext {
        track_name: "Beethoven - Symphony No. 9".to_string(),
        position: 0,
        volume_fsm: None,
        volume_ctx: VolumeContext { level: 0 },
    };

    let mut player = PlayerFSM::Stopped;
    player.init(&mut ctx);

    println!("\n--- Scenario: Play music and adjust volume ---\n");

    // Start playing (activates nested volume FSM)
    player.dispatch(&mut ctx, &PlayerEvent::Play);

    // Volume controls only work when Playing
    player.dispatch(&mut ctx, &PlayerEvent::VolumeChange(VolumeEvent::VolumeUp));
    player.dispatch(&mut ctx, &PlayerEvent::VolumeChange(VolumeEvent::VolumeUp));
    player.dispatch(
        &mut ctx,
        &PlayerEvent::VolumeChange(VolumeEvent::VolumeDown),
    );

    println!();

    // Pause (volume FSM still active)
    player.dispatch(&mut ctx, &PlayerEvent::Pause);
    ctx.position = 45;

    // Resume playing (reactivates volume FSM from initial state)
    player.dispatch(&mut ctx, &PlayerEvent::Play);

    // Volume controls work again
    player.dispatch(
        &mut ctx,
        &PlayerEvent::VolumeChange(VolumeEvent::VolumeDown),
    );
    player.dispatch(
        &mut ctx,
        &PlayerEvent::VolumeChange(VolumeEvent::VolumeDown),
    );

    println!();

    // Stop (deactivates volume FSM)
    player.dispatch(&mut ctx, &PlayerEvent::Stop);

    // Volume controls have no effect when Stopped
    println!("\nAttempting volume change while stopped (should have no effect):");
    player.dispatch(&mut ctx, &PlayerEvent::VolumeChange(VolumeEvent::VolumeUp));

    println!("\n=== Example Complete ===");
    println!("\nKey Takeaways:");
    println!("  • Nested FSM (VolumeFSM) only exists when parent is in Playing state");
    println!("  • Volume controls are automatically disabled when not Playing");
    println!("  • Each FSM manages its own lifecycle independently");
    println!("  • Context can hold optional nested FSMs for hierarchical behavior");
}
