/// Pure parser for yt-dlp `--newline` stdout lines. Returns `None`
/// for lines carrying no progress information.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProgressTick {
    pub percent: Option<f32>,
    pub total: Option<String>,
    pub speed: Option<String>,
    pub eta: Option<String>,
    pub filename: Option<String>,
}

pub fn parse_progress_line(line: &str) -> Option<ProgressTick> {
    let body = line.strip_prefix("[download]").unwrap_or(line).trim();
    if let Some(rest) = body.strip_prefix("Destination:") {
        let name = rest.trim();
        if name.is_empty() {
            return None;
        }
        return Some(ProgressTick {
            filename: Some(name.to_string()),
            ..Default::default()
        });
    }
    if let Some(name) = parse_merger_target(body) {
        return Some(ProgressTick {
            filename: Some(name),
            ..Default::default()
        });
    }
    parse_percent_line(body)
}

fn parse_merger_target(body: &str) -> Option<String> {
    let marker = "Merging formats into";
    let start = body.find(marker)?;
    let quoted = body[start + marker.len()..].trim();
    let name = quoted.trim_matches('"').trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

fn parse_percent_line(body: &str) -> Option<ProgressTick> {
    let pct_pos = body.find('%')?;
    let percent = scan_percent_before(body, pct_pos)?;
    let after = body[pct_pos + 1..].trim().to_string();
    let mut tick = ProgressTick {
        percent: Some(percent),
        ..Default::default()
    };
    // Shape: "of <total> at <speed> ETA <eta>"; every segment optional.
    let mut rest = after.as_str();
    if let Some(tail) = rest.strip_prefix("of") {
        rest = tail.trim();
    }
    if let Some(at) = rest.find(" at ") {
        tick.total = some_if_present(&rest[..at]);
        rest = rest[at + 4..].trim();
    } else if tick.total.is_none() && !looks_like_eta_segment(rest) {
        tick.total = some_if_present(rest);
        rest = "";
    }
    if let Some(eta_pos) = rest.find("ETA") {
        tick.speed = some_if_present(&rest[..eta_pos]);
        tick.eta = some_if_present(&rest[eta_pos + 3..]);
    } else {
        tick.speed = some_if_present(rest);
    }
    Some(tick)
}

fn scan_percent_before(body: &str, pct_pos: usize) -> Option<f32> {
    let bytes = body.as_bytes();
    let mut start = pct_pos;
    while start > 0 && (bytes[start - 1].is_ascii_digit() || bytes[start - 1] == b'.') {
        start -= 1;
    }
    if start == pct_pos {
        return None;
    }
    body[start..pct_pos].parse::<f32>().ok()
}

fn looks_like_eta_segment(rest: &str) -> bool {
    rest.contains("ETA") || rest.is_empty()
}

fn some_if_present(raw: &str) -> Option<String> {
    let value = raw.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_typical_progress_line() {
        let tick = parse_progress_line("[download]  23.4% of   85.20MiB at    3.21MiB/s ETA 00:21")
            .expect("progress line parses");
        assert!((tick.percent.unwrap_or(-1.0) - 23.4).abs() < 0.01);
        assert_eq!(tick.total.as_deref(), Some("85.20MiB"));
        assert_eq!(tick.speed.as_deref(), Some("3.21MiB/s"));
        assert_eq!(tick.eta.as_deref(), Some("00:21"));
        assert_eq!(tick.filename, None);
    }

    #[test]
    fn parses_complete_line() {
        let tick = parse_progress_line("[download] 100.0% of   85.20MiB at    5.55MiB/s ETA 00:00")
            .expect("complete line parses");
        assert!((tick.percent.unwrap_or(-1.0) - 100.0).abs() < 0.01);
        assert_eq!(tick.eta.as_deref(), Some("00:00"));
    }

    #[test]
    fn parses_early_line_with_kib_speed() {
        let tick = parse_progress_line("[download]   0.0% of    1.20MiB at  512.00KiB/s ETA 00:02")
            .expect("early line parses");
        assert!((tick.percent.unwrap_or(-1.0) - 0.0).abs() < 0.01);
        assert_eq!(tick.total.as_deref(), Some("1.20MiB"));
        assert_eq!(tick.speed.as_deref(), Some("512.00KiB/s"));
    }

    #[test]
    fn parses_destination_filename() {
        let tick = parse_progress_line("[download] Destination: My Video Title [abc123].mp4")
            .expect("destination parses");
        assert_eq!(
            tick.filename.as_deref(),
            Some("My Video Title [abc123].mp4")
        );
    }

    #[test]
    fn parses_merger_target() {
        let tick =
            parse_progress_line("[Merger] Merging formats into \"My Video Title [abc123].mp4\"")
                .expect("merger line parses");
        assert_eq!(
            tick.filename.as_deref(),
            Some("My Video Title [abc123].mp4")
        );
    }

    #[test]
    fn ignores_unrelated_lines() {
        assert_eq!(parse_progress_line("[info] Fetching metadata"), None);
        assert_eq!(parse_progress_line(""), None);
        assert_eq!(
            parse_progress_line("[download] file has already been downloaded"),
            None
        );
    }
}
