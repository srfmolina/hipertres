use bevy::color::{Color, Saturation};

use super::color::mute;

#[test]
fn mute_makes_white_darker_and_colors_less_saturated() {
    let white = mute(Color::WHITE).to_srgba();
    assert!(white.red < 1.0);

    let blue = Color::srgb(0.25, 0.55, 0.95);
    assert!(mute(blue).saturation() < blue.saturation());
}
