//! Tests for Guards (Conditional Transitions) pattern
//!
//! This test suite validates that guard conditions work correctly:
//! - Guards block invalid transitions
//! - Guards allow valid transitions
//! - Multiple guard conditions (AND/OR logic)
//! - Context mutations in guard checks

use typed_fsm::{state_machine, Transition};

// ============================================================================
// Test 1: Simple Guard - PIN Verification
// ============================================================================

struct PINContext {
    correct_pin: u32,
    attempts: u32,
}

#[derive(Debug, Clone, PartialEq)]
enum PINEvent {
    EnterPIN { pin: u32 },
    #[allow(dead_code)]
    Reset,
}

state_machine! {
    Name: PINMachine,
    Context: PINContext,
    Event: PINEvent,

    States: {
        Locked => {
            process: |ctx, evt| {
                match evt {
                    PINEvent::EnterPIN { pin } => {
                        ctx.attempts += 1;

                        // Guard: Check PIN correctness
                        if *pin == ctx.correct_pin {
                            Transition::To(PINMachine::Unlocked)
                        } else {
                            // Guard: Block after 3 attempts
                            if ctx.attempts >= 3 {
                                Transition::To(PINMachine::Blocked)
                            } else {
                                Transition::None
                            }
                        }
                    }
                    _ => Transition::None
                }
            }
        },

        Unlocked => {
            entry: |ctx| {
                ctx.attempts = 0;
            }

            process: |_ctx, _evt| {
                Transition::None
            }
        },

        Blocked => {
            process: |ctx, evt| {
                match evt {
                    PINEvent::Reset => {
                        ctx.attempts = 0;
                        Transition::To(PINMachine::Locked)
                    }
                    _ => Transition::None
                }
            }
        }
    }
}

#[test]
fn test_guard_allows_correct_pin() {
    let mut ctx = PINContext {
        correct_pin: 1234,
        attempts: 0,
    };

    let mut machine = PINMachine::Locked;
    machine.init(&mut ctx);

    machine.dispatch(&mut ctx, &PINEvent::EnterPIN { pin: 1234 });

    assert!(matches!(machine, PINMachine::Unlocked));
    assert_eq!(ctx.attempts, 0); // Reset by entry hook
}

#[test]
fn test_guard_blocks_incorrect_pin() {
    let mut ctx = PINContext {
        correct_pin: 1234,
        attempts: 0,
    };

    let mut machine = PINMachine::Locked;
    machine.init(&mut ctx);

    machine.dispatch(&mut ctx, &PINEvent::EnterPIN { pin: 0000 });

    assert!(matches!(machine, PINMachine::Locked));
    assert_eq!(ctx.attempts, 1);
}

#[test]
fn test_guard_blocks_after_max_attempts() {
    let mut ctx = PINContext {
        correct_pin: 1234,
        attempts: 0,
    };

    let mut machine = PINMachine::Locked;
    machine.init(&mut ctx);

    // 3 wrong attempts
    machine.dispatch(&mut ctx, &PINEvent::EnterPIN { pin: 0000 });
    machine.dispatch(&mut ctx, &PINEvent::EnterPIN { pin: 1111 });
    machine.dispatch(&mut ctx, &PINEvent::EnterPIN { pin: 2222 });

    assert!(matches!(machine, PINMachine::Blocked));
    assert_eq!(ctx.attempts, 3);
}

#[test]
fn test_guard_resets_attempts_on_unlock() {
    let mut ctx = PINContext {
        correct_pin: 1234,
        attempts: 2,
    };

    let mut machine = PINMachine::Locked;
    machine.init(&mut ctx);

    machine.dispatch(&mut ctx, &PINEvent::EnterPIN { pin: 1234 });

    assert!(matches!(machine, PINMachine::Unlocked));
    assert_eq!(ctx.attempts, 0); // Reset in entry hook
}

// ============================================================================
// Test 2: Multiple Guard Conditions (AND logic)
// ============================================================================

struct OrderContext {
    balance: f32,
    stock_available: bool,
    order_value: f32,
}

#[derive(Debug, Clone)]
enum OrderEvent {
    Submit,
    #[allow(dead_code)]
    Cancel,
}

state_machine! {
    Name: Order,
    Context: OrderContext,
    Event: OrderEvent,

    States: {
        Draft => {
            process: |ctx, evt| {
                match evt {
                    OrderEvent::Submit => {
                        // Guard 1: Stock available
                        if !ctx.stock_available {
                            return Transition::None;
                        }

                        // Guard 2: Sufficient balance
                        if ctx.order_value > ctx.balance {
                            return Transition::None;
                        }

                        // Both guards passed
                        Transition::To(Order::Submitted)
                    }
                    _ => Transition::None
                }
            }
        },

        Submitted => {
            entry: |ctx| {
                ctx.balance -= ctx.order_value;
            }

            process: |_ctx, _evt| {
                Transition::None
            }
        }
    }
}

#[test]
fn test_multiple_guards_all_pass() {
    let mut ctx = OrderContext {
        balance: 100.0,
        stock_available: true,
        order_value: 50.0,
    };

    let mut order = Order::Draft;
    order.init(&mut ctx);

    order.dispatch(&mut ctx, &OrderEvent::Submit);

    assert!(matches!(order, Order::Submitted));
    assert_eq!(ctx.balance, 50.0);
}

#[test]
fn test_multiple_guards_stock_fails() {
    let mut ctx = OrderContext {
        balance: 100.0,
        stock_available: false,
        order_value: 50.0,
    };

    let mut order = Order::Draft;
    order.init(&mut ctx);

    order.dispatch(&mut ctx, &OrderEvent::Submit);

    assert!(matches!(order, Order::Draft));
    assert_eq!(ctx.balance, 100.0); // Unchanged
}

