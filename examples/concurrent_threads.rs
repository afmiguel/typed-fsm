//! # Concurrent Multithreading Example
//!
//! This example demonstrates safe FSM usage with multiple concurrent threads.
//!
//! ## Scenario
//! A task processing system with:
//! - **Producer Thread**: Generates tasks
//! - **Monitor Thread**: Checks status and pauses/resumes
//! - **Main Thread**: Processes completions
//!
//! ## Concurrency Safety
//!
//! The `concurrent` feature ensures:
//! - Multiple threads can safely call `dispatch()` simultaneously
//! - Events are queued when dispatch is busy
//! - No data races on FSM state or context
//! - FIFO order is preserved
//!
//! ## Running
//!
//! ```bash
//! cargo run --example concurrent_threads --features concurrent
//! ```

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use typed_fsm::{state_machine, Transition};

// ============================================================================
// FSM Definition
// ============================================================================

struct TaskContext {
    active_tasks: u32,
    completed_tasks: u32,
    failed_tasks: u32,
}

#[derive(Debug, Clone)]
enum TaskEvent {
    NewTask,
    TaskComplete,
    TaskFailed,
    Pause,
    Resume,
    Shutdown,
}

state_machine! {
    Name: TaskProcessor,
    Context: TaskContext,
    Event: TaskEvent,
    States: {
        Idle => {
            entry: |ctx| {
                println!("[Idle] Task processor ready");
                ctx.active_tasks = 0;
            }

            process: |_ctx, evt| {
                match evt {
                    TaskEvent::NewTask => {
                        println!("[Idle] First task received, starting...");
                        Transition::To(TaskProcessor::Processing)
                    }
                    TaskEvent::Shutdown => {
                        println!("[Idle] Shutdown requested");
                        Transition::To(TaskProcessor::Shutdown)
                    }
                    _ => {
                        println!("[Idle] Ignoring event (not active)");
                        Transition::None
                    }
                }
            }
        },

        Processing => {
            entry: |_ctx| {
                println!("[Processing] ▶️  Task processing active");
            }

            process: |ctx, evt| {
                match evt {
                    TaskEvent::NewTask => {
                        ctx.active_tasks += 1;
                        println!("[Processing] 📥 New task queued (active: {})", ctx.active_tasks);
                        Transition::None
                    }
                    TaskEvent::TaskComplete => {
                        if ctx.active_tasks > 0 {
                            ctx.active_tasks -= 1;
                        }
                        ctx.completed_tasks += 1;
                        println!("[Processing] ✅ Task completed (active: {}, total: {})",
                                 ctx.active_tasks, ctx.completed_tasks);

                        if ctx.active_tasks == 0 {
                            println!("[Processing] All tasks complete, returning to Idle");
                            Transition::To(TaskProcessor::Idle)
                        } else {
                            Transition::None
                        }
                    }
                    TaskEvent::TaskFailed => {
                        if ctx.active_tasks > 0 {
                            ctx.active_tasks -= 1;
                        }
                        ctx.failed_tasks += 1;
                        println!("[Processing] ❌ Task failed (active: {}, failed: {})",
                                 ctx.active_tasks, ctx.failed_tasks);

                        if ctx.failed_tasks >= 3 {
                            println!("[Processing] Too many failures! Pausing...");
                            Transition::To(TaskProcessor::Paused)
                        } else {
                            Transition::None
                        }
                    }
                    TaskEvent::Pause => {
                        println!("[Processing] Pause requested");
                        Transition::To(TaskProcessor::Paused)
                    }
                    TaskEvent::Shutdown => {
                        println!("[Processing] Shutdown requested");
                        Transition::To(TaskProcessor::Shutdown)
                    }
                    _ => Transition::None
                }
            }

            exit: |ctx| {
                println!("[Processing] Exiting (active tasks: {})", ctx.active_tasks);
            }
        },

        Paused => {
            entry: |_ctx| {
                println!("[Paused] ⏸️  Task processing paused");
            }

            process: |ctx, evt| {
                match evt {
                    TaskEvent::Resume => {
                        println!("[Paused] Resuming processing...");
                        ctx.failed_tasks = 0;  // Reset failure counter
                        Transition::To(TaskProcessor::Processing)
                    }
                    TaskEvent::Shutdown => {
                        println!("[Paused] Shutdown requested");
                        Transition::To(TaskProcessor::Shutdown)
                    }
                    _ => {
                        println!("[Paused] Ignoring event (paused)");
                        Transition::None
                    }
                }
            }
        },

        Shutdown => {
            entry: |ctx| {
                println!("[Shutdown] 🛑 Shutting down");
                println!("  - Completed: {}", ctx.completed_tasks);
                println!("  - Failed: {}", ctx.failed_tasks);
                println!("  - Active: {}", ctx.active_tasks);
            }

            process: |_ctx, _evt| {
                // Ignore all events in shutdown state
                Transition::None
            }
        }
    }
}

