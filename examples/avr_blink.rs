#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)] // Required for AVR interrupt handlers

use typed_fsm::{state_machine, Transition};
use core::cell::Cell;
use avr_device::interrupt::{self, Mutex};
use avr_device::entry;

// Panic handler for no_std
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // In a real application, you'd want to do something more robust here,
    // like blinking an LED, sending debug info, or resetting.
    loop {}
}

// Context: Shared state across all states
struct LedContext {
    toggle_count: Cell<u32>,
}

// Event: Simple tick event
#[derive(Debug, Clone)]
enum Event {
    Tick,
}

// State machine with two states: On and Off
state_machine! {
    Name: BlinkFSM,
    Context: LedContext,
    Event: Event,
    QueueCapacity: 4, // Explicitly set capacity for heapless Deque

    States: {
        On => {
            entry: |ctx| {
                // In a real AVR app, you'd toggle a pin here.
                ctx.toggle_count.set(ctx.toggle_count.get() + 1);
            }

            process: |_ctx, event| {
                match event {
                    Event::Tick => Transition::To(BlinkFSM::Off),
                }
            }
        },

        Off => {
            entry: |ctx| {
                // In a real AVR app, you'd toggle a pin here.
                ctx.toggle_count.set(ctx.toggle_count.get() + 1);
            }

            process: |_ctx, event| {
                match event {
                    Event::Tick => Transition::To(BlinkFSM::On),
                }
            }
        }
    }
}

// Global static FSM and Context protected by Mutex for ISR access
static FSM: Mutex<Cell<Option<BlinkFSM>>> = Mutex::new(Cell::new(None));
static CTX: Mutex<Cell<Option<LedContext>>> = Mutex::new(Cell::new(None));

#[entry]
fn main() -> ! {
    // Initialize the FSM and Context
    interrupt::free(|cs| {
        FSM.borrow(cs).set(Some(BlinkFSM::On));
        CTX.borrow(cs).set(Some(LedContext { toggle_count: Cell::new(0) }));

        // CRITICAL: Initialize the FSM, which calls the entry action of the initial state.
        // This is done once after setup.
        if let (Some(fsm_instance), Some(ctx_instance)) = (
            FSM.borrow(cs).get_mut(),
            CTX.borrow(cs).get_mut(),
        ) {
            fsm_instance.init(ctx_instance);
        }
    });

    // Enable interrupts globally.
    // In a real AVR HAL, this might be handled by a specific function like avr_hal_generic::avr::interrupt::enable();
    unsafe {
        avr_device::interrupt::enable();
    }

    // Simulate an event loop (e.g., from a main loop or another timer ISR)
    loop {
        // Simulate a tick from the main loop
        interrupt::free(|cs| {
            if let (Some(fsm_instance), Some(ctx_instance)) = (
                FSM.borrow(cs).get_mut(),
                CTX.borrow(cs).get_mut(),
            ) {
                fsm_instance.dispatch(ctx_instance, &Event::Tick);
            }
        });

        // Small delay
        avr_device::delay_ms(500);

        // Simulate another tick (e.g., from an ISR)
        // This would typically be in an actual ISR
        // #[avr_device::interrupt(atmega328p)] // Assuming ATmega328P
        // fn TIMER1_COMPA() {
        interrupt::free(|cs| {
            if let (Some(fsm_instance), Some(ctx_instance)) = (
                FSM.borrow(cs).get_mut(),
                CTX.borrow(cs).get_mut(),
            ) {
                fsm_instance.dispatch(ctx_instance, &Event::Tick);
            }
        });
        // }

        avr_device::delay_ms(500);
    }
}
