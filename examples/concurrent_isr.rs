//! # Concurrent ISR Example
//!
//! This example demonstrates safe FSM usage with simulated Interrupt Service Routines (ISRs).
//!
//! ## Scenario
//! A sensor monitoring system with:
//! - **Main loop**: Processes commands and updates
//! - **Simulated Timer ISR**: Fires every 100ms to read sensor
//! - **Simulated Data ISR**: Fires when external data arrives
//!
//! ## Concurrency Safety
//!
//! The `concurrent` feature ensures:
//! - ISRs can safely call `dispatch()` while main loop is active
//! - Events are queued when dispatch is busy
//! - FIFO order is preserved
//! - No data races on FSM state or context
//!
//! ## Running
//!
//! ```bash
//! cargo run --example concurrent_isr --features concurrent
//! ```

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use typed_fsm::{state_machine, Transition};

// ============================================================================
// FSM Definition
// ============================================================================

struct SensorContext {
    sensor_value: u32,
    sample_count: u32,
    error_count: u32,
}

#[derive(Debug, Clone)]
enum SensorEvent {
    Start,
    Stop,
    TimerTick,      // From simulated timer ISR
    DataReady(u32), // From simulated data ISR
    #[allow(dead_code)]
    Error,
}

state_machine! {
    Name: SensorFSM,
    Context: SensorContext,
    Event: SensorEvent,
    States: {
        Idle => {
            entry: |ctx| {
                println!("[Idle] Sensor monitoring stopped");
                ctx.sample_count = 0;
            }

            process: |_ctx, evt| {
                match evt {
                    SensorEvent::Start => {
                        println!("[Idle] Starting sensor monitoring...");
                        Transition::To(SensorFSM::Monitoring)
                    }
                    _ => Transition::None
                }
            }
        },

        Monitoring => {
            entry: |_ctx| {
                println!("[Monitoring] Sensor monitoring active");
            }

            process: |ctx, evt| {
                match evt {
                    SensorEvent::TimerTick => {
                        // Simulated periodic sensor read (from ISR)
                        println!("[Monitoring] Timer tick (sample #{})", ctx.sample_count);
                        ctx.sample_count += 1;
                        Transition::None
                    }
                    SensorEvent::DataReady(value) => {
                        // Process sensor data (from ISR)
                        println!("[Monitoring] Data received: {}", value);
                        ctx.sensor_value = *value;

                        // Check for out-of-range values
                        if *value > 1000 {
                            println!("[Monitoring] ⚠️  Value out of range! Transitioning to Error");
                            Transition::To(SensorFSM::Error)
                        } else {
                            Transition::None
                        }
                    }
                    SensorEvent::Stop => {
                        println!("[Monitoring] Stop requested");
                        Transition::To(SensorFSM::Idle)
                    }
                    SensorEvent::Error => {
                        Transition::To(SensorFSM::Error)
                    }
                    _ => Transition::None
                }
            }

            exit: |ctx| {
                println!("[Monitoring] Collected {} samples", ctx.sample_count);
            }
        },

        Error => {
            entry: |ctx| {
                println!("[Error] ❌ Error state entered");
                ctx.error_count += 1;
            }

            process: |_ctx, evt| {
                match evt {
                    SensorEvent::Start => {
                        println!("[Error] Recovering from error...");
                        Transition::To(SensorFSM::Monitoring)
                    }
                    SensorEvent::Stop => {
                        Transition::To(SensorFSM::Idle)
                    }
                    _ => {
                        println!("[Error] Ignoring event while in error state");
                        Transition::None
                    }
                }
            }
        }
    }
}

// ============================================================================
// Simulated ISR Infrastructure
// ============================================================================

// Global state for ISR simulation
static ISR_ENABLED: AtomicBool = AtomicBool::new(false);
static SENSOR_DATA: AtomicU32 = AtomicU32::new(0);

// FSM and Context must be globally accessible for ISRs
// In real embedded: would be static mut or in interrupt-safe container
static FSM: Mutex<Option<SensorFSM>> = Mutex::new(None);
static CTX: Mutex<Option<SensorContext>> = Mutex::new(None);

