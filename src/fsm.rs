//! # Finite State Machine Microframework
//!
//! A lightweight, zero-cost, declarative FSM generator using Rust macros.
//! Designed for embedded systems (no-std compatible) and high-performance applications.
//!
//! ## Design Philosophy
//!
//! - **Zero Allocations:** Uses `enums` and static dispatch. No `Box`, `dyn`, or heap allocations.
//! - **Memory Safety:** Leverages Rust's type system to prevent invalid state transitions.
//! - **Hygiene:** Uses closure-like syntax to strictly define variable scopes.
//! - **Performance:** Compiles to efficient machine code with minimal overhead.
//!
//! ## How It Works
//!
//! The `state_machine!` macro generates:
//! 1. A `pub enum` where each variant represents a state (can carry data)
//! 2. Methods for state lifecycle: `init()`, `dispatch()`, `on_entry()`, `on_exit()`, `on_process()`
//! 3. Type-safe transition logic with compile-time validation
//!
//! The generated state machine follows this lifecycle for each event:
//! ```text
//! Event → Process → [Transition?] → Exit (old) → Entry (new) → Update State
//! ```

// Logging support (optional) - Internal macro for code generation
#[macro_export]
#[doc(hidden)]
macro_rules! __fsm_log {
    ($($arg:tt)*) => {
        #[cfg(feature = "logging")]
        {
            log::info!($($arg)*);
        }
        // When logging feature disabled, generate no code at all (true zero-cost)
    };
}

/// Represents the result of a state processing step.
///
/// This enum guides the state machine on whether to stay or switch states.
/// **Every `process` closure must return a `Transition`.**
///
/// # Type Parameters
///
/// * `S` - The state machine enum type
///
/// # When to Use Each Variant
///
/// ## `Transition::None`
/// Use when an event should be processed but doesn't require changing states:
/// - Event updates context but state logic remains the same
/// - Event should be ignored in the current state
/// - Handling events that don't affect state flow
///
/// ## `Transition::To(State)`
/// Use when an event should trigger a state change:
/// - Event triggers a state transition
/// - Conditions are met for moving to another state
/// - Need to execute `exit` and `entry` hooks
/// - Even for self-transitions (same state to same state)
///
/// # Examples
///
/// ```rust
/// use typed_fsm::Transition;
///
/// #[derive(Debug)]
/// enum MyState {
///     Idle,
///     Active { speed: u32 },
/// }
///
/// // Stay in current state - no hooks execute
/// let no_change: Transition<MyState> = Transition::None;
///
/// // Transition to a new state - exit + entry execute
/// let change = Transition::To(MyState::Idle);
///
/// // Transition with state data
/// let with_data = Transition::To(MyState::Active { speed: 100 });
/// ```
///
/// # Common Pattern in `process` Hook
///
/// ```rust
/// # use typed_fsm::{state_machine, Transition};
/// # struct Context { data: u32 }
/// # #[derive(Debug)]
/// # enum Event { Update(u32), Activate, Ignore }
/// # state_machine! {
/// #     Name: FSM,
/// #     Context: Context,
/// #     Event: Event,
/// #     States: {
/// #         Idle => {
/// process: |ctx, evt| {
///     match evt {
///         Event::Update(value) => {
///             ctx.data = *value;        // Update context
///             Transition::None          // Stay in same state
///         },
///         Event::Activate => {
///             Transition::To(FSM::Active { speed: 100 })  // Change state
///         },
///         Event::Ignore => Transition::None  // Do nothing
///     }
/// }
/// #         },
/// #         Active { speed: u32 } => {
/// #             process: |ctx, evt| { Transition::None }
/// #         }
/// #     }
/// # }
/// ```
///
/// # Performance
///
/// Creating a `Transition` has zero runtime overhead. The enum is optimized
/// by the compiler and typically doesn't allocate any heap memory.
///
/// # Thread Safety
///
/// `Transition` is `Send` and `Sync` if the state type `S` is `Send` and `Sync`.
pub enum Transition<S> {
    /// Stay in the current state (no action required).
    ///
    /// Use this when an event should be handled but doesn't trigger a state change.
    ///
    /// # When to Use
    ///
    /// - Event updates context but state remains the same
    /// - Event should be ignored in this state
    /// - Processing an event that doesn't affect state flow
    ///
    /// # Lifecycle Impact
    ///
    /// - `process` executes
    /// - `exit` does NOT execute (no state change)
    /// - `entry` does NOT execute (no state change)
    /// - State remains unchanged
    ///
    /// # Example
    ///
    /// ```rust
    /// # use typed_fsm::{state_machine, Transition};
    /// # struct Context { counter: u32 }
    /// # #[derive(Debug)]
    /// # enum Event { Increment }
    /// # state_machine! {
    /// #     Name: FSM,
    /// #     Context: Context,
    /// #     Event: Event,
    /// #     States: {
    /// #         Active => {
    /// process: |ctx, evt| {
    ///     match evt {
    ///         Event::Increment => {
    ///             ctx.counter += 1;   // Update context
    ///             Transition::None    // Stay in Active state
    ///         }
    ///     }
    /// }
    /// #         }
    /// #     }
    /// # }
    /// ```
    None,

