use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::services::log_buffer::LogLevel;
use crate::services::traits::{
    LogSink, Transcriber, TranscriptFormat, TranscriptOrder, TranscriptResult,
};
use crate::services::yt_dlp::binary::{hidden_command, resolve_binary};

/// Locale key for "no subtitles exist" failures. The error value sent
/// through the channel is `transcript_error_no_transcript|<url>` so the
/// view can translate the key while history (F5) keeps the url.
pub const NO_TRANSCRIPT_KEY: &str = "transcript_error_no_transcript";

pub fn no_transcript_error(url: &str) -> String {
    format!("{NO_TRANSCRIPT_KEY}|{}", url.trim())
}

pub fn parse_no_transcript_error(err: &str) -> Option<String> {
    err.strip_prefix(NO_TRANSCRIPT_KEY)
        .and_then(|tail| tail.strip_prefix('|'))
        .map(str::to_string)
}

pub fn transcript_error_display_key(err: &str) -> &str {
    if err.starts_with(NO_TRANSCRIPT_KEY) {
        NO_TRANSCRIPT_KEY
    } else {
        err
    }
}

/// One rung of the attempt ladder: which `--sub-langs` value to request
/// and whether `--write-auto-subs` is also passed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranscriptAttempt {
    pub lang: String,
    pub auto_allowed: bool,
}

/// Pure ladder: manual preferred, then auto preferred (if accepted),
/// then manual fallback, then auto fallback (if accepted and configured).
/// A missing/empty/`off` fallback contributes no rungs.
pub fn plan_attempts(order: &TranscriptOrder) -> Vec<TranscriptAttempt> {
    let mut attempts = vec![TranscriptAttempt {
        lang: order.lang.clone(),
        auto_allowed: false,
    }];
    if order.accept_auto {
        attempts.push(TranscriptAttempt {
            lang: order.lang.clone(),
            auto_allowed: true,
        });
    }
    if let Some(fallback) = normalized_fallback(order) {
        attempts.push(TranscriptAttempt {
            lang: fallback.clone(),
            auto_allowed: false,
        });
        if order.accept_auto {
            attempts.push(TranscriptAttempt {
                lang: fallback,
                auto_allowed: true,
            });
        }
    }
    attempts
}

fn normalized_fallback(order: &TranscriptOrder) -> Option<String> {
    let fallback = order.fallback.as_deref().unwrap_or("").trim();
    if fallback.is_empty() || fallback.eq_ignore_ascii_case("off") || fallback == order.lang {
        None
    } else {
        Some(fallback.to_string())
    }
}

