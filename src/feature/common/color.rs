//! Color helpers used by several features.

use bevy::color::{Color, Mix};

use super::constant::{MUTE_AMOUNT, MUTE_TARGET};

/// A muted version of `color`: mixed toward a neutral gray, so it looks
/// desaturated and dimmer. Works for white too (plain desaturation would
/// leave white unchanged, since white has no saturation).
///
/// `mix` comes from Bevy's `Mix` trait.
pub fn mute(color: Color) -> Color {
    color.mix(&MUTE_TARGET, MUTE_AMOUNT)
}