#[test]
fn test_multiple_guards_balance_fails() {
    let mut ctx = OrderContext {
        balance: 30.0,
        stock_available: true,
        order_value: 50.0,
    };

    let mut order = Order::Draft;
    order.init(&mut ctx);

    order.dispatch(&mut ctx, &OrderEvent::Submit);

    assert!(matches!(order, Order::Draft));
    assert_eq!(ctx.balance, 30.0); // Unchanged
}

#[test]
fn test_multiple_guards_both_fail() {
    let mut ctx = OrderContext {
        balance: 30.0,
        stock_available: false,
        order_value: 50.0,
    };

    let mut order = Order::Draft;
    order.init(&mut ctx);

    order.dispatch(&mut ctx, &OrderEvent::Submit);

    assert!(matches!(order, Order::Draft));
}

// ============================================================================
// Test 3: Guard with Range Check
// ============================================================================

struct TempContext {
    temperature: i32,
    min_safe: i32,
    max_safe: i32,
}

#[derive(Debug, Clone)]
enum TempEvent {
    UpdateTemp { temp: i32 },
}

state_machine! {
    Name: TempMonitor,
    Context: TempContext,
    Event: TempEvent,

    States: {
        Normal => {
            process: |ctx, evt| {
                match evt {
                    TempEvent::UpdateTemp { temp } => {
                        ctx.temperature = *temp;

                        // Guard: Check if temperature is in safe range
                        if *temp < ctx.min_safe || *temp > ctx.max_safe {
                            Transition::To(TempMonitor::Alert)
                        } else {
                            Transition::None
                        }
                    }
                }
            }
        },

        Alert => {
            process: |ctx, evt| {
                match evt {
                    TempEvent::UpdateTemp { temp } => {
                        ctx.temperature = *temp;

                        // Guard: Return to normal if temp is safe
                        if *temp >= ctx.min_safe && *temp <= ctx.max_safe {
                            Transition::To(TempMonitor::Normal)
                        } else {
                            Transition::None
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test_guard_range_check_stays_normal() {
    let mut ctx = TempContext {
        temperature: 25,
        min_safe: 10,
        max_safe: 30,
    };

    let mut monitor = TempMonitor::Normal;
    monitor.init(&mut ctx);

    monitor.dispatch(&mut ctx, &TempEvent::UpdateTemp { temp: 20 });

    assert!(matches!(monitor, TempMonitor::Normal));
    assert_eq!(ctx.temperature, 20);
}

#[test]
fn test_guard_range_check_too_high() {
    let mut ctx = TempContext {
        temperature: 25,
        min_safe: 10,
        max_safe: 30,
    };

    let mut monitor = TempMonitor::Normal;
    monitor.init(&mut ctx);

    monitor.dispatch(&mut ctx, &TempEvent::UpdateTemp { temp: 35 });

    assert!(matches!(monitor, TempMonitor::Alert));
    assert_eq!(ctx.temperature, 35);
}

#[test]
fn test_guard_range_check_too_low() {
    let mut ctx = TempContext {
        temperature: 25,
        min_safe: 10,
        max_safe: 30,
    };

    let mut monitor = TempMonitor::Normal;
    monitor.init(&mut ctx);

    monitor.dispatch(&mut ctx, &TempEvent::UpdateTemp { temp: 5 });

    assert!(matches!(monitor, TempMonitor::Alert));
    assert_eq!(ctx.temperature, 5);
}

#[test]
fn test_guard_range_check_returns_to_normal() {
    let mut ctx = TempContext {
        temperature: 35,
        min_safe: 10,
        max_safe: 30,
    };

    let mut monitor = TempMonitor::Alert;
    monitor.init(&mut ctx);

    monitor.dispatch(&mut ctx, &TempEvent::UpdateTemp { temp: 25 });

    assert!(matches!(monitor, TempMonitor::Normal));
    assert_eq!(ctx.temperature, 25);
}

// ============================================================================
// Test 4: Guard with List Membership Check
// ============================================================================

struct AccessContext {
    authorized_users: Vec<String>,
}

#[derive(Debug, Clone)]
enum AccessEvent {
    Login { username: String },
}

state_machine! {
    Name: AccessControl,
    Context: AccessContext,
    Event: AccessEvent,

    States: {
        LoggedOut => {
            process: |ctx, evt| {
                match evt {
                    AccessEvent::Login { username } => {
                        // Guard: Check if user is in authorized list
                        if ctx.authorized_users.contains(username) {
                            Transition::To(AccessControl::LoggedIn)
                        } else {
                            Transition::To(AccessControl::Denied)
                        }
                    }
                }
            }
        },

        LoggedIn => {
            process: |_ctx, _evt| {
                Transition::None
            }
        },

        Denied => {
            process: |_ctx, _evt| {
                Transition::None
            }
        }
    }
}

#[test]
fn test_guard_list_authorized_user() {
    let mut ctx = AccessContext {
        authorized_users: vec!["alice".to_string(), "bob".to_string()],
    };

    let mut access = AccessControl::LoggedOut;
    access.init(&mut ctx);

    access.dispatch(&mut ctx, &AccessEvent::Login {
        username: "alice".to_string(),
    });

    assert!(matches!(access, AccessControl::LoggedIn));
}

#[test]
fn test_guard_list_unauthorized_user() {
    let mut ctx = AccessContext {
        authorized_users: vec!["alice".to_string(), "bob".to_string()],
    };

    let mut access = AccessControl::LoggedOut;
    access.init(&mut ctx);

    access.dispatch(&mut ctx, &AccessEvent::Login {
        username: "eve".to_string(),
    });

    assert!(matches!(access, AccessControl::Denied));
}