    /// Transition to a new state.
    ///
    /// This will trigger the full state transition lifecycle:
    /// `exit` (old state) → `entry` (new state) → update state
    ///
    /// # When to Use
    ///
    /// - Event triggers a state change
    /// - Conditions are met for transitioning
    /// - Need to move to a different state
    /// - Self-transitions (same state, but re-execute entry/exit)
    ///
    /// # Arguments
    ///
    /// * `0` - The new state instance. Can carry data (payloads).
    ///
    /// # Lifecycle Impact
    ///
    /// 1. `process` executes and returns new state
    /// 2. Current state's `exit` executes (if defined)
    /// 3. New state's `entry` executes (if defined)
    /// 4. State updates to the new state
    ///
    /// # Example
    ///
    /// ```rust
    /// # use typed_fsm::{state_machine, Transition};
    /// # struct Context { }
    /// # #[derive(Debug)]
    /// # enum Event { Start, Stop }
    /// # state_machine! {
    /// #     Name: FSM,
    /// #     Context: Context,
    /// #     Event: Event,
    /// #     States: {
    /// #         Idle => {
    /// process: |ctx, evt| {
    ///     match evt {
    ///         Event::Start => {
    ///             // Transition to Running state with data
    ///             Transition::To(FSM::Running { speed: 100 })
    ///         },
    ///         Event::Stop => Transition::None
    ///     }
    /// }
    /// #         },
    /// #         Running { speed: u32 } => {
    /// #             process: |ctx, evt| { Transition::None }
    /// #         }
    /// #     }
    /// # }
    /// ```
    ///
    /// # Performance
    ///
    /// State transitions use move semantics, making them extremely fast
    /// (typically just a few CPU instructions).
    To(S),
}

