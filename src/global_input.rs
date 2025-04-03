use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use strum::{
    EnumIter,
    IntoEnumIterator,
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<GlobalAction>::default())
            .init_resource::<ActionState<GlobalAction>>()
            .insert_resource(GlobalAction::global_input_map());
    }
}

// inspector windows don't open full size
#[derive(Actionlike, EnumIter, Reflect, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum GlobalAction {
    AABBs,
    BoundaryInspector,
    CameraConfigInspector,
    Debug,
    Home,
    LightsInspector,
    MissileInspector,
    NateroidInspector,
    PhysicsAABB,
    PlanesInspector,
    PortalInspector,
    Pause,
    SpaceshipInspector,
    SpaceshipControlInspector,
    Stars,
    SuppressNateroids,
}

/// GlobalActions assign keys to do a lot of obvious stuff. Debug is less
/// obvious.
///
/// Use Debug like this - invoke it with a system as follows:
/// ```rust
/// app.add_systems(Update, my_debug_system.run_if(toggle_active(false, GlobalAction::Debug))
/// ```
/// useful when you want to limit the amount of info that is being emitted
///
/// similarly you can also ask for the GlobalAction and use it in your code
/// directly
/// ```rust
/// fn my_system(user_input: Res<ActionState<GlobalAction>>) {
///    if user_input.pressed(&GlobalAction::Debug) {
///       // whatever debug statements you're using will only happen while you
/// press it    }
/// }
/// ```
impl GlobalAction {
    pub fn global_input_map() -> InputMap<Self> {
        fn insert_shift_input(
            input_map: InputMap<GlobalAction>,
            action: GlobalAction,
            key: KeyCode,
        ) -> InputMap<GlobalAction> {
            input_map.with_one_to_many(
                action,
                [
                    ButtonlikeChord::new([KeyCode::ShiftLeft]).with(key),
                    ButtonlikeChord::new([KeyCode::ShiftRight]).with(key),
                ],
            )
        }

        // while fold accumulates each pass - we just do an insert each time as a
        // statement and then return the map at the end of each iteration so the
        // accumulation works
        Self::iter().fold(InputMap::default(), |input_map, action| match action {
            Self::AABBs => input_map.with(action, KeyCode::F1),
            Self::BoundaryInspector => insert_shift_input(input_map, action, KeyCode::KeyB),
            Self::CameraConfigInspector => insert_shift_input(input_map, action, KeyCode::KeyC),
            Self::Debug => insert_shift_input(input_map, action, KeyCode::KeyD),
            Self::Home => input_map.with(action, KeyCode::F12),
            Self::LightsInspector => insert_shift_input(input_map, action, KeyCode::KeyL),
            Self::MissileInspector => insert_shift_input(input_map, action, KeyCode::Digit1),
            Self::NateroidInspector => insert_shift_input(input_map, action, KeyCode::Digit2),
            Self::Pause => input_map.with(action, KeyCode::Escape),
            Self::PhysicsAABB => input_map.with(action, KeyCode::F2),
            Self::PlanesInspector => insert_shift_input(input_map, action, KeyCode::KeyP),
            Self::PortalInspector => insert_shift_input(input_map, action, KeyCode::KeyG),
            Self::SpaceshipInspector => insert_shift_input(input_map, action, KeyCode::Digit3),
            Self::SpaceshipControlInspector => insert_shift_input(input_map, action, KeyCode::Digit4),
            Self::Stars => input_map.with(action, KeyCode::F3),
            Self::SuppressNateroids => input_map.with(action, KeyCode::F4),
        })
    }
}

// #todo: #bevyquestion #rustquestion - how does bevy know how to do the
// dependency injection with this impl?        because it makes using
// toggle_active super intuitive and useful

/// ToggleActive allows us to do something cool - we can use it like the bevy
/// input_toggle_active but it works with leafwing_input_manager input_map
/// entries so we can have simple syntax for toggling systems as a run condition
/// as follows:
///
/// ```
/// .add_systems(Update, my_system.run_if(toggle_active(false, GlobalAction::AABBs)))
/// ```
/// cool, huh? the fact that the closure works with Bevy's dependency injection
/// is rocket science to me- i don't know how it knows to do this but it does
pub fn toggle_active(
    default: bool,
    action: GlobalAction,
) -> impl Fn(Res<ActionState<GlobalAction>>, Local<ToggleState>) -> bool {
    move |action_state: Res<ActionState<GlobalAction>>, mut state: Local<ToggleState>| {
        if action_state.just_pressed(&action) {
            state.state = !state.state;
        }

        if state.state {
            !default
        } else {
            default
        }
    }
}

#[derive(Default)]
pub struct ToggleState {
    pub state: bool,
}

pub fn just_pressed<A: Actionlike>(action: A) -> impl Fn(Res<ActionState<A>>) -> bool {
    move |action_state: Res<ActionState<A>>| action_state.just_pressed(&action)
}