/// Pure mapping from yt-dlp stderr to "no subtitles available".
/// Manual substring scan (no regex crate): matches real samples like
/// "WARNING: There are no subtitles for the requested languages".
pub fn is_no_subtitles_stderr(stderr: &str) -> bool {
    let lower = stderr.to_lowercase();
    [
        "there are no subtitles",
        "has no subtitles",
        "no subtitles for the requested languages",
        "requested languages",
        "subtitles not available",
        "no captions",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

pub fn transcript_args(
    attempt: &TranscriptAttempt,
    format: TranscriptFormat,
    job_dir: &Path,
    url: &str,
) -> Vec<String> {
    let template = job_dir.join("%(title)s.%(id)s.%(ext)s");
    let mut args = vec![
        "--skip-download".to_string(),
        "--no-playlist".to_string(),
        "--write-subs".to_string(),
    ];
    if attempt.auto_allowed {
        args.push("--write-auto-subs".to_string());
    }
    args.push("--sub-langs".to_string());
    args.push(attempt.lang.clone());
    args.push("--convert-subs".to_string());
    args.push(format.convert_target().to_string());
    args.push("-o".to_string());
    args.push(template.to_string_lossy().into_owned());
    args.push(url.to_string());
    args
}

/// SRT/VTT -> spoken lines, cue numbers and timing lines dropped,
/// consecutive duplicates collapsed.
/// `timestamps` only affects TXT output: SRT/VTT always keep timings.
pub fn strip_to_plain_text(source: &str) -> String {
    parse_cues(source)
        .into_iter()
        .map(|cue| cue.text)
        .fold(Vec::new(), |mut acc, line| {
            if acc.last().is_none_or(|prev| *prev != line) {
                acc.push(line);
            }
            acc
        })
        .join("\n")
}

/// TXT-with-timestamps variant: `[start] line` per cue, deduped the
/// same way as [`strip_to_plain_text`].
pub fn subs_to_timestamped_text(source: &str) -> String {
    parse_cues(source)
        .into_iter()
        .fold(Vec::new(), |mut acc: Vec<(String, String)>, cue| {
            if acc.last().is_none_or(|(_, prev)| *prev != cue.text) {
                acc.push((cue.start, cue.text));
            }
            acc
        })
        .into_iter()
        .map(|(start, text)| format!("[{start}] {text}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Cue {
    start: String,
    text: String,
}

fn parse_cues(source: &str) -> Vec<Cue> {
    let mut cues = Vec::new();
    let mut pending_start = String::new();
    for raw in source.lines() {
        let line = raw.trim();
        if line.is_empty() || line == "WEBVTT" {
            continue;
        }
        if line.starts_with("NOTE") {
            continue;
        }
        if is_cue_number(line) {
            continue;
        }
        if let Some(start) = split_timing_start(line) {
            pending_start = start;
            continue;
        }
        let text = strip_inline_tags(line);
        if text.is_empty() {
            continue;
        }
        cues.push(Cue {
            start: pending_start.clone(),
            text,
        });
    }
    cues
}

fn is_cue_number(line: &str) -> bool {
    !line.is_empty() && line.bytes().all(|b| b.is_ascii_digit())
}

fn split_timing_start(line: &str) -> Option<String> {
    let arrow = line.find("-->")?;
    let start = line[..arrow].trim();
    if start.is_empty() || !start.bytes().next().is_some_and(|b| b.is_ascii_digit()) {
        return None;
    }
    Some(start.to_string())
}

fn strip_inline_tags(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut in_tag = false;
    for ch in line.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub struct YtDlpTranscriber {
    log: Arc<dyn LogSink>,
}

impl YtDlpTranscriber {
    pub fn new(log: Arc<dyn LogSink>) -> Self {
        Self { log }
    }

    fn note(&self, level: LogLevel, message: &str) {
        self.log.push(level, "yt-dlp", message);
    }
}

#[async_trait::async_trait]
impl Transcriber for YtDlpTranscriber {
    async fn transcribe(&self, order: &TranscriptOrder) -> Result<TranscriptResult, String> {
        let url = order.url.trim().to_string();
        if url.is_empty() {
            return Err("video_error_empty_url".to_string());
        }
        let binary = resolve_binary()?;
        let base_dir = ensure_dir(&order.output_dir).inspect_err(|_| {
            self.note(LogLevel::Error, "unable to create transcript output dir");
        })?;
        for (index, attempt) in plan_attempts(order).iter().enumerate() {
            let job_dir = create_job_dir(&base_dir, index).map_err(|_| {
                self.note(LogLevel::Error, "unable to create transcript job dir");
                "transcript_error_fetch".to_string()
            })?;
            let run = run_attempt(&binary, &url, attempt, order.format, &job_dir).await;
            match run {
                Ok(stderr) => {
                    push_stderr_lines(&stderr, &*self.log);
                    if is_no_subtitles_stderr(&stderr) {
                        remove_job_dir(&job_dir);
                        continue;
                    }
                    match find_sub_file(&job_dir) {
                        Some(found) => {
                            let finalized =
                                finalize_output(&found, &base_dir, order.format, order.timestamps)
                                    .map_err(|_| {
                                        self.note(LogLevel::Error, "unable to finalize transcript");
                                        "transcript_error_fetch".to_string()
                                    })?;
                            remove_job_dir(&job_dir);
                            let size_bytes = std::fs::metadata(&finalized)
                                .map(|meta| meta.len())
                                .unwrap_or(0);
                            self.note(LogLevel::Info, "transcript fetched");
                            return Ok(TranscriptResult {
                                path: finalized,
                                lang_used: attempt.lang.clone(),
                                auto_generated: attempt.auto_allowed,
                                size_bytes,
                            });
                        }
                        None => {
                            remove_job_dir(&job_dir);
                            continue;
                        }
                    }
                }
                Err(stderr) => {
                    push_stderr_lines(&stderr, &*self.log);
                    if is_no_subtitles_stderr(&stderr) {
                        remove_job_dir(&job_dir);
                        continue;
                    }
                    self.note(LogLevel::Warn, "transcript attempt exited non-zero");
                    remove_job_dir(&job_dir);
                    return Err("transcript_error_fetch".to_string());
                }
            }
        }
        self.note(
            LogLevel::Warn,
            "no transcript found for requested languages",
        );
        Err(no_transcript_error(&url))
    }
}

async fn run_attempt(
    binary: &PathBuf,
    url: &str,
    attempt: &TranscriptAttempt,
    format: TranscriptFormat,
    job_dir: &Path,
) -> Result<String, String> {
    let args = transcript_args(attempt, format, job_dir, url);
    let mut command = hidden_command(binary);
    command.args(&args).kill_on_drop(true);
    let output = command
        .output()
        .await
        .map_err(|_| "transcript_error_fetch".to_string())?;
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    if output.status.success() {
        Ok(stderr)
    } else {
        Err(stderr)
    }
}

fn push_stderr_lines(stderr: &str, log: &dyn LogSink) {
    for line in stderr.lines() {
        if !line.trim().is_empty() {
            log.push(LogLevel::Debug, "yt-dlp", line.trim());
        }
    }
}

fn ensure_dir(dir: &Path) -> Result<PathBuf, String> {
    if std::fs::create_dir_all(dir).is_err() {
        return Err("transcript_error_fetch".to_string());
    }
    Ok(dir.to_path_buf())
}

fn create_job_dir(base: &Path, index: usize) -> Result<PathBuf, String> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|span| span.as_nanos())
        .unwrap_or(0);
    let dir = base.join(format!(
        ".smd-transcript-{}-{nanos}-{index}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).map_err(|_| "transcript_error_fetch".to_string())?;
    Ok(dir)
}

fn remove_job_dir(dir: &Path) {
    let _ = std::fs::remove_dir_all(dir);
}

fn find_sub_file(job_dir: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(job_dir).ok()?;
    let mut best: Option<(u64, PathBuf)> = None;
    for entry in entries.flatten() {
        let path = entry.path();
        let is_sub = path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("srt") || ext.eq_ignore_ascii_case("vtt"));
        if !is_sub {
            continue;
        }
        let size = std::fs::metadata(&path).map(|meta| meta.len()).unwrap_or(0);
        let replace = best
            .as_ref()
            .is_none_or(|(best_size, _)| size >= *best_size);
        if replace {
            best = Some((size, path));
        }
    }
    best.map(|(_, path)| path)
}

fn finalize_output(
    found: &Path,
    base_dir: &Path,
    format: TranscriptFormat,
    timestamps: bool,
) -> Result<PathBuf, String> {
    let stem = found
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .ok_or_else(|| "transcript_error_fetch".to_string())?;
    let dest = unique_dest(base_dir, stem, format.extension());
    if format == TranscriptFormat::Txt {
        let source =
            std::fs::read_to_string(found).map_err(|_| "transcript_error_fetch".to_string())?;
        let body = if timestamps {
            subs_to_timestamped_text(&source)
        } else {
            strip_to_plain_text(&source)
        };
        std::fs::write(&dest, body).map_err(|_| "transcript_error_fetch".to_string())?;
    } else if std::fs::rename(found, &dest).is_err() {
        std::fs::copy(found, &dest).map_err(|_| "transcript_error_fetch".to_string())?;
        let _ = std::fs::remove_file(found);
    }
    Ok(dest)
}

fn unique_dest(base_dir: &Path, stem: &str, ext: &str) -> PathBuf {
    let candidate = base_dir.join(format!("{stem}.{ext}"));
    if !candidate.exists() {
        return candidate;
    }
    for index in 1..1000 {
        let candidate = base_dir.join(format!("{stem}-{index}.{ext}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    base_dir.join(format!("{stem}-final.{ext}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn order_for(lang: &str, fallback: Option<&str>, accept_auto: bool) -> TranscriptOrder {
        TranscriptOrder {
            url: "https://example.com/watch?v=abc".to_string(),
            lang: lang.to_string(),
            format: TranscriptFormat::Srt,
            fallback: fallback.map(str::to_string),
            accept_auto,
            timestamps: true,
            output_dir: std::env::temp_dir(),
        }
    }

    #[test]
    fn ladder_manual_only_without_auto_or_fallback() {
        let attempts = plan_attempts(&order_for("pt", None, false));
        assert_eq!(
            attempts,
            vec![TranscriptAttempt {
                lang: "pt".to_string(),
                auto_allowed: false,
            }]
        );
    }

    #[test]
    fn ladder_adds_auto_when_accepted() {
        let attempts = plan_attempts(&order_for("pt", None, true));
        assert_eq!(
            attempts
                .iter()
                .map(|attempt| (attempt.lang.as_str(), attempt.auto_allowed))
                .collect::<Vec<_>>(),
            vec![("pt", false), ("pt", true)]
        );
    }

    #[test]
    fn ladder_covers_fallback_manual_then_auto() {
        let attempts = plan_attempts(&order_for("pt", Some("en"), true));
        assert_eq!(
            attempts
                .iter()
                .map(|attempt| (attempt.lang.as_str(), attempt.auto_allowed))
                .collect::<Vec<_>>(),
            vec![("pt", false), ("pt", true), ("en", false), ("en", true)]
        );
    }

    #[test]
    fn ladder_skips_off_same_or_empty_fallback() {
        for fallback in [Some("off"), Some("OFF"), Some(""), Some("pt"), None] {
            let attempts = plan_attempts(&order_for("pt", fallback, true));
            assert_eq!(attempts.len(), 2, "fallback {fallback:?} must add no rungs");
        }
    }

    #[test]
    fn ladder_fallback_manual_without_auto() {
        let attempts = plan_attempts(&order_for("pt", Some("en"), false));
        assert_eq!(
            attempts
                .iter()
                .map(|attempt| (attempt.lang.as_str(), attempt.auto_allowed))
                .collect::<Vec<_>>(),
            vec![("pt", false), ("en", false)]
        );
    }

    #[test]
    fn stderr_mapping_matches_real_samples() {
        let samples = [
            "WARNING: [Youtube] There are no subtitles for the requested languages",
            "[info] Video has no subtitles",
            "ERROR: Requested languages lack subtitles",
            "WARNING: No captions available for this video",
        ];
        for sample in samples {
            assert!(is_no_subtitles_stderr(sample), "missed: {sample}");
        }
        assert!(!is_no_subtitles_stderr(
            "[download]  23.4% of 85MiB at 3MiB/s ETA 00:21"
        ));
        assert!(!is_no_subtitles_stderr(""));
    }

    #[test]
    fn no_transcript_error_round_trips_url() {
        let err = no_transcript_error("https://example.com/watch?v=abc");
        assert_eq!(transcript_error_display_key(&err), NO_TRANSCRIPT_KEY);
        assert_eq!(
            parse_no_transcript_error(&err).as_deref(),
            Some("https://example.com/watch?v=abc")
        );
        assert_eq!(parse_no_transcript_error("transcript_error_fetch"), None);
        assert_eq!(
            transcript_error_display_key("transcript_error_fetch"),
            "transcript_error_fetch"
        );
    }

    #[test]
    fn strip_srt_drops_numbers_timings_and_dupes() {
        let source = "1\n00:00:00,000 --> 00:00:02,000\nHello <i>world</i>\n\n2\n00:00:02,000 --> 00:00:04,000\nHello world\n\n3\n00:00:04,000 --> 00:00:06,000\nNext line\n";
        assert_eq!(strip_to_plain_text(source), "Hello world\nNext line");
    }

    #[test]
    fn strip_vtt_drops_header_cue_settings_and_dupes() {
        let source = "WEBVTT\n\n00:00.000 --> 00:02.000 align:start position:0%\nFirst line\n\n00:02.000 --> 00:04.000\nFirst line\n\nNOTE comment\n\n00:04.000 --> 00:06.000\nSecond line\n";
        assert_eq!(strip_to_plain_text(source), "First line\nSecond line");
    }

    #[test]
    fn timestamped_txt_keeps_first_start_per_line() {
        let source =
            "1\n00:00:01,000 --> 00:00:02,000\nHello\n\n2\n00:00:02,000 --> 00:00:03,000\nHello\n";
        assert_eq!(subs_to_timestamped_text(source), "[00:00:01,000] Hello");
    }

    #[test]
    fn transcript_args_request_convert_target_and_job_template() {
        let attempt = TranscriptAttempt {
            lang: "pt".to_string(),
            auto_allowed: true,
        };
        let args = transcript_args(
            &attempt,
            TranscriptFormat::Txt,
            Path::new("/tmp/job"),
            "https://v",
        );
        assert!(args.contains(&"--write-auto-subs".to_string()));
        assert!(args.contains(&"pt".to_string()));
        let convert_pos = args
            .iter()
            .position(|arg| arg == "--convert-subs")
            .expect("convert flag");
        assert_eq!(args[convert_pos + 1], "srt");
        assert!(args.iter().any(|arg| arg.contains("%(title)s")));
    }
}