// ============================================================================
// Worker Threads
// ============================================================================

fn producer_thread(fsm: Arc<Mutex<TaskProcessor>>, ctx: Arc<Mutex<TaskContext>>) {
    println!("\n[Thread:Producer] Started");

    for i in 1..=10 {
        thread::sleep(Duration::from_millis(150));

        let mut fsm_guard = fsm.lock().unwrap();
        let mut ctx_guard = ctx.lock().unwrap();

        println!("\n[Thread:Producer] Dispatching NewTask #{}", i);
        fsm_guard.dispatch(&mut ctx_guard, &TaskEvent::NewTask);
    }

    println!("\n[Thread:Producer] Done generating tasks");
}

fn monitor_thread(fsm: Arc<Mutex<TaskProcessor>>, ctx: Arc<Mutex<TaskContext>>) {
    println!("\n[Thread:Monitor] Started");

    thread::sleep(Duration::from_millis(600));

    // Pause after a while
    {
        let mut fsm_guard = fsm.lock().unwrap();
        let mut ctx_guard = ctx.lock().unwrap();

        println!("\n[Thread:Monitor] ⚠️  Pausing system for maintenance");
        fsm_guard.dispatch(&mut ctx_guard, &TaskEvent::Pause);
    }

    thread::sleep(Duration::from_millis(500));

    // Resume
    {
        let mut fsm_guard = fsm.lock().unwrap();
        let mut ctx_guard = ctx.lock().unwrap();

        println!("\n[Thread:Monitor] ▶️  Resuming system");
        fsm_guard.dispatch(&mut ctx_guard, &TaskEvent::Resume);
    }

    println!("\n[Thread:Monitor] Done");
}

fn worker_thread(fsm: Arc<Mutex<TaskProcessor>>, ctx: Arc<Mutex<TaskContext>>) {
    println!("\n[Thread:Worker] Started");

    for i in 1..=8 {
        thread::sleep(Duration::from_millis(200));

        let mut fsm_guard = fsm.lock().unwrap();
        let mut ctx_guard = ctx.lock().unwrap();

        // Simulate occasional failures
        if i % 7 == 0 {
            println!("\n[Thread:Worker] Dispatching TaskFailed #{}", i);
            fsm_guard.dispatch(&mut ctx_guard, &TaskEvent::TaskFailed);
        } else {
            println!("\n[Thread:Worker] Dispatching TaskComplete #{}", i);
            fsm_guard.dispatch(&mut ctx_guard, &TaskEvent::TaskComplete);
        }
    }

    println!("\n[Thread:Worker] Done processing tasks");
}

// ============================================================================
// Main Application
// ============================================================================

fn main() {
    println!("========================================");
    println!("  Concurrent Multithreading Example");
    println!("========================================");
    println!("Demonstrates safe FSM with multiple threads");
    println!("Feature 'concurrent' MUST be enabled!\n");

    // Initialize FSM and Context
    let mut fsm = TaskProcessor::Idle;
    let mut ctx = TaskContext {
        active_tasks: 0,
        completed_tasks: 0,
        failed_tasks: 0,
    };

    // Initialize FSM
    fsm.init(&mut ctx);

    // Wrap in Arc<Mutex> for thread sharing
    let fsm = Arc::new(Mutex::new(fsm));
    let ctx = Arc::new(Mutex::new(ctx));

    // Spawn worker threads
    let producer = {
        let fsm = Arc::clone(&fsm);
        let ctx = Arc::clone(&ctx);
        thread::spawn(move || producer_thread(fsm, ctx))
    };

    let monitor = {
        let fsm = Arc::clone(&fsm);
        let ctx = Arc::clone(&ctx);
        thread::spawn(move || monitor_thread(fsm, ctx))
    };

    let worker = {
        let fsm = Arc::clone(&fsm);
        let ctx = Arc::clone(&ctx);
        thread::spawn(move || worker_thread(fsm, ctx))
    };

    // Wait for all threads to complete
    println!("\n[Main] Waiting for worker threads...\n");

    producer.join().unwrap();
    monitor.join().unwrap();
    worker.join().unwrap();

    // Shutdown
    thread::sleep(Duration::from_millis(200));
    {
        let mut fsm_guard = fsm.lock().unwrap();
        let mut ctx_guard = ctx.lock().unwrap();

        println!("\n[Main] Initiating shutdown...");
        fsm_guard.dispatch(&mut ctx_guard, &TaskEvent::Shutdown);
    }

    // Print final statistics
    {
        let ctx_guard = ctx.lock().unwrap();
        println!("\n========================================");
        println!("  Final Statistics:");
        println!("========================================");
        println!("Completed tasks: {}", ctx_guard.completed_tasks);
        println!("Failed tasks: {}", ctx_guard.failed_tasks);
        println!("Active tasks: {}", ctx_guard.active_tasks);
        println!("========================================\n");
    }

    println!("✅ Example completed successfully!");
    println!("Notice how events from multiple threads were safely processed.\n");
}
