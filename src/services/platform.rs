pub fn platform_folder(url: &str) -> &'static str {
    let lower = url.to_lowercase();
    if lower.contains("youtube.com") || lower.contains("youtu.be") {
        "youtube"
    } else if lower.contains("x.com") || lower.contains("twitter.com") {
        "twitter"
    } else if lower.contains("instagram.com") {
        "instagram"
    } else if lower.contains("tiktok.com") {
        "tiktok"
    } else if lower.contains("facebook.com") || lower.contains("fb.watch") {
        "facebook"
    } else if lower.contains("threads.com") || lower.contains("threads.net") {
        "threads"
    } else if lower.contains("reddit.com") || lower.contains("redd.it") {
        "reddit"
    } else if lower.contains("twitch.tv") {
        "twitch"
    } else if lower.contains("vimeo.com") {
        "vimeo"
    } else if lower.contains("dailymotion.com") || lower.contains("dai.ly") {
        "dailymotion"
    } else {
        "outros"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_known_platforms() {
        assert_eq!(
            platform_folder("https://www.youtube.com/watch?v=abc"),
            "youtube"
        );
        assert_eq!(platform_folder("https://youtu.be/abc"), "youtube");
        assert_eq!(
            platform_folder("https://x.com/user/status/123?s=20"),
            "twitter"
        );
        assert_eq!(
            platform_folder("https://www.instagram.com/reel/abc/"),
            "instagram"
        );
        assert_eq!(
            platform_folder("https://www.tiktok.com/@u/video/123"),
            "tiktok"
        );
        assert_eq!(
            platform_folder("https://www.facebook.com/watch/?v=1"),
            "facebook"
        );
    }

    #[test]
    fn unknown_urls_fall_back_to_outros() {
        assert_eq!(platform_folder("https://example.com/video.mp4"), "outros");
        assert_eq!(platform_folder("not a url"), "outros");
        assert_eq!(platform_folder(""), "outros");
    }
}