/// Simulates a timer ISR that fires periodically
fn simulated_timer_isr() {
    thread::spawn(|| {
        loop {
            thread::sleep(Duration::from_millis(100));

            if ISR_ENABLED.load(Ordering::Relaxed) {
                // This is the ISR context - must be fast!
                println!("\n  [ISR:Timer] 🔔 Timer interrupt fired!");

                // Call dispatch from ISR - safe with concurrent feature
                if let (Ok(mut fsm_guard), Ok(mut ctx_guard)) = (FSM.lock(), CTX.lock()) {
                    if let (Some(fsm), Some(ctx)) = (fsm_guard.as_mut(), ctx_guard.as_mut()) {
                        fsm.dispatch(ctx, &SensorEvent::TimerTick);
                        println!("  [ISR:Timer] ✅ Event dispatched\n");
                    }
                }
            }
        }
    });
}

/// Simulates a data-ready ISR (e.g., ADC conversion complete)
fn simulated_data_isr() {
    thread::spawn(|| {
        let mut counter = 0u32;
        loop {
            thread::sleep(Duration::from_millis(250));

            if ISR_ENABLED.load(Ordering::Relaxed) {
                counter += 1;
                let value = (counter * 137) % 1200; // Simulated sensor reading
                SENSOR_DATA.store(value, Ordering::Relaxed);

                println!("\n  [ISR:Data] 📊 Data interrupt fired! Value={}", value);

                // Call dispatch from ISR
                if let (Ok(mut fsm_guard), Ok(mut ctx_guard)) = (FSM.lock(), CTX.lock()) {
                    if let (Some(fsm), Some(ctx)) = (fsm_guard.as_mut(), ctx_guard.as_mut()) {
                        fsm.dispatch(ctx, &SensorEvent::DataReady(value));
                        println!("  [ISR:Data] ✅ Event dispatched\n");
                    }
                }
            }
        }
    });
}

// ============================================================================
// Main Application
// ============================================================================

fn main() {
    println!("========================================");
    println!("  Concurrent ISR Example");
    println!("========================================");
    println!("Demonstrates safe FSM usage with ISRs");
    println!("Feature 'concurrent' MUST be enabled!\n");

    // Initialize FSM and Context
    let mut fsm = SensorFSM::Idle;
    let mut ctx = SensorContext {
        sensor_value: 0,
        sample_count: 0,
        error_count: 0,
    };

    // Initialize FSM
    fsm.init(&mut ctx);

    // Move to global storage for ISR access
    *FSM.lock().unwrap() = Some(fsm);
    *CTX.lock().unwrap() = Some(ctx);

    // Start simulated ISRs
    println!("Starting simulated ISRs...\n");
    simulated_timer_isr();
    simulated_data_isr();

    // Wait for threads to start
    thread::sleep(Duration::from_millis(50));

    // Main loop - processes commands
    println!("\n[Main] Starting sensor...");
    if let (Ok(mut fsm_guard), Ok(mut ctx_guard)) = (FSM.lock(), CTX.lock()) {
        if let (Some(fsm), Some(ctx)) = (fsm_guard.as_mut(), ctx_guard.as_mut()) {
            fsm.dispatch(ctx, &SensorEvent::Start);
        }
    }

    // Enable ISRs
    ISR_ENABLED.store(true, Ordering::Relaxed);

    // Let it run for a while
    println!("\n[Main] Monitoring for 2 seconds...\n");
    thread::sleep(Duration::from_secs(2));

    // Stop monitoring
    println!("\n[Main] Stopping sensor...");
    if let (Ok(mut fsm_guard), Ok(mut ctx_guard)) = (FSM.lock(), CTX.lock()) {
        if let (Some(fsm), Some(ctx)) = (fsm_guard.as_mut(), ctx_guard.as_mut()) {
            fsm.dispatch(ctx, &SensorEvent::Stop);
        }
    }

    // Disable ISRs
    ISR_ENABLED.store(false, Ordering::Relaxed);

    thread::sleep(Duration::from_millis(200));

    // Print statistics
    if let Ok(ctx_guard) = CTX.lock() {
        if let Some(ctx) = ctx_guard.as_ref() {
            println!("\n========================================");
            println!("  Statistics:");
            println!("========================================");
            println!("Total samples: {}", ctx.sample_count);
            println!("Last value: {}", ctx.sensor_value);
            println!("Errors: {}", ctx.error_count);
            println!("========================================\n");
        }
    }

    println!("✅ Example completed successfully!");
    println!("Notice how ISR events were safely queued when main was active.\n");
}
