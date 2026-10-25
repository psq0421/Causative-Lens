use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TerminalCapabilities {
    pub utf8: bool,
    pub ansi: bool,
    pub color: bool,
    pub legacy_conhost: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TimelineRow {
    pub assertion: String,
    pub event: String,
    pub relation: String,
    pub deviation: bool,
}

/// Detect the minimum terminal contract without assuming a shell.
pub fn detect_terminal() -> TerminalCapabilities {
    let no_color = std::env::var_os("NO_COLOR").is_some();
    let wt = std::env::var_os("WT_SESSION").is_some();
    let conemu = std::env::var("ConEmuANSI")
        .map(|value| value == "ON")
        .unwrap_or(false);
    let term = std::env::var("TERM")
        .unwrap_or_default()
        .to_ascii_lowercase();
    let ansi = !no_color
        && (wt
            || conemu
            || term.contains("xterm")
            || term.contains("ansi")
            || std::env::var_os("ANSICON").is_some());
    let legacy_conhost = !wt && !conemu && !term.contains("xterm") && !term.contains("ansi");
    TerminalCapabilities {
        utf8: true,
        ansi,
        color: ansi,
        legacy_conhost,
    }
}

/// Stable two-lane timeline renderer. Redirected output and old conhost stay plain.
pub fn render_timeline(rows: &[TimelineRow], capabilities: TerminalCapabilities) -> String {
    let mut output = String::from("ASSERTION                         | EVENT\n");
    output.push_str("----------------------------------+----------------------------------\n");
    for row in rows {
        let marker = if row.deviation { "!" } else { "." };
        let line = format!(
            "{marker} {:<32} {} {:<32} {}\n",
            truncate(&row.assertion, 32),
            row.relation,
            truncate(&row.event, 32),
            if row.deviation {
                "DEVIATED"
            } else {
                "observed"
            }
        );
        if capabilities.color && row.deviation {
            output.push_str("\x1b[31m");
            output.push_str(&line);
            output.push_str("\x1b[0m");
        } else {
            output.push_str(&line);
        }
    }
    output
}
fn truncate(value: &str, width: usize) -> String {
    let mut chars = value.chars();
    let mut result: String = chars.by_ref().take(width).collect();
    if value.chars().count() > width && width > 1 {
        result.pop();
        result.push_str("...");
    }
    format!("{result:<width$}")
}

#[cfg(test)]
mod tests {
    use super::*;
    fn caps(ansi: bool) -> TerminalCapabilities {
        TerminalCapabilities {
            utf8: true,
            ansi,
            color: ansi,
            legacy_conhost: !ansi,
        }
    }
    #[test]
    fn timeline_has_two_lanes_and_red_deviation_when_supported() {
        let text = render_timeline(
            &[TimelineRow {
                assertion: "read config".to_owned(),
                event: "FileRead config".to_owned(),
                relation: "->".to_owned(),
                deviation: true,
            }],
            caps(true),
        );
        assert!(text.contains("ASSERTION"));
        assert!(text.contains("\x1b[31m"));
        assert!(text.contains("DEVIATED"));
    }
    #[test]
    fn plain_mode_contains_no_escape_sequences() {
        let text = render_timeline(&[], caps(false));
        assert!(!text.contains('\x1b'));
    }
}
