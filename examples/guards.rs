//! # Guards Example: Conditional State Transitions
//!
//! This example demonstrates **guards** - conditional logic that controls whether
//! state transitions are allowed to occur.
//!
//! ## What are Guards?
//!
//! Guards are boolean conditions evaluated during event processing that determine
//! if a state transition should happen. They act as gatekeepers, preventing invalid
//! or unauthorized state changes.
//!
//! ## Use Cases Demonstrated
//!
//! 1. **ATM PIN Verification** - Security guard checking PIN correctness
//! 2. **Resource Availability** - Checking if resources are available before transition
//! 3. **Business Rules** - Enforcing business logic constraints
//!
//! Run with: `cargo run --example guards`

use typed_fsm::{state_machine, Transition};

// ============================================================================
// Example 1: ATM with PIN Security Guard
// ============================================================================

struct ATMContext {
    correct_pin: u32,
    attempts: u32,
    account_balance: u32,
}

#[derive(Debug, Clone)]
enum ATMEvent {
    InsertCard,
    EnterPIN {
        pin: u32,
    },
    Withdraw {
        amount: u32,
    },
    #[allow(dead_code)]
    Cancel,
}

state_machine! {
    Name: ATM,
    Context: ATMContext,
    Event: ATMEvent,

    States: {
        Idle => {
            entry: |ctx| {
                ctx.attempts = 0;
                println!("\nATM: Ready. Please insert card.");
            }

            process: |_ctx, evt| {
                match evt {
                    ATMEvent::InsertCard => {
                        println!("ATM: Card inserted. Enter PIN.");
                        Transition::To(ATM::WaitingPIN)
                    }
                    _ => Transition::None
                }
            }
        },

        WaitingPIN => {
            process: |ctx, evt| {
                match evt {
                    ATMEvent::EnterPIN { pin } => {
                        // GUARD 1: Check if PIN is correct
                        if *pin == ctx.correct_pin {
                            println!("ATM: PIN accepted. Welcome!");
                            Transition::To(ATM::Authenticated)
                        } else {
                            ctx.attempts += 1;
                            println!("ATM: Incorrect PIN. Attempt {}/3", ctx.attempts);

                            // GUARD 2: Block after 3 failed attempts
                            if ctx.attempts >= 3 {
                                println!("ATM: Too many failed attempts. Card blocked!");
                                Transition::To(ATM::Blocked)
                            } else {
                                Transition::None
                            }
                        }
                    }
                    ATMEvent::Cancel => {
                        println!("ATM: Transaction cancelled.");
                        Transition::To(ATM::Idle)
                    }
                    _ => Transition::None
                }
            }
        },

        Authenticated => {
            entry: |ctx| {
                println!("ATM: Account balance: ${}", ctx.account_balance);
            }

            process: |ctx, evt| {
                match evt {
                    ATMEvent::Withdraw { amount } => {
                        // GUARD 3: Check if sufficient balance
                        if *amount <= ctx.account_balance {
                            ctx.account_balance -= amount;
                            println!("ATM: Dispensing ${}. New balance: ${}",
                                     amount, ctx.account_balance);
                            println!("ATM: Thank you!");
                            Transition::To(ATM::Idle)
                        } else {
                            println!("ATM: Insufficient funds. Balance: ${}",
                                     ctx.account_balance);
                            Transition::None
                        }
                    }
                    ATMEvent::Cancel => {
                        println!("ATM: Session ended.");
                        Transition::To(ATM::Idle)
                    }
                    _ => Transition::None
                }
            }
        },

        Blocked => {
            entry: |_ctx| {
                println!("ATM: This card is blocked. Contact your bank.");
            }

            process: |_ctx, _evt| {
                // Cannot process any events when blocked
                Transition::None
            }
        }
    }
}

// ============================================================================
// Example 2: Door Lock with Access Control
// ============================================================================

struct DoorContext {
    authorized_codes: Vec<u32>,
    is_locked: bool,
}

#[derive(Debug, Clone)]
enum DoorEvent {
    EnterCode { code: u32 },
    Lock,
    EmergencyOpen,
}

state_machine! {
    Name: DoorLock,
    Context: DoorContext,
    Event: DoorEvent,

    States: {
        Locked => {
            process: |ctx, evt| {
                match evt {
                    DoorEvent::EnterCode { code } => {
                        // GUARD: Check if code is in authorized list
                        if ctx.authorized_codes.contains(code) {
                            println!("Door: Access granted. Code: {}", code);
                            Transition::To(DoorLock::Unlocked)
                        } else {
                            println!("Door: Access denied. Invalid code: {}", code);
                            Transition::None
                        }
                    }
                    DoorEvent::EmergencyOpen => {
                        // Emergency always works (no guard)
                        println!("Door: Emergency open activated!");
                        Transition::To(DoorLock::Unlocked)
                    }
                    _ => Transition::None
                }
            }
        },

        Unlocked => {
            entry: |ctx| {
                ctx.is_locked = false;
                println!("Door: Now unlocked. Please enter.");
            }

            process: |ctx, evt| {
                match evt {
                    DoorEvent::Lock => {
                        ctx.is_locked = true;
                        println!("Door: Locked.");
                        Transition::To(DoorLock::Locked)
                    }
                    _ => Transition::None
                }
            }
        }
    }
}

// ============================================================================
// Example 3: Order Processing with Business Rules
// ============================================================================

struct OrderContext {
    order_value: f32,
    customer_credit: f32,
    items_in_stock: bool,
}

#[derive(Debug, Clone)]
enum OrderEvent {
    Submit,
    PaymentReceived,
    Ship,
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
                        // GUARD: Multiple conditions (business rules)
                        // 1. Check stock availability
                        if !ctx.items_in_stock {
                            println!("Order: Cannot submit. Items out of stock.");
                            return Transition::None;
                        }

