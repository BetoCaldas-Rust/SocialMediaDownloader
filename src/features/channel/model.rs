pub use crate::core::state::ChannelState as ChannelModel;

use chrono::{Days, NaiveDate};

use crate::core::state::DatePreset;
use crate::services::traits::ChannelVideo;

/// Inclusive date window for a preset, relative to `today`. Quick presets
/// size the window to their name (Last7 covers today + 6 days back);
/// All/Custom carry no automatic bounds.
pub fn preset_range(
    preset: DatePreset,
    today: NaiveDate,
) -> (Option<NaiveDate>, Option<NaiveDate>) {
    let back = |days: u64| today.checked_sub_days(Days::new(days)).unwrap_or(today);
    match preset {
        DatePreset::Last7 => (Some(back(6)), Some(today)),
        DatePreset::Last30 => (Some(back(29)), Some(today)),
        DatePreset::Last60 => (Some(back(59)), Some(today)),
        DatePreset::Year => (Some(back(364)), Some(today)),
        DatePreset::All | DatePreset::Custom => (None, None),
    }
}

/// Parses `dd/mm/yyyy` (single digits tolerated). Manual split, no regex.
pub fn parse_br_date(text: &str) -> Option<NaiveDate> {
    let mut parts = text.trim().split('/');
    let day: u32 = parts.next()?.trim().parse().ok()?;
    let month: u32 = parts.next()?.trim().parse().ok()?;
    let year: i32 = parts.next()?.trim().parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    NaiveDate::from_ymd_opt(year, month, day)
}

pub fn selected_count(selected: &[bool]) -> usize {
    selected.iter().filter(|flag| **flag).count()
}

/// Total duration of selected videos. Flat playlists report no file
/// sizes, so the summary bar shows count + duration only — never a
/// fabricated GB estimate.
pub fn total_duration_secs(videos: &[ChannelVideo], selected: &[bool]) -> u64 {
    videos
        .iter()
        .zip(selected.iter())
        .filter(|(_, flag)| **flag)
        .map(|(video, _)| video.duration_secs)
        .sum()
}

pub fn format_duration(total_secs: u64) -> String {
    let hours = total_secs / 3600;
    let minutes = total_secs % 3600 / 60;
    let secs = total_secs % 60;
    if hours > 0 {
        format!("{hours}:{minutes:02}:{secs:02}")
    } else {
        format!("{minutes:02}:{secs:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2024, 3, 15).expect("fixed today")
    }

    #[test]
    fn presets_cover_named_windows() {
        let cases = [
            (DatePreset::Last7, "09/03/2024"),
            (DatePreset::Last30, "15/02/2024"),
            (DatePreset::Last60, "16/01/2024"),
            (DatePreset::Year, "17/03/2023"),
        ];
        for (preset, expected_from) in cases {
            let (from, to) = preset_range(preset, today());
            assert_eq!(
                from.map(|day| day.format("%d/%m/%Y").to_string())
                    .as_deref(),
                Some(expected_from),
                "{preset:?}"
            );
            assert_eq!(to, Some(today()));
        }
    }

    #[test]
    fn all_and_custom_carry_no_bounds() {
        assert_eq!(preset_range(DatePreset::All, today()), (None, None));
        assert_eq!(preset_range(DatePreset::Custom, today()), (None, None));
    }

    #[test]
    fn parses_valid_br_dates() {
        assert_eq!(
            parse_br_date("15/03/2024"),
            NaiveDate::from_ymd_opt(2024, 3, 15)
        );
        assert_eq!(
            parse_br_date("29/02/2024"),
            NaiveDate::from_ymd_opt(2024, 2, 29)
        );
        assert_eq!(
            parse_br_date("1/2/2024"),
            NaiveDate::from_ymd_opt(2024, 2, 1)
        );
    }

    #[test]
    fn rejects_invalid_br_dates() {
        for text in [
            "",
            "2024-03-15",
            "32/01/2024",
            "15/13/2024",
            "29/02/2023",
            "15/03",
            "15/03/2024/extra",
            "aa/bb/cccc",
        ] {
            assert_eq!(parse_br_date(text), None, "{text}");
        }
    }

    #[test]
    fn summary_counts_only_selected() {
        let videos = vec![
            ChannelVideo {
                duration_secs: 60,
                ..Default::default()
            },
            ChannelVideo {
                duration_secs: 120,
                ..Default::default()
            },
        ];
        assert_eq!(selected_count(&[true, false]), 1);
        assert_eq!(total_duration_secs(&videos, &[true, false]), 60);
        assert_eq!(total_duration_secs(&videos, &[true, true]), 180);
    }
}
