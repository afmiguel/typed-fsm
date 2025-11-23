//! Tests for Timeouts (Timer trait pattern)
//!
//! This test suite validates the Timer trait abstraction pattern:
//! - Mock timer implementation for testing
//! - Timeout detection and transitions
//! - Timer reset functionality
//! - Multiple timeouts in sequence
//! - Retry logic with timeouts

use typed_fsm::{state_machine, Transition};

// ============================================================================
// Mock Timer for Testing (deterministic, no real time delays)
// ============================================================================

pub trait Timer {
    fn start(&mut self, duration_ms: u64);
    fn is_expired(&self) -> bool;
    fn reset(&mut self);
}

#[derive(Debug, Clone)]
pub struct MockTimer {
    remaining_ms: u64,
    is_running: bool,
}

impl Default for MockTimer {
    fn default() -> Self {
        Self::new()
    }
}

impl MockTimer {
    pub fn new() -> Self {
        Self {
            remaining_ms: 0,
            is_running: false,
        }
    }

    /// Simulate time passing (for tests)
    pub fn tick(&mut self, ms: u64) {
        if self.is_running && self.remaining_ms > 0 {
            if ms >= self.remaining_ms {
                self.remaining_ms = 0;
            } else {
                self.remaining_ms -= ms;
            }
        }
    }
}

impl Timer for MockTimer {
    fn start(&mut self, duration_ms: u64) {
        self.remaining_ms = duration_ms;
        self.is_running = true;
    }

    fn is_expired(&self) -> bool {
        self.is_running && self.remaining_ms == 0
    }

    fn reset(&mut self) {
        self.remaining_ms = 0;
        self.is_running = false;
    }
}

// ============================================================================
// Test 1: Basic Timeout
// ============================================================================

struct ConnectionContext {
    timer: MockTimer,
    timeout_ms: u64,
}

#[derive(Debug, Clone)]
enum ConnEvent {
    Connect,
    Connected,
    CheckTimeout,
}

