# Input/Event Command Pattern

This project uses a three-layer command flow for input-driven behavior:

1. Input observer (`Start<...>` from `bevy_enhanced_input`)
2. App command event (`...Event`)
3. Cached command system (`run_system_cached(...)`)

## Why this pattern

- Keeps input handling thin and keybinding-focused.
- Lets BRP trigger the same behavior with `world.trigger_event` (no key simulation).
- Centralizes behavior in one command system.
- Improves testability:
  - integration tests can trigger app events
  - focused tests can invoke command systems directly

## Flow

Keyboard path:

1. Key/chord fires `Start<Action>`
2. `on_*_input` observer triggers app event
3. `on_*_event` observer runs `run_system_cached(command_system)`
4. `command_system` performs the real logic

BRP path:

1. `world.trigger_event("...::SomeEvent")`
2. `on_*_event` observer runs `run_system_cached(command_system)`
3. Same command logic as keyboard path

## Current examples

- State commands:
  - `PauseEvent`
  - `RestartGameEvent`
  - `RestartWithSplashEvent`
  - See `src/state.rs`

- Camera commands:
  - `ZoomToFitEvent`
  - `CameraHomeEvent`
  - `ToggleFitTargetDebugEvent`
  - See `src/camera/zoom.rs`

## Naming convention

- Input observer: `on_<feature>_input`
- App event: `<Feature>Event`
- Event observer: `on_<feature>_event`
- Command system: `<feature>_command`

Example shape:

```rust
fn on_restart_game_input(
    _trigger: On<input_events::Start<RestartGameShortcut>>,
    mut commands: Commands,
) {
    commands.trigger(RestartGameEvent);
}

fn on_restart_game_event(_trigger: On<RestartGameEvent>, mut commands: Commands) {
    commands.run_system_cached(restart_game_command);
}

fn restart_game_command(
    game_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if *game_state.get() != GameState::InGame {
        return;
    }
    next_state.set(GameState::GameOver);
}
```

## When to use `run_system_cached_with`

Use `run_system_cached_with` if the command needs per-event payload (`In<T>`), e.g. clicked entity:

```rust
commands.run_system_cached_with(select_actor_command, clicked_entity);
```

Use plain `run_system_cached` when the command can derive everything from world state/resources.

## `Single` vs `Query::single()`

- Prefer `Single<...>` when singleton presence is a hard invariant.
- Keep `Query::single()` guard paths when absence is expected and should degrade gracefully.

## Plugin wiring checklist

For each app command event:

1. Define event type:
   - `#[derive(Event, Reflect)]`
   - `#[reflect(Event)]`
2. Register in plugin:
   - `.register_type::<MyEvent>()`
3. Add event observer:
   - `.add_observer(on_my_event)`
4. Input observer should trigger event, not run command directly.

## BRP usage

Use `world.trigger_event` with the fully-qualified app event type path.

This intentionally routes external automation through the same command observer and cached command logic used by input.
