//! # Timeouts Example: Time-Based State Transitions
//!
//! This example demonstrates the **Timer trait abstraction pattern** for implementing
//! timeouts, retries, and time-based behaviors in state machines.
//!
//! ## What are Timeouts?
//!
//! Timeouts allow states to automatically transition after a specified time duration.
//! This is essential for:
//! - Connection timeouts
//! - Retry mechanisms with exponential backoff
//! - Button debouncing
//! - Watchdog timers in embedded systems
//! - Session expiration
//!
//! ## Implementation Pattern
//!
//! typed-fsm maintains no_std compatibility by NOT providing built-in timer functionality.
//! Instead, we use a **Timer trait abstraction pattern**:
//!
//! 1. Define a `Timer` trait (user-provided or use this example)
//! 2. Store timer instances in the Context
//! 3. Check timeouts in entry/process hooks
//! 4. Implement the Timer trait for your platform (std, embedded, mock)
//!
//! This pattern is:
//! - **Zero-cost** - No overhead if you don't use it
//! - **no_std compatible** - Users implement for their platform
//! - **Completely optional** - Ignore if you don't need timeouts
//!
//! Run with: `cargo run --example timeouts`

use std::time::{Duration, Instant};
use typed_fsm::{state_machine, Transition};

// ============================================================================
// Timer Trait Abstraction
// ============================================================================

/// Timer trait that can be implemented for any platform
///
/// For std: Use Instant
/// For embedded: Use HAL timer peripherals
/// For testing: Use mock time
pub trait Timer {
    fn start(&mut self, duration_ms: u64);
    fn is_expired(&self) -> bool;
    fn reset(&mut self);
}

// ============================================================================
// std Implementation (using std::time::Instant)
// ============================================================================

#[derive(Debug)]
pub struct StdTimer {
    start_time: Option<Instant>,
    duration: Duration,
}

impl Default for StdTimer {
    fn default() -> Self {
        Self::new()
    }
}

impl StdTimer {
    pub fn new() -> Self {
        Self {
            start_time: None,
            duration: Duration::from_secs(0),
        }
    }
}

impl Timer for StdTimer {
    fn start(&mut self, duration_ms: u64) {
        self.start_time = Some(Instant::now());
        self.duration = Duration::from_millis(duration_ms);
    }

    fn is_expired(&self) -> bool {
        if let Some(start) = self.start_time {
            start.elapsed() >= self.duration
        } else {
            false
        }
    }

    fn reset(&mut self) {
        self.start_time = None;
    }
}

// ============================================================================
// Example 1: WiFi Connection with Timeout and Retry
// ============================================================================

struct WiFiContext {
    timer: StdTimer,
    retry_count: u32,
    max_retries: u32,
    connection_timeout_ms: u64,
    retry_delay_ms: u64,
}

#[derive(Debug, Clone)]
enum WiFiEvent {
    Connect,
    Connected,
    #[allow(dead_code)]
    Disconnect,
    CheckTimeout, // Polled event to check timeout
}