/// Generates the State Machine Enum and its implementation.
///
/// This macro creates a `pub enum` with the specified name and implements
/// the necessary logic for state transitions, entry/exit actions, and event processing.
///
/// # Macro Parameters
///
/// - **Name**: The identifier for the generated state machine enum
/// - **Context**: The type of shared state accessible to all states
/// - **Event**: The type of events that drive the state machine
/// - **States**: Block defining all possible states and their behavior
///
/// # State Definition
///
/// Each state can have:
/// - **entry** (optional): Closure executed once when entering the state
/// - **process** (required): Closure that handles events and returns `Transition<S>`
/// - **exit** (optional): Closure executed once when leaving the state
///
/// States can carry data by adding fields: `StateName { field: Type }`
///
/// # Complete Example
///
/// ```rust
/// use typed_fsm::{state_machine, Transition};
///
/// struct MyContext {
///     counter: u32,
/// }
///
/// #[derive(Debug)]
/// enum MyEvent {
///     Start,
///     Stop,
/// }
///
/// state_machine! {
///     Name: MyMachine,
///     Context: MyContext,
///     Event: MyEvent,
///     States: {
///         Idle => {
///             entry: |ctx| {
///                 println!("Entering Idle");
///                 ctx.counter = 0;
///             }
///
///             process: |_ctx, evt| {
///                 match evt {
///                     MyEvent::Start => Transition::To(MyMachine::Active { id: 1 }),
///                     _ => Transition::None
///                 }
///             }
///         },
///
///         Active { id: u32 } => {
///             entry: |ctx| {
///                 println!("Entering Active with id: {}", id);
///                 ctx.counter += 1;
///             }
///
///             process: |_ctx, evt| {
///                 match evt {
///                     MyEvent::Stop => Transition::To(MyMachine::Idle),
///                     _ => Transition::None
///                 }
///             }
///
///             exit: |_ctx| {
///                 println!("Leaving Active");
///             }
///         }
///     }
/// }
/// ```
///
/// # Usage
///
/// ```rust
/// # use typed_fsm::{state_machine, Transition};
/// # struct MyContext { counter: u32 }
/// # #[derive(Debug)]
/// # enum MyEvent { Start, Stop }
/// # state_machine! {
/// #     Name: MyMachine,
/// #     Context: MyContext,
/// #     Event: MyEvent,
/// #     States: {
/// #         Idle => {
/// #             process: |_ctx, evt| {
/// #                 match evt {
/// #                     MyEvent::Start => Transition::To(MyMachine::Active { id: 1 }),
/// #                     _ => Transition::None
/// #                 }
/// #             }
/// #         },
/// #         Active { id: u32 } => {
/// #             process: |_ctx, evt| {
/// #                 match evt {
/// #                     MyEvent::Stop => Transition::To(MyMachine::Idle),
/// #                     _ => Transition::None
/// #                 }
/// #             }
/// #         }
/// #     }
/// # }
/// let mut ctx = MyContext { counter: 0 };
/// let mut fsm = MyMachine::Idle;
///
/// // Initialize (calls entry action of initial state)
/// fsm.init(&mut ctx);
///
/// // Dispatch events
/// fsm.dispatch(&mut ctx, &MyEvent::Start);
/// fsm.dispatch(&mut ctx, &MyEvent::Stop);
/// ```
#[macro_export]
macro_rules! state_machine {
    (
        Name: $enum_name:ident,
        Context: $ctx_type:ty,
        Event: $event_type:ty,
        States: {
            $(
                // Captures the State Name and optional fields (e.g., Running { speed: u32 })
                $state_name:ident $( { $($field_name:ident : $field_type:ty),* } )? => {

                    // Optional Entry Block: entry: |ctx| { ... }
                    $( entry: |$entry_ctx:ident| $entry_block:block )?

                    // Mandatory Process Block: process: |ctx, evt| { ... }
                    process: |$ctx_var:ident, $evt_var:ident| $process_block:block

                    // Optional Exit Block: exit: |ctx| { ... }
                    $( exit: |$exit_ctx:ident| $exit_block:block )?
                }
            ),* $(,)?
        }
    ) => {
        /// Auto-generated State Machine Enum.
        /// Holds the current state and its internal data.
        #[derive(Debug)]
        pub enum $enum_name {
            $(
                $state_name $( { $($field_name : $field_type),* } )?,
            )*
        }

        impl $enum_name {
            /// Initializes the state machine by executing the entry action of the initial state.
            ///
            /// # CRITICAL: Must be called before the event loop!
            ///
            /// **Forgetting to call `init()` will cause silent failures:**
            /// - The `entry` action of the initial state will NEVER execute
            /// - State machine will still process events, but initialization is skipped
            /// - This can lead to incorrect behavior that is difficult to debug
            ///
            /// # Correct Usage
            ///
            /// ```rust
            /// # use typed_fsm::{state_machine, Transition};
            /// # struct Context { count: u32 }
            /// # #[derive(Debug)]
            /// # enum Event { Tick }
            /// # state_machine! {
            /// #     Name: FSM,
            /// #     Context: Context,
            /// #     Event: Event,
            /// #     States: {
            /// #         Idle => {
            /// #             entry: |ctx| { ctx.count = 0; }
            /// #             process: |_ctx, _evt| { Transition::None }
            /// #         }
            /// #     }
            /// # }
            /// let mut ctx = Context { count: 0 };
            /// let mut fsm = FSM::Idle;
            ///
            /// // CORRECT: Call init() before event loop
            /// fsm.init(&mut ctx);
            ///
            /// // Now safe to dispatch events
            /// fsm.dispatch(&mut ctx, &Event::Tick);
            /// ```
            ///
            /// # Incorrect Usage (Common Mistake)
            ///
            /// ```rust,no_run
            /// # use typed_fsm::{state_machine, Transition};
            /// # struct Context { count: u32 }
            /// # #[derive(Debug)]
            /// # enum Event { Tick }
            /// # state_machine! {
            /// #     Name: FSM,
            /// #     Context: Context,
            /// #     Event: Event,
            /// #     States: {
            /// #         Idle => {
            /// #             entry: |ctx| { ctx.count = 0; }
            /// #             process: |_ctx, _evt| { Transition::None }
            /// #         }
            /// #     }
            /// # }
            /// let mut ctx = Context { count: 0 };
            /// let mut fsm = FSM::Idle;
            ///
            /// // WRONG: Forgot to call init()!
            /// // The entry action will NEVER execute!
            /// fsm.dispatch(&mut ctx, &Event::Tick);
            /// ```
            ///
            /// # When to Call
            ///
            /// - Call exactly **once** after creating the state machine
            /// - Call **before** entering the event loop
            /// - Call **before** the first `dispatch()`
            #[allow(unused_variables)]
            pub fn init(&mut self, ctx: &mut $ctx_type) {
                $crate::__fsm_log!("[{}] init() -> {:?}", stringify!($enum_name), self);
                self.on_entry(ctx);
            }

            /// Internal: Executes the entry action for the current state.
            #[allow(unused_variables)]
            fn on_entry(&mut self, arg_ctx: &mut $ctx_type) {
                $crate::__fsm_log!("[{}] {:?}.entry()", stringify!($enum_name), self);
                match self {
                    $(
                        // Matches the current state and captures its fields (if any)
                        Self::$state_name $( { $($field_name),* } )? => {
                            // Only expands if the user defined an entry block
                            $(
                                // Rename the context variable to what the user chose (e.g., |ctx|)
                                #[allow(unused_variables)]
                                let $entry_ctx = arg_ctx;

                                // Execute user code
                                $entry_block
                            )?
                        }
                    )*
                }
            }

            /// Internal: Executes the exit action for the current state.
            #[allow(unused_variables)]
            fn on_exit(&mut self, arg_ctx: &mut $ctx_type) {
                $crate::__fsm_log!("[{}] {:?}.exit()", stringify!($enum_name), self);
                match self {
                    $(
                        Self::$state_name $( { $($field_name),* } )? => {
                            $(
                                #[allow(unused_variables)]
                                let $exit_ctx = arg_ctx;
                                $exit_block
                            )?
                        }
                    )*
                }
            }

            /// Internal: Determines the next state based on the event.
            /// Returns a `Transition` enum.
            fn on_process(&mut self, arg_ctx: &mut $ctx_type, arg_evt: &$event_type) -> Transition<Self> {
                match self {
                    $(
                        // We allow unused variables here because the state might have data
                        // (like 'speed') that the user logic doesn't need to access in this specific event.
                        #[allow(unused_variables)]
                        Self::$state_name $( { $($field_name),* } )? => {

                            // Bind context and event to user-defined names (e.g., |ctx, evt|)
                            #[allow(unused_variables)]
                            let $ctx_var = arg_ctx;

                            #[allow(unused_variables)]
                            let $evt_var = arg_evt;

                            // Execute user's process logic
                            $process_block
                        }
                    )*
                }
            }

            /// Main Event Dispatcher.
            ///
            /// This is the primary function to call in your main loop.
            /// It handles the full lifecycle: `Process` -> `Exit Old` -> `Update` -> `Entry New`.
            ///
            /// # Performance
            /// Marked `#[inline(always)]` to allow the compiler to flatten the state machine
            /// into a highly optimized jump table / switch-case structure.
            #[inline(always)]
            pub fn dispatch(&mut self, ctx: &mut $ctx_type, event: &$event_type) {
                // 1. Calculate Transition
                let transition = self.on_process(ctx, event);

                // 2. Apply Transition (if any)
                match transition {
                    Transition::To(mut new_state) => {
                        $crate::__fsm_log!("[{}] {:?} + {:?} -> {:?}",
                                           stringify!($enum_name), self, event, new_state);

                        // A. Exit current state
                        self.on_exit(ctx);

                        // B. Enter new state
                        new_state.on_entry(ctx);

                        // C. Update state (Move semantics - extremely fast)
                        *self = new_state;
                    }
                    Transition::None => {
                        $crate::__fsm_log!("[{}] {:?} + {:?} -> None (stayed)",
                                           stringify!($enum_name), self, event);
                    }
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transition_none_is_none() {
        // Verify that Transition::None can be created and pattern matched
        let trans: Transition<i32> = Transition::None;
        match trans {
            Transition::None => {} // Test passes if we reach this branch
            Transition::To(_) => panic!("Expected None"),
        }
    }

    #[test]
    fn test_transition_to_carries_value() {
        // Verify that Transition::To carries the correct value
        let trans = Transition::To(42);
        match trans {
            Transition::To(value) => assert_eq!(value, 42),
            Transition::None => panic!("Expected To"),
        }
    }

    #[test]
    fn test_transition_with_enum() {
        #[derive(Debug, PartialEq)]
        enum State {
            A,
            B { value: u32 },
        }

        // Test with simple variant
        let trans = Transition::To(State::A);
        match trans {
            Transition::To(State::A) => {} // Test passes if we reach this branch
            _ => panic!("Expected State::A"),
        }

        // Test with variant carrying data
        let trans = Transition::To(State::B { value: 100 });
        match trans {
            Transition::To(State::B { value }) => assert_eq!(value, 100),
            _ => panic!("Expected State::B"),
        }
    }
}
