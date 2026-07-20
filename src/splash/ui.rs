use bevy::prelude::*;

use crate::camera::RenderLayer;
use crate::constants::APPLICATION_TITLE;
use crate::constants::SPLASH_INITIAL_FONT_SIZE;
use crate::constants::SPLASH_SKIP_HINT_ALPHA;
use crate::constants::SPLASH_SKIP_HINT_BOTTOM_OFFSET;
use crate::constants::SPLASH_SKIP_HINT_COLOR;
use crate::constants::SPLASH_SKIP_HINT_FONT_SIZE;
use crate::constants::SPLASH_SKIP_HINT_TEXT;
use crate::constants::SPLASH_SKIP_HINT_WIDTH_PERCENT;

#[derive(Component, Default, Clone)]
pub(crate) struct SplashText;

/// Bottom hint shown during splash to indicate that users can skip.
#[derive(Component, Default, Clone)]
pub(crate) struct SplashSkipHint;

pub(super) fn splash_ui() -> impl SceneList {
    bsn_list![
        (
            SplashText
            Text(APPLICATION_TITLE)
            TextFont {
                font_size: FontSize::Px(SPLASH_INITIAL_FONT_SIZE),
            }
            Node {
                align_self: AlignSelf::Center,
                justify_self: JustifySelf::Center,
                position_type: PositionType::Absolute,
            }
            template_value(RenderLayer::UI.layers())
        ),
        (
            SplashSkipHint
            Text(SPLASH_SKIP_HINT_TEXT)
            TextFont {
                font_size: FontSize::Px(SPLASH_SKIP_HINT_FONT_SIZE),
            }
            template_value(TextLayout::justify(Justify::Center))
            TextColor({SPLASH_SKIP_HINT_COLOR.with_alpha(SPLASH_SKIP_HINT_ALPHA)})
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(SPLASH_SKIP_HINT_BOTTOM_OFFSET),
                left: Val::Px(0.0),
                width: Val::Percent(SPLASH_SKIP_HINT_WIDTH_PERCENT),
            }
            template_value(RenderLayer::UI.layers())
        ),
    ]
}