state_machine! {
    Name: WiFi,
    Context: WiFiContext,
    Event: WiFiEvent,

    States: {
        Idle => {
            entry: |ctx| {
                println!("WiFi: Idle. Ready to connect.");
                ctx.retry_count = 0;
            }

            process: |_ctx, evt| {
                match evt {
                    WiFiEvent::Connect => {
                        println!("WiFi: Starting connection...");
                        Transition::To(WiFi::Connecting)
                    }
                    _ => Transition::None
                }
            }
        },

        Connecting => {
            entry: |ctx| {
                println!("WiFi: Attempting connection (attempt {}/{})",
                         ctx.retry_count + 1, ctx.max_retries);
                // Start timeout timer
                ctx.timer.start(ctx.connection_timeout_ms);
            }

            process: |ctx, evt| {
                match evt {
                    WiFiEvent::Connected => {
                        println!("WiFi: Connection successful!");
                        ctx.timer.reset();
                        Transition::To(WiFi::Active)
                    }
                    WiFiEvent::CheckTimeout => {
                        // Check if timeout expired
                        if ctx.timer.is_expired() {
                            println!("WiFi: Connection timeout!");
                            ctx.timer.reset();
                            Transition::To(WiFi::Failed)
                        } else {
                            Transition::None
                        }
                    }
                    _ => Transition::None
                }
            }

            exit: |ctx| {
                ctx.timer.reset();
            }
        },

        Active => {
            entry: |_ctx| {
                println!("WiFi: Connected and active.");
            }

            process: |_ctx, evt| {
                match evt {
                    WiFiEvent::Disconnect => {
                        println!("WiFi: Disconnecting...");
                        Transition::To(WiFi::Idle)
                    }
                    _ => Transition::None
                }
            }
        },

        Failed => {
            entry: |ctx| {
                ctx.retry_count += 1;

                if ctx.retry_count < ctx.max_retries {
                    println!("WiFi: Retry #{} scheduled in {}ms",
                             ctx.retry_count + 1, ctx.retry_delay_ms);
                    // Start retry delay timer
                    ctx.timer.start(ctx.retry_delay_ms);
                } else {
                    println!("WiFi: Max retries ({}) exceeded. Giving up.", ctx.max_retries);
                }
            }

            process: |ctx, evt| {
                match evt {
                    WiFiEvent::CheckTimeout => {
                        // Check if retry delay expired
                        if ctx.timer.is_expired() && ctx.retry_count < ctx.max_retries {
                            println!("WiFi: Retry delay complete. Retrying...");
                            ctx.timer.reset();
                            Transition::To(WiFi::Connecting)
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
// Example 2: Session Timeout (Idle Detection)
// ============================================================================

struct SessionContext {
    timer: StdTimer,
    session_timeout_ms: u64,
    username: String,
}

#[derive(Debug, Clone)]
enum SessionEvent {
    Login {
        username: String,
    },
    Activity,
    CheckTimeout,
    #[allow(dead_code)]
    Logout,
}

state_machine! {
    Name: Session,
    Context: SessionContext,
    Event: SessionEvent,

    States: {
        LoggedOut => {
            entry: |ctx| {
                ctx.username.clear();
                println!("Session: Logged out.");
            }

            process: |ctx, evt| {
                match evt {
                    SessionEvent::Login { username } => {
                        ctx.username = username.clone();
                        println!("Session: User '{}' logged in.", username);
                        Transition::To(Session::Active)
                    }
                    _ => Transition::None
                }
            }
        },

        Active => {
            entry: |ctx| {
                println!("Session: Active session for '{}'.", ctx.username);
                // Start inactivity timeout
                ctx.timer.start(ctx.session_timeout_ms);
            }

            process: |ctx, evt| {
                match evt {
                    SessionEvent::Activity => {
                        println!("Session: User activity detected. Resetting timer.");
                        // Reset timeout on activity
                        ctx.timer.start(ctx.session_timeout_ms);
                        Transition::None
                    }
                    SessionEvent::CheckTimeout => {
                        // Check if session expired
                        if ctx.timer.is_expired() {
                            println!("Session: Timeout! User '{}' inactive.", ctx.username);
                            Transition::To(Session::LoggedOut)
                        } else {
                            Transition::None
                        }
                    }
                    SessionEvent::Logout => {
                        println!("Session: User '{}' logged out manually.", ctx.username);
                        Transition::To(Session::LoggedOut)
                    }
                    _ => Transition::None
                }
            }

            exit: |ctx| {
                ctx.timer.reset();
            }
        }
    }
}

// ============================================================================
// Example 3: Button Debouncing with Timeout
// ============================================================================

struct ButtonContext {
    timer: StdTimer,
    debounce_ms: u64,
    press_count: u32,
}

#[derive(Debug, Clone)]
enum ButtonEvent {
    Press,
    CheckTimeout,
}

state_machine! {
    Name: Button,
    Context: ButtonContext,
    Event: ButtonEvent,

    States: {
        Idle => {
            entry: |_ctx| {
                println!("Button: Ready");
            }

            process: |_ctx, evt| {
                match evt {
                    ButtonEvent::Press => {
                        println!("Button: First press detected");
                        Transition::To(Button::Debouncing)
                    }
                    _ => Transition::None
                }
            }
        },

        Debouncing => {
            entry: |ctx| {
                println!("Button: Debouncing for {}ms...", ctx.debounce_ms);
                ctx.timer.start(ctx.debounce_ms);
            }

            process: |ctx, evt| {
                match evt {
                    ButtonEvent::Press => {
                        // Ignore presses during debounce
                        println!("Button: Ignored (debouncing)");
                        Transition::None
                    }
                    ButtonEvent::CheckTimeout => {
                        if ctx.timer.is_expired() {
                            println!("Button: Debounce complete. Registering press #{}", ctx.press_count + 1);
                            ctx.press_count += 1;
                            Transition::To(Button::Pressed)
                        } else {
                            Transition::None
                        }
                    }
                }
            }
        },

        Pressed => {
            entry: |_ctx| {
                println!("Button: Press confirmed");
            }

            process: |_ctx, evt| {
                match evt {
                    ButtonEvent::Press => {
                        println!("Button: New press detected");
                        Transition::To(Button::Debouncing)
                    }
                    ButtonEvent::CheckTimeout => Transition::None
                }
            }
        }
    }
}

// ============================================================================
// Main: Run all examples
// ============================================================================

fn main() {
    println!("=== Timeouts Example: Time-Based State Transitions ===\n");

    println!("--- Example 1: WiFi Connection with Timeout and Retry ---\n");
    run_wifi_example();

    println!("\n--- Example 2: Session Timeout (Idle Detection) ---\n");
    run_session_example();

    println!("\n--- Example 3: Button Debouncing ---\n");
    run_button_example();

    println!("\n=== Key Takeaways ===");
    println!("1. Timeouts use Timer trait abstraction pattern (no built-in timer)");
    println!("2. Completely optional - zero impact on users who don't need it");
    println!("3. no_std compatible - users implement Timer for their platform");
    println!("4. Store timers in Context, check in process blocks");
    println!("5. Use polling pattern with CheckTimeout events");
    println!("6. Reset timers in entry/exit hooks as needed");
    println!("\nImplementation:");
    println!("  - For std: Use std::time::Instant (this example)");
    println!("  - For embedded: Use HAL timer peripherals");
    println!("  - For testing: Use mock time");
}

fn run_wifi_example() {
    let mut ctx = WiFiContext {
        timer: StdTimer::new(),
        retry_count: 0,
        max_retries: 3,
        connection_timeout_ms: 2000, // 2 second timeout
        retry_delay_ms: 1000,        // 1 second between retries
    };

    let mut wifi = WiFi::Idle;
    wifi.init(&mut ctx);

    // Start connection
    wifi.dispatch(&mut ctx, &WiFiEvent::Connect);

    // Simulate timeout by advancing time
    std::thread::sleep(std::time::Duration::from_millis(500));
    wifi.dispatch(&mut ctx, &WiFiEvent::CheckTimeout); // Not expired yet

    std::thread::sleep(std::time::Duration::from_millis(1600));
    wifi.dispatch(&mut ctx, &WiFiEvent::CheckTimeout); // Timeout! (total 2.1s)

    // Wait for retry delay
    std::thread::sleep(std::time::Duration::from_millis(1100));
    wifi.dispatch(&mut ctx, &WiFiEvent::CheckTimeout); // Retry delay complete

    // This time, simulate successful connection
    std::thread::sleep(std::time::Duration::from_millis(100));
    wifi.dispatch(&mut ctx, &WiFiEvent::Connected);
}

fn run_session_example() {
    let mut ctx = SessionContext {
        timer: StdTimer::new(),
        session_timeout_ms: 3000, // 3 second timeout
        username: String::new(),
    };

    let mut session = Session::LoggedOut;
    session.init(&mut ctx);

    // User logs in
    session.dispatch(
        &mut ctx,
        &SessionEvent::Login {
            username: "alice".to_string(),
        },
    );

    // User activity (resets timer)
    std::thread::sleep(std::time::Duration::from_millis(1000));
    session.dispatch(&mut ctx, &SessionEvent::Activity);

    // More activity
    std::thread::sleep(std::time::Duration::from_millis(1500));
    session.dispatch(&mut ctx, &SessionEvent::Activity);

    // No activity - timeout
    std::thread::sleep(std::time::Duration::from_millis(3100));
    session.dispatch(&mut ctx, &SessionEvent::CheckTimeout); // Timeout!
}

fn run_button_example() {
    let mut ctx = ButtonContext {
        timer: StdTimer::new(),
        debounce_ms: 500, // 500ms debounce
        press_count: 0,
    };

    let mut button = Button::Idle;
    button.init(&mut ctx);

    // First press
    button.dispatch(&mut ctx, &ButtonEvent::Press);

    // Rapid presses during debounce (ignored)
    std::thread::sleep(std::time::Duration::from_millis(100));
    button.dispatch(&mut ctx, &ButtonEvent::Press);

    std::thread::sleep(std::time::Duration::from_millis(100));
    button.dispatch(&mut ctx, &ButtonEvent::Press);

    // Wait for debounce
    std::thread::sleep(std::time::Duration::from_millis(350));
    button.dispatch(&mut ctx, &ButtonEvent::CheckTimeout); // Debounce complete

    // Second press
    std::thread::sleep(std::time::Duration::from_millis(100));
    button.dispatch(&mut ctx, &ButtonEvent::Press);

    std::thread::sleep(std::time::Duration::from_millis(550));
    button.dispatch(&mut ctx, &ButtonEvent::CheckTimeout); // Second press confirmed
}
