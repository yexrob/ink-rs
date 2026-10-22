use crate::ColorLevel;

pub(super) fn graphics_allowed(env: impl Fn(&str) -> Option<String>) -> bool {
    env("TERM_PROGRAM").as_deref() != Some("WezTerm") && env("KONSOLE_VERSION").is_none()
}

pub(super) fn in_ci(env: impl Fn(&str) -> Option<String>) -> bool {
    ["CI", "CONTINUOUS_INTEGRATION"]
        .iter()
        .any(|key| env(key).is_some_and(|value| value != "0" && value != "false"))
}

// Chalk's vendored supports-color, for fd 1. Libraries do not consume argv.
pub(super) fn color_level(
    tty: bool,
    windows: bool,
    env: impl Fn(&str) -> Option<String>,
) -> ColorLevel {
    let level = detect_color(tty, windows, env);
    match level {
        0 => ColorLevel::None,
        1 => ColorLevel::Basic,
        2 => ColorLevel::Ansi256,
        _ => ColorLevel::TrueColor,
    }
}

fn detect_color(tty: bool, windows: bool, env: impl Fn(&str) -> Option<String>) -> u8 {
    let force = env("FORCE_COLOR");
    let force = match force.as_deref() {
        None => None,
        Some("false") => return 0,
        Some("true" | "") => Some(1),
        // A numeric value names the exact level.
        Some(value) if value.bytes().all(|byte| byte.is_ascii_digit()) => {
            return parse_int(value).unwrap_or(0).min(3) as u8;
        }
        // envForceColor: Math.min(parseInt(value), 3). NaN, like a negative
        // level, still counts as forced, with no minimum.
        Some(value) => match parse_int(value) {
            Some(0) => return 0,
            Some(level) if level > 0 => Some(level.min(3) as u8),
            _ => Some(0),
        },
    };
    if env("TF_BUILD").is_some() && env("AGENT_NAME").is_some() {
        return 1;
    }
    if !tty && force.is_none() {
        return 0;
    }
    let min = force.unwrap_or(0);
    let term = env("TERM").unwrap_or_default();
    if term == "dumb" {
        return min;
    }
    // The Windows 10 1809 minimum exceeds supports-color's build 14931 cutoff.
    if windows {
        return 3;
    }
    if env("CI").is_some() {
        if ["GITHUB_ACTIONS", "GITEA_ACTIONS", "CIRCLECI"]
            .iter()
            .any(|key| env(key).is_some())
        {
            return 3;
        }
        if ["TRAVIS", "APPVEYOR", "GITLAB_CI", "BUILDKITE", "DRONE"]
            .iter()
            .any(|key| env(key).is_some())
            || env("CI_NAME").as_deref() == Some("codeship")
        {
            return 1;
        }
        return min;
    }
    if let Some(version) = env("TEAMCITY_VERSION") {
        let mut parts = version.split('.');
        let major = parts.next().unwrap_or_default();
        let minor = parts.next();
        let digits = |value: &str| !value.is_empty() && value.bytes().all(|b| b.is_ascii_digit());
        return u8::from(
            (major.len() >= 2 && digits(major) && minor.is_some())
                || (major == "9"
                    && minor
                        .is_some_and(|minor| digits(minor) && minor.bytes().any(|b| b != b'0'))
                    && parts.next().is_some()),
        );
    }
    if env("COLORTERM").as_deref() == Some("truecolor") {
        return 3;
    }
    if matches!(term.as_str(), "xterm-kitty" | "xterm-ghostty" | "wezterm") {
        return 3;
    }
    match env("TERM_PROGRAM").as_deref() {
        Some("iTerm.app") => {
            let version = env("TERM_PROGRAM_VERSION").unwrap_or_default();
            return if parse_int(&version).is_some_and(|major| major >= 3) {
                3
            } else {
                2
            };
        }
        Some("Apple_Terminal") => return 2,
        _ => {}
    }
    let term = term.to_ascii_lowercase();
    if term.ends_with("-256") || term.ends_with("-256color") {
        return 2;
    }
    if ["screen", "xterm", "vt100", "vt220", "rxvt"]
        .iter()
        .any(|prefix| term.starts_with(prefix))
        || ["color", "ansi", "cygwin", "linux"]
            .iter()
            .any(|pattern| term.contains(pattern))
        || env("COLORTERM").is_some()
    {
        return 1;
    }
    min
}

// JS Number.parseInt(value, 10): leading whitespace, a sign, then the digit
// prefix; None for NaN.
fn parse_int(value: &str) -> Option<i64> {
    let value = value.trim_start();
    let (sign, rest) = match value.strip_prefix('-') {
        Some(rest) => (-1, rest),
        None => (1, value.strip_prefix('+').unwrap_or(value)),
    };
    let mut digits = rest.bytes().take_while(u8::is_ascii_digit).peekable();
    digits.peek()?;
    Some(
        sign * digits.fold(0_i64, |n, digit| {
            n.saturating_mul(10).saturating_add(i64::from(digit - b'0'))
        }),
    )
}

#[cfg(test)]
mod tests;