                        // 2. Check credit limit
                        if ctx.order_value > ctx.customer_credit {
                            println!("Order: Cannot submit. Exceeds credit limit.");
                            println!("       Order: ${:.2}, Credit: ${:.2}",
                                     ctx.order_value, ctx.customer_credit);
                            return Transition::None;
                        }

                        println!("Order: Submitted successfully. Awaiting payment.");
                        Transition::To(Order::PendingPayment)
                    }
                    _ => Transition::None
                }
            }
        },

        PendingPayment => {
            process: |ctx, evt| {
                match evt {
                    OrderEvent::PaymentReceived => {
                        println!("Order: Payment received. Processing...");
                        ctx.customer_credit -= ctx.order_value;
                        Transition::To(Order::Paid)
                    }
                    OrderEvent::Cancel => {
                        println!("Order: Cancelled before payment.");
                        Transition::To(Order::Cancelled)
                    }
                    _ => Transition::None
                }
            }
        },

        Paid => {
            process: |_ctx, evt| {
                match evt {
                    OrderEvent::Ship => {
                        println!("Order: Shipped!");
                        Transition::To(Order::Completed)
                    }
                    _ => Transition::None
                }
            }
        },

        Completed => {
            entry: |_ctx| {
                println!("Order: Delivered. Thank you!");
            }

            process: |_ctx, _evt| {
                Transition::None
            }
        },

        Cancelled => {
            entry: |_ctx| {
                println!("Order: Order has been cancelled.");
            }

            process: |_ctx, _evt| {
                Transition::None
            }
        }
    }
}

// ============================================================================
// Main: Run all examples
// ============================================================================

fn main() {
    println!("=== Guards Example: Conditional State Transitions ===\n");

    // Example 1: ATM with PIN verification
    println!("--- Example 1: ATM with Security Guards ---");
    run_atm_example();

    println!("\n");

    // Example 2: Door lock with access control
    println!("--- Example 2: Door Lock with Access Control ---");
    run_door_example();

    println!("\n");

    // Example 3: Order with business rules
    println!("--- Example 3: Order Processing with Business Rules ---");
    run_order_example();

    println!("\n=== Key Takeaways ===");
    println!("1. Guards are implemented using if/else logic in process blocks");
    println!("2. Guards can check multiple conditions (AND/OR logic)");
    println!("3. Guards prevent invalid state transitions at runtime");
    println!("4. Guards can enforce security, resource checks, and business rules");
    println!("5. No special syntax needed - uses standard Rust conditionals");
}

fn run_atm_example() {
    let mut ctx = ATMContext {
        correct_pin: 1234,
        attempts: 0,
        account_balance: 1000,
    };

    let mut atm = ATM::Idle;
    atm.init(&mut ctx);

    // Scenario 1: Wrong PIN attempts
    atm.dispatch(&mut ctx, &ATMEvent::InsertCard);
    atm.dispatch(&mut ctx, &ATMEvent::EnterPIN { pin: 0000 });
    atm.dispatch(&mut ctx, &ATMEvent::EnterPIN { pin: 1111 });
    atm.dispatch(&mut ctx, &ATMEvent::EnterPIN { pin: 2222 });
    // Now blocked!

    println!("\n  (Starting fresh ATM for successful transaction)");

    // Scenario 2: Correct PIN and withdrawal
    let mut ctx2 = ATMContext {
        correct_pin: 1234,
        attempts: 0,
        account_balance: 1000,
    };
    let mut atm2 = ATM::Idle;
    atm2.init(&mut ctx2);

    atm2.dispatch(&mut ctx2, &ATMEvent::InsertCard);
    atm2.dispatch(&mut ctx2, &ATMEvent::EnterPIN { pin: 1234 });
    atm2.dispatch(&mut ctx2, &ATMEvent::Withdraw { amount: 200 });
}

fn run_door_example() {
    let mut ctx = DoorContext {
        authorized_codes: vec![1234, 5678, 9999],
        is_locked: true,
    };

    let mut door = DoorLock::Locked;
    door.init(&mut ctx);

    // Try wrong code
    door.dispatch(&mut ctx, &DoorEvent::EnterCode { code: 0000 });

    // Try correct code
    door.dispatch(&mut ctx, &DoorEvent::EnterCode { code: 5678 });

    // Lock again
    door.dispatch(&mut ctx, &DoorEvent::Lock);

    // Emergency open (no guard needed)
    door.dispatch(&mut ctx, &DoorEvent::EmergencyOpen);
}

fn run_order_example() {
    let mut ctx = OrderContext {
        order_value: 500.0,
        customer_credit: 1000.0,
        items_in_stock: true,
    };

    let mut order = Order::Draft;
    order.init(&mut ctx);

    // Successful submission
    order.dispatch(&mut ctx, &OrderEvent::Submit);
    order.dispatch(&mut ctx, &OrderEvent::PaymentReceived);
    order.dispatch(&mut ctx, &OrderEvent::Ship);

    println!("\n  (Testing guard failures)");

    // Test: Insufficient credit
    let mut ctx2 = OrderContext {
        order_value: 1500.0,
        customer_credit: 1000.0,
        items_in_stock: true,
    };
    let mut order2 = Order::Draft;
    order2.init(&mut ctx2);
    order2.dispatch(&mut ctx2, &OrderEvent::Submit);

    // Test: Out of stock
    let mut ctx3 = OrderContext {
        order_value: 500.0,
        customer_credit: 1000.0,
        items_in_stock: false,
    };
    let mut order3 = Order::Draft;
    order3.init(&mut ctx3);
    order3.dispatch(&mut ctx3, &OrderEvent::Submit);
}
