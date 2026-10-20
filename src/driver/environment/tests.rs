use super::{color_level, in_ci};
use crate::ColorLevel::{Ansi256, Basic, None as NoColor, TrueColor};

#[test]
fn ci_rules() {
    for (env, expected) in [
        (vec![], false),
        (vec![("CI", "0")], false),
        (vec![("CI", "false")], false),
        (vec![("CI", "")], true),
        (
            vec![("CI", "false"), ("CONTINUOUS_INTEGRATION", "true")],
            true,
        ),
    ] {
        assert_eq!(
            in_ci(|key| env
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| v.to_string())),
            expected,
            "{env:?}"
        );
    }
}

#[test]
fn color_rules() {
    // Every supports-color decision is otherwise invisible to frame tests,
    // which supply their own color level rather than reading the environment.
    for (tty, env, expected) in [
        (
            true,
            vec![("FORCE_COLOR", "false"), ("TERM", "xterm-kitty")],
            NoColor,
        ),
        (
            false,
            vec![("FORCE_COLOR", "true"), ("COLORTERM", "truecolor")],
            TrueColor,
        ),
        (false, vec![("FORCE_COLOR", "")], Basic),
        (
            true,
            vec![("FORCE_COLOR", "0002"), ("TERM", "xterm-kitty")],
            Ansi256,
        ),
        (
            false,
            vec![("FORCE_COLOR", "invalid"), ("TERM", "xterm")],
            Basic,
        ),
        (false, vec![("TF_BUILD", ""), ("AGENT_NAME", "")], Basic),
        (false, vec![("TERM", "xterm-kitty")], NoColor),
        (
            true,
            vec![("TERM", "dumb"), ("CI", "true"), ("GITHUB_ACTIONS", "true")],
            NoColor,
        ),
        (
            true,
            vec![("CI", "false"), ("GITHUB_ACTIONS", "")],
            TrueColor,
        ),
        (true, vec![("CI", "true"), ("TRAVIS", "")], Basic),
        (true, vec![("CI", "true"), ("CI_NAME", "codeship")], Basic),
        (
            true,
            vec![("CI", "true"), ("COLORTERM", "truecolor")],
            NoColor,
        ),
        (true, vec![("TEAMCITY_VERSION", "9.01.0")], Basic),
        (true, vec![("TEAMCITY_VERSION", "10.0")], Basic),
        (
            true,
            vec![("TEAMCITY_VERSION", "9.0.0"), ("FORCE_COLOR", "true")],
            NoColor,
        ),
        (true, vec![("COLORTERM", "truecolor")], TrueColor),
        (true, vec![("TERM", "xterm-kitty")], TrueColor),
        (true, vec![("TERM", "xterm-ghostty")], TrueColor),
        (true, vec![("TERM", "wezterm")], TrueColor),
        (
            true,
            vec![
                ("TERM_PROGRAM", "iTerm.app"),
                ("TERM_PROGRAM_VERSION", "3.0"),
            ],
            TrueColor,
        ),
        (true, vec![("TERM_PROGRAM", "iTerm.app")], Ansi256),
        (true, vec![("TERM_PROGRAM", "Apple_Terminal")], Ansi256),
        (
            true,
            vec![("TERM_PROGRAM", "unknown"), ("TERM", "XTERM-256COLOR")],
            Ansi256,
        ),
        (true, vec![("TERM", "screen")], Basic),
        (true, vec![("TERM", "my-ansi-terminal")], Basic),
        (true, vec![("COLORTERM", "")], Basic),
        (true, vec![], NoColor),
    ] {
        assert_eq!(
            color_level(tty, false, |key| env
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| v.to_string())),
            expected,
            "tty={tty}, {env:?}"
        );
    }
}
