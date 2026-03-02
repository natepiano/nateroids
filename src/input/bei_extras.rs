use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

/// Generates a BEI `InputAction` struct.
///
/// ```rust
/// action!(AabbsSwitch);
/// ```
///
/// Expands to:
/// ```rust
/// #[derive(InputAction)]
/// #[action_output(bool)]
/// pub struct AabbsSwitch;
/// ```
macro_rules! action {
    ($action:ident) => {
        #[derive(InputAction)]
        #[action_output(bool)]
        pub struct $action;
    };
}

/// Generates a Bevy `Event` struct for BRP-triggerable events.
///
/// ```rust
/// event!(PauseEvent);
/// ```
///
/// Expands to:
/// ```rust
/// #[derive(Event, Reflect, Default)]
/// #[reflect(Event)]
/// pub struct PauseEvent;
/// ```
macro_rules! event {
    ($event:ident) => {
        #[derive(Event, Reflect, Default)]
        #[reflect(Event)]
        pub struct $event;
    };
}

/// Wires an input action to a command function through an intermediate event.
///
/// Registers two observers:
/// 1. `On<Start<Action>>` → triggers `Event`
/// 2. `On<Event>` → runs `command` via `run_system_cached`
///
/// The intermediate event decouples the keyboard input from the command execution.
/// This means the same command can be invoked both by a user-initiated keybinding
/// and programmatically (e.g. via `commands.trigger(PauseEvent)` or the Bevy Remote
/// Protocol's `world.trigger_event`), with both paths calling the same
/// `run_system_cached` command.
///
/// Use with `action!` and `event!` to generate the action and event structs.
///
/// Requires `bevy::prelude::*` in scope at the call site.
///
/// ```rust
/// bind_action_system!(app, PauseToggle, PauseEvent, pause_command);
/// ```
macro_rules! bind_action_system {
    ($app:expr, $action:ty, $event:ty, $command:path) => {
        $app.add_observer(
            |_: On<bevy_enhanced_input::action::events::Start<$action>>, mut commands: Commands| {
                commands.trigger(<$event>::default());
            },
        )
        .add_observer(|_: On<$event>, mut commands: Commands| {
            commands.run_system_cached($command);
        })
    };
}

/// Extension trait for `ActionSpawner` providing shorthand methods for spawning
/// keybinding actions.
pub trait ActionSpawnerExt<C: Component> {
    /// Spawn an action bound to a single key.
    fn spawn_key<A: InputAction>(&mut self, settings: ActionSettings, key: KeyCode);

    /// Spawn an action bound to Shift + key.
    fn spawn_shift_key<A: InputAction>(&mut self, settings: ActionSettings, key: KeyCode);

    /// Spawn an action with arbitrary bindings.
    fn spawn_binding<A: InputAction, B: Bundle>(&mut self, settings: ActionSettings, bindings: B);
}

impl<C: Component> ActionSpawnerExt<C> for ActionSpawner<'_, C> {
    fn spawn_key<A: InputAction>(&mut self, settings: ActionSettings, key: KeyCode) {
        self.spawn_binding::<A, _>(settings, bindings![key]);
    }

    fn spawn_shift_key<A: InputAction>(&mut self, settings: ActionSettings, key: KeyCode) {
        self.spawn_binding::<A, _>(settings, bindings![key.with_mod_keys(ModKeys::SHIFT)]);
    }

    fn spawn_binding<A: InputAction, B: Bundle>(&mut self, settings: ActionSettings, bindings: B) {
        self.spawn((Action::<A>::new(), settings, bindings));
    }
}