state_machine! {
    Name: Connection,
    Context: ConnectionContext,
    Event: ConnEvent,

    States: {
        Idle => {
            process: |_ctx, evt| {
                match evt {
                    ConnEvent::Connect => Transition::To(Connection::Connecting),
                    _ => Transition::None
                }
            }
        },

        Connecting => {
            entry: |ctx| {
                ctx.timer.start(ctx.timeout_ms);
            }

            process: |ctx, evt| {
                match evt {
                    ConnEvent::Connected => {
                        ctx.timer.reset();
                        Transition::To(Connection::Connected)
                    }
                    ConnEvent::CheckTimeout => {
                        if ctx.timer.is_expired() {
                            ctx.timer.reset();
                            Transition::To(Connection::Failed)
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

        Connected => {
            process: |_ctx, _evt| {
                Transition::None
            }
        },

        Failed => {
            process: |_ctx, _evt| {
                Transition::None
            }
        }
    }
}

#[test]
fn test_timeout_successful_connection_before_timeout() {
    let mut ctx = ConnectionContext {
        timer: MockTimer::new(),
        timeout_ms: 5000,
    };

    let mut conn = Connection::Idle;
    conn.init(&mut ctx);

    conn.dispatch(&mut ctx, &ConnEvent::Connect);
    assert!(matches!(conn, Connection::Connecting));

    // Simulate time passing (not expired yet)
    ctx.timer.tick(2000);
    conn.dispatch(&mut ctx, &ConnEvent::CheckTimeout);
    assert!(matches!(conn, Connection::Connecting));

    // Connect before timeout
    conn.dispatch(&mut ctx, &ConnEvent::Connected);
    assert!(matches!(conn, Connection::Connected));
}

#[test]
fn test_timeout_expires_and_fails() {
    let mut ctx = ConnectionContext {
        timer: MockTimer::new(),
        timeout_ms: 5000,
    };

    let mut conn = Connection::Idle;
    conn.init(&mut ctx);

    conn.dispatch(&mut ctx, &ConnEvent::Connect);

    // Simulate timeout expiration
    ctx.timer.tick(5000);
    conn.dispatch(&mut ctx, &ConnEvent::CheckTimeout);

    assert!(matches!(conn, Connection::Failed));
}

#[test]
fn test_timeout_reset_on_exit() {
    let mut ctx = ConnectionContext {
        timer: MockTimer::new(),
        timeout_ms: 5000,
    };

    let mut conn = Connection::Idle;
    conn.init(&mut ctx);

    conn.dispatch(&mut ctx, &ConnEvent::Connect);
    ctx.timer.tick(3000);

    // Connect (should reset timer via exit hook)
    conn.dispatch(&mut ctx, &ConnEvent::Connected);

    assert!(!ctx.timer.is_running);
    assert_eq!(ctx.timer.remaining_ms, 0);
}

// ============================================================================
// Test 2: Retry with Delays
// ============================================================================

struct RetryContext {
    timer: MockTimer,
    retry_count: u32,
    max_retries: u32,
    retry_delay_ms: u64,
}

#[derive(Debug, Clone)]
enum RetryEvent {
    Start,
    Success,
    CheckTimeout,
}

state_machine! {
    Name: RetryMachine,
    Context: RetryContext,
    Event: RetryEvent,

    States: {
        Idle => {
            entry: |ctx| {
                ctx.retry_count = 0;
            }

            process: |_ctx, evt| {
                match evt {
                    RetryEvent::Start => Transition::To(RetryMachine::Trying),
                    _ => Transition::None
                }
            }
        },

        Trying => {
            entry: |ctx| {
                ctx.timer.start(1000); // 1 second attempt timeout
            }

            process: |ctx, evt| {
                match evt {
                    RetryEvent::Success => {
                        ctx.timer.reset();
                        Transition::To(RetryMachine::Success)
                    }
                    RetryEvent::CheckTimeout => {
                        if ctx.timer.is_expired() {
                            ctx.timer.reset();
                            Transition::To(RetryMachine::Failed)
                        } else {
                            Transition::None
                        }
                    }
                    _ => Transition::None
                }
            }
        },

        Failed => {
            entry: |ctx| {
                ctx.retry_count += 1;
                if ctx.retry_count < ctx.max_retries {
                    ctx.timer.start(ctx.retry_delay_ms);
                }
            }

            process: |ctx, evt| {
                match evt {
                    RetryEvent::CheckTimeout => {
                        if ctx.timer.is_expired() && ctx.retry_count < ctx.max_retries {
                            ctx.timer.reset();
                            Transition::To(RetryMachine::Trying)
                        } else {
                            Transition::None
                        }
                    }
                    _ => Transition::None
                }
            }
        },

        Success => {
            process: |_ctx, _evt| {
                Transition::None
            }
        }
    }
}

#[test]
fn test_retry_succeeds_on_first_attempt() {
    let mut ctx = RetryContext {
        timer: MockTimer::new(),
        retry_count: 0,
        max_retries: 3,
        retry_delay_ms: 1000,
    };

    let mut machine = RetryMachine::Idle;
    machine.init(&mut ctx);

    machine.dispatch(&mut ctx, &RetryEvent::Start);
    assert!(matches!(machine, RetryMachine::Trying));

    machine.dispatch(&mut ctx, &RetryEvent::Success);
    assert!(matches!(machine, RetryMachine::Success));
    assert_eq!(ctx.retry_count, 0);
}

#[test]
fn test_retry_fails_then_retries() {
    let mut ctx = RetryContext {
        timer: MockTimer::new(),
        retry_count: 0,
        max_retries: 3,
        retry_delay_ms: 1000,
    };

    let mut machine = RetryMachine::Idle;
    machine.init(&mut ctx);

    machine.dispatch(&mut ctx, &RetryEvent::Start);

    // First attempt fails
    ctx.timer.tick(1000);
    machine.dispatch(&mut ctx, &RetryEvent::CheckTimeout);
    assert!(matches!(machine, RetryMachine::Failed));
    assert_eq!(ctx.retry_count, 1);

    // Wait for retry delay
    ctx.timer.tick(1000);
    machine.dispatch(&mut ctx, &RetryEvent::CheckTimeout);
    assert!(matches!(machine, RetryMachine::Trying));

    // Second attempt succeeds
    machine.dispatch(&mut ctx, &RetryEvent::Success);
    assert!(matches!(machine, RetryMachine::Success));
    assert_eq!(ctx.retry_count, 1);
}

#[test]
fn test_retry_max_attempts_exceeded() {
    let mut ctx = RetryContext {
        timer: MockTimer::new(),
        retry_count: 0,
        max_retries: 2,
        retry_delay_ms: 500,
    };

    let mut machine = RetryMachine::Idle;
    machine.init(&mut ctx);

    machine.dispatch(&mut ctx, &RetryEvent::Start);

    // Fail 3 times (max_retries = 2, so 2 retries after initial)
    for _ in 0..3 {
        ctx.timer.tick(1000); // Attempt timeout
        machine.dispatch(&mut ctx, &RetryEvent::CheckTimeout);

        if ctx.retry_count < ctx.max_retries {
            assert!(matches!(machine, RetryMachine::Failed));
            ctx.timer.tick(500); // Retry delay
            machine.dispatch(&mut ctx, &RetryEvent::CheckTimeout);
        }
    }

    assert!(matches!(machine, RetryMachine::Failed));
    assert_eq!(ctx.retry_count, 2);

    // Should not retry anymore
    ctx.timer.tick(500);
    machine.dispatch(&mut ctx, &RetryEvent::CheckTimeout);
    assert!(matches!(machine, RetryMachine::Failed));
}

// ============================================================================
// Test 3: Session Timeout (Inactivity)
// ============================================================================

struct SessionContext {
    timer: MockTimer,
    session_timeout_ms: u64,
    last_activity: u32,
}

#[derive(Debug, Clone)]
enum SessionEvent {
    Login,
    Activity,
    CheckTimeout,
    Logout,
}

state_machine! {
    Name: Session,
    Context: SessionContext,
    Event: SessionEvent,

    States: {
        LoggedOut => {
            process: |_ctx, evt| {
                match evt {
                    SessionEvent::Login => Transition::To(Session::Active),
                    _ => Transition::None
                }
            }
        },

        Active => {
            entry: |ctx| {
                ctx.timer.start(ctx.session_timeout_ms);
            }

            process: |ctx, evt| {
                match evt {
                    SessionEvent::Activity => {
                        // Reset timeout on activity
                        ctx.last_activity += 1;
                        ctx.timer.start(ctx.session_timeout_ms);
                        Transition::None
                    }
                    SessionEvent::CheckTimeout => {
                        if ctx.timer.is_expired() {
                            Transition::To(Session::LoggedOut)
                        } else {
                            Transition::None
                        }
                    }
                    SessionEvent::Logout => {
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

#[test]
fn test_session_activity_resets_timeout() {
    let mut ctx = SessionContext {
        timer: MockTimer::new(),
        session_timeout_ms: 3000,
        last_activity: 0,
    };

    let mut session = Session::LoggedOut;
    session.init(&mut ctx);

    session.dispatch(&mut ctx, &SessionEvent::Login);
    assert!(matches!(session, Session::Active));

    // Some time passes
    ctx.timer.tick(2000);

    // User activity (should reset timer)
    session.dispatch(&mut ctx, &SessionEvent::Activity);
    assert_eq!(ctx.last_activity, 1);

    // More time passes (would have timed out without activity)
    ctx.timer.tick(2000);
    session.dispatch(&mut ctx, &SessionEvent::CheckTimeout);

    // Should still be active (timer was reset)
    assert!(matches!(session, Session::Active));
}

#[test]
fn test_session_timeout_logs_out() {
    let mut ctx = SessionContext {
        timer: MockTimer::new(),
        session_timeout_ms: 3000,
        last_activity: 0,
    };

    let mut session = Session::LoggedOut;
    session.init(&mut ctx);

    session.dispatch(&mut ctx, &SessionEvent::Login);

    // Timeout expires
    ctx.timer.tick(3000);
    session.dispatch(&mut ctx, &SessionEvent::CheckTimeout);

    assert!(matches!(session, Session::LoggedOut));
}

#[test]
fn test_session_manual_logout_resets_timer() {
    let mut ctx = SessionContext {
        timer: MockTimer::new(),
        session_timeout_ms: 3000,
        last_activity: 0,
    };

    let mut session = Session::LoggedOut;
    session.init(&mut ctx);

    session.dispatch(&mut ctx, &SessionEvent::Login);
    ctx.timer.tick(1000);

    session.dispatch(&mut ctx, &SessionEvent::Logout);

    assert!(matches!(session, Session::LoggedOut));
    assert!(!ctx.timer.is_running);
}

// ============================================================================
// Test 4: Multiple Timers Pattern (if needed in future)
// ============================================================================

#[test]
fn test_timer_starts_on_entry() {
    let mut ctx = ConnectionContext {
        timer: MockTimer::new(),
        timeout_ms: 2000,
    };

    let mut conn = Connection::Idle;
    conn.init(&mut ctx);

    assert!(!ctx.timer.is_running);

    conn.dispatch(&mut ctx, &ConnEvent::Connect);

    // Entry hook should have started timer
    assert!(ctx.timer.is_running);
    assert_eq!(ctx.timer.remaining_ms, 2000);
}

#[test]
fn test_timer_reset_behavior() {
    let mut timer = MockTimer::new();

    timer.start(5000);
    assert!(timer.is_running);
    assert_eq!(timer.remaining_ms, 5000);

    timer.tick(2000);
    assert_eq!(timer.remaining_ms, 3000);
    assert!(!timer.is_expired());

    timer.reset();
    assert!(!timer.is_running);
    assert_eq!(timer.remaining_ms, 0);
}
