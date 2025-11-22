//! Network Connection Manager Example
//!
//! This example demonstrates a realistic network connection state machine.
//! It showcases:
//! - Connection lifecycle management
//! - Retry logic with backoff
//! - Error handling and recovery
//! - Stateful states with connection metadata

use typed_fsm::{state_machine, Transition};

// ============================================================================
// 1. Context (Shared State)
// ============================================================================

/// Represents the connection manager's shared state.
#[derive(Debug)]
pub struct ConnectionContext {
    /// Server address
    pub server: String,
    /// Number of connection attempts
    pub attempt_count: u32,
    /// Maximum retry attempts before giving up
    pub max_retries: u32,
    /// Last error message (if any)
    pub last_error: Option<String>,
}

impl ConnectionContext {
    fn log(&self, msg: &str) {
        println!("[Connection to {}] {}", self.server, msg);
    }

    fn reset_attempts(&mut self) {
        self.attempt_count = 0;
        self.last_error = None;
    }
}

// ============================================================================
// 2. Events
// ============================================================================

/// Events that drive the connection state machine.
#[derive(Debug)]
pub enum ConnectionEvent {
    /// User initiated connect
    Connect,
    /// Connection succeeded
    ConnectionEstablished,
    /// Connection failed with error message
    ConnectionFailed(String),
    /// User initiated disconnect
    Disconnect,
    /// Connection lost unexpectedly
    ConnectionLost,
    /// Retry timer expired
    RetryTimeout,
}

// ============================================================================
// 3. State Machine Definition
// ============================================================================

state_machine! {
    Name: ConnectionManager,
    Context: ConnectionContext,
    Event: ConnectionEvent,

    States: {
        // --------------------------------------------------------------------
        // State: DISCONNECTED
        // Description: No active connection
        // --------------------------------------------------------------------
        Disconnected => {
            entry: |ctx| {
                ctx.log("Ready to connect");
                ctx.reset_attempts();
            }

            process: |_ctx, evt| {
                match evt {
                    ConnectionEvent::Connect => {
                        Transition::To(ConnectionManager::Connecting { attempt: 1 })
                    }
                    _ => Transition::None
                }
            }
        },

        // --------------------------------------------------------------------
        // State: CONNECTING
        // Description: Attempting to establish connection
        // --------------------------------------------------------------------
        Connecting { attempt: u32 } => {
            entry: |ctx| {
                ctx.attempt_count = *attempt;
                ctx.log(&format!("Connecting... (attempt {}/{})", attempt, ctx.max_retries));

                // In a real application, you would initiate async connection here
            }

            process: |ctx, evt| {
                match evt {
                    ConnectionEvent::ConnectionEstablished => {
                        Transition::To(ConnectionManager::Connected {
                            session_id: format!("session-{}", ctx.attempt_count),
                        })
                    }
                    ConnectionEvent::ConnectionFailed(error) => {
                        ctx.last_error = Some(error.clone());

                        if ctx.attempt_count >= ctx.max_retries {
                            ctx.log(&format!("Connection failed after {} attempts", ctx.max_retries));
                            Transition::To(ConnectionManager::Failed {
                                error: error.clone(),
                            })
                        } else {
                            ctx.log(&format!("Connection attempt failed: {}", error));
                            Transition::To(ConnectionManager::Retrying {
                                next_attempt: ctx.attempt_count + 1,
                                backoff_ms: 1000 * ctx.attempt_count, // Exponential backoff
                            })
                        }
                    }
                    ConnectionEvent::Disconnect => {
                        ctx.log("Connection cancelled by user");
                        Transition::To(ConnectionManager::Disconnected)
                    }
                    _ => Transition::None
                }
            }
        },

        // --------------------------------------------------------------------
        // State: CONNECTED
        // Description: Active connection established
        // --------------------------------------------------------------------
        Connected { session_id: String } => {
            entry: |ctx| {
                ctx.log(&format!("Connected successfully! Session: {}", session_id));
                ctx.reset_attempts();
            }

            process: |ctx, evt| {
                match evt {
                    ConnectionEvent::Disconnect => {
                        ctx.log("Disconnecting gracefully...");
                        Transition::To(ConnectionManager::Disconnected)
                    }
                    ConnectionEvent::ConnectionLost => {
                        ctx.log("Connection lost unexpectedly!");
                        Transition::To(ConnectionManager::Retrying {
                            next_attempt: 1,
                            backoff_ms: 500,
                        })
                    }
                    _ => Transition::None
                }
            }

            exit: |ctx| {
                ctx.log("Closing connection...");
                // In a real application, clean up resources here
            }
        },

        // --------------------------------------------------------------------
        // State: RETRYING
        // Description: Waiting before retry attempt
        // --------------------------------------------------------------------
        Retrying {
            next_attempt: u32,
            backoff_ms: u32
        } => {
            entry: |ctx| {
                ctx.log(&format!(
                    "Retrying in {}ms... (next attempt: {}/{})",
                    backoff_ms, next_attempt, ctx.max_retries
                ));
            }

            process: |_ctx, evt| {
                match evt {
                    ConnectionEvent::RetryTimeout => {
                        Transition::To(ConnectionManager::Connecting {
                            attempt: *next_attempt,
                        })
                    }
                    ConnectionEvent::Disconnect => {
                        Transition::To(ConnectionManager::Disconnected)
                    }
                    _ => Transition::None
                }
            }
        },

        // --------------------------------------------------------------------
        // State: FAILED
        // Description: Connection failed permanently
        // --------------------------------------------------------------------
        Failed { error: String } => {
            entry: |ctx| {
                ctx.log(&format!("Connection failed permanently: {}", error));
                ctx.log("Manual intervention required");
            }

            process: |_ctx, evt| {
                match evt {
                    ConnectionEvent::Connect => {
                        // Allow retry from failed state
                        Transition::To(ConnectionManager::Connecting { attempt: 1 })
                    }
                    _ => Transition::None
                }
            }
        }
    }
}

// ============================================================================
// 4. Simulation
// ============================================================================

fn main() {
    println!("=== Network Connection Manager ===\n");

    // Initialize context
    let mut ctx = ConnectionContext {
        server: "api.example.com:443".to_string(),
        attempt_count: 0,
        max_retries: 3,
        last_error: None,
    };

    // Start in Disconnected state
    let mut connection = ConnectionManager::Disconnected;
    connection.init(&mut ctx);

    println!("\n--- Scenario 1: Successful connection after retry ---\n");

    // Attempt to connect
    connection.dispatch(&mut ctx, &ConnectionEvent::Connect);

    // Simulate first failure
    connection.dispatch(&mut ctx, &ConnectionEvent::ConnectionFailed(
        "Timeout".to_string(),
    ));

    // Simulate retry timeout
    connection.dispatch(&mut ctx, &ConnectionEvent::RetryTimeout);

    // Simulate successful connection
    connection.dispatch(&mut ctx, &ConnectionEvent::ConnectionEstablished);

    println!("\n--- Scenario 2: Connection loss and recovery ---\n");

    // Simulate connection lost
    connection.dispatch(&mut ctx, &ConnectionEvent::ConnectionLost);

    // Retry timeout
    connection.dispatch(&mut ctx, &ConnectionEvent::RetryTimeout);

    // Successful reconnection
    connection.dispatch(&mut ctx, &ConnectionEvent::ConnectionEstablished);

    println!("\n--- Scenario 3: Graceful disconnect ---\n");

    // User disconnects
    connection.dispatch(&mut ctx, &ConnectionEvent::Disconnect);

    println!("\n--- Simulation Complete ---");
}
