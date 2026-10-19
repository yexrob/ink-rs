//! Ported from ink test/colorize.ts @ 26d2c3f.

mod common;

use ink_rs::*;

#[test]
fn rgb_foreground_colors_allow_whitespace_around_components() {
    for color in [
        "rgb(232 , 131 , 136)",
        "rgb(  232,  131,  136  )",
        "rgb(\t232\t,\n131\n,\t136\t)",
    ] {
        assert_eq!(
            color.parse::<Color>(),
            Ok(Color::Rgb(232, 131, 136)),
            "{color:?}"
        );
    }
}

#[test]
fn rgb_background_colors_allow_whitespace_around_components() {
    for color in [
        "rgb(232 , 131 , 136)",
        "rgb(  232,  131,  136  )",
        "rgb(\t232\t,\n131\n,\t136\t)",
    ] {
        assert_eq!(
            color.parse::<Color>(),
            Ok(Color::Rgb(232, 131, 136)),
            "{color:?}"
        );
    }
}

#[test]
fn ignores_invalid_foreground_color_level() {
    assert!("level".parse::<Color>().is_err());
}

#[test]
fn ignores_invalid_background_color_level() {
    assert!("level".parse::<Color>().is_err());
}

#[test]
fn ignores_invalid_foreground_color_constructor() {
    assert!("constructor".parse::<Color>().is_err());
}

#[test]
fn ignores_invalid_background_color_constructor() {
    assert!("constructor".parse::<Color>().is_err());
}

#[test]
fn ignores_invalid_foreground_color_bold() {
    assert!("bold".parse::<Color>().is_err());
}

#[test]
fn ignores_invalid_background_color_bold() {
    assert!("bold".parse::<Color>().is_err());
}

#[test]
fn ignores_invalid_foreground_color_bgred() {
    assert!("bgRed".parse::<Color>().is_err());
}

#[test]
fn ignores_invalid_background_color_bgred() {
    assert!("bgRed".parse::<Color>().is_err());
}

#[test]
fn ignores_invalid_foreground_color_unknown() {
    assert!("unknown".parse::<Color>().is_err());
}

#[test]
fn ignores_invalid_background_color_unknown() {
    assert!("unknown".parse::<Color>().is_err());
}

#[test]
fn ignores_invalid_foreground_color_rgb_23_2_131_136() {
    assert!("rgb(23 2, 131, 136)".parse::<Color>().is_err());
}

#[test]
fn ignores_invalid_background_color_rgb_23_2_131_136() {
    assert!("rgb(23 2, 131, 136)".parse::<Color>().is_err());
}

#[test]
fn ignores_invalid_foreground_color_rgb_232_136() {
    assert!("rgb(232, , 136)".parse::<Color>().is_err());
}

#[test]
fn ignores_invalid_background_color_rgb_232_136() {
    assert!("rgb(232, , 136)".parse::<Color>().is_err());
}
