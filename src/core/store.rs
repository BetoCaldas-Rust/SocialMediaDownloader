use std::path::Path;
use std::sync::Arc;

use tokio::runtime::{Builder, Runtime};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;

use super::effect::Effect;
use super::intent::{AppIntent, ChannelIntent, TranscriptIntent, VideoIntent};
use super::state::{AppState, TranscriptStatus, VideoStatus};
use crate::features::{channel, console, history, settings, transcript, video};
use crate::services::log_buffer::LogLevel;
use crate::services::traits::{
    BatchItem, ChannelOrder, ChannelProvider, DownloadOrder, DownloadProgress, Downloader,
    LocaleProvider, LogSink, MetadataProvider, Transcriber, TranscriptOrder, VideoQuality,
};
use crate::services::yt_dlp::downloader::default_download_dir;

pub fn update(state: &mut AppState, intent: &AppIntent) -> Vec<Effect> {
    match intent {
        AppIntent::Navigate(screen) => {
            state.screen = *screen;
            vec![Effect::PushLog {
                level: LogLevel::Info,
                source: "navigation".to_string(),
                message: format!("navigated to {screen:?}"),
            }]
        }
        AppIntent::Video(inner) => {
            let mut effects = video::update::apply(&mut state.video, inner);
            if matches!(inner, VideoIntent::DownloadFinished(Ok(_)))
                && state.video.include_transcript
            {
                let url = state.video.url.trim().to_string();
                if !url.is_empty() {
                    state.transcript.input = url;
                    effects.extend(transcript::update::apply(
                        &mut state.transcript,
                        &TranscriptIntent::FetchTranscript,
                    ));
                }
            }
            effects
        }
        AppIntent::Transcript(inner) => transcript::update::apply(&mut state.transcript, inner),
        AppIntent::Channel(inner) => {
            let mut effects = channel::update::apply(&mut state.channel, inner);
            for effect in &mut effects {
                if let Effect::DownloadBatch { transcript, .. } = effect {
                    *transcript = transcript::update::order_from(&state.transcript);
                }
            }
            effects
        }
        AppIntent::History(inner) => history::update::apply(&mut state.history, inner),
        AppIntent::Console(inner) => console::update::apply(&mut state.console, inner),
        AppIntent::Settings(inner) => settings::update::apply(&mut state.settings, inner),
    }
}

pub struct Store {
    state: AppState,
    locale_provider: Arc<dyn LocaleProvider>,
    log_sink: Arc<dyn LogSink>,
    metadata: Arc<dyn MetadataProvider>,
    downloader: Arc<dyn Downloader>,
    transcriber: Arc<dyn Transcriber>,
    channel: Arc<dyn ChannelProvider>,
    runtime: Option<Arc<Runtime>>,
    intent_tx: UnboundedSender<AppIntent>,
    intent_rx: UnboundedReceiver<AppIntent>,
    active_task: Option<JoinHandle<()>>,
    active_transcript_task: Option<JoinHandle<()>>,
    active_channel_task: Option<JoinHandle<()>>,
}

impl Store {
    pub fn new(
        locale_provider: Arc<dyn LocaleProvider>,
        log_sink: Arc<dyn LogSink>,
        metadata: Arc<dyn MetadataProvider>,
        downloader: Arc<dyn Downloader>,
        transcriber: Arc<dyn Transcriber>,
        channel: Arc<dyn ChannelProvider>,
    ) -> Self {
        let state = AppState {
            locale: locale_provider.current(),
            ..Default::default()
        };
        let (intent_tx, intent_rx) = unbounded_channel();
        Self {
            state,
            locale_provider,
            log_sink,
            metadata,
            downloader,
            transcriber,
            channel,
            runtime: build_runtime(),
            intent_tx,
            intent_rx,
            active_task: None,
            active_transcript_task: None,
            active_channel_task: None,
        }
    }

    pub fn state(&self) -> &AppState {
        &self.state
    }

    pub fn available_locales(&self) -> Vec<String> {
        self.locale_provider.available()
    }

    pub fn is_busy(&self) -> bool {
        matches!(
            self.state.video.status,
            VideoStatus::Resolving | VideoStatus::Downloading
        ) || matches!(
            self.state.transcript.status,
            TranscriptStatus::Resolving | TranscriptStatus::Downloading
        ) || self.state.channel.status.is_busy()
    }

    pub fn dispatch(&mut self, intent: AppIntent) {
        let effects = update(&mut self.state, &intent);
        self.execute(&effects);
    }

    /// Drains intents sent back by background tokio tasks. Called once
    /// per egui frame before rendering; keeps all state changes on the
    /// UI thread so the reducer stays the single writer.
    pub fn drain_pending(&mut self) {
        while let Ok(intent) = self.intent_rx.try_recv() {
            self.dispatch(intent);
        }
    }

    fn execute(&mut self, effects: &[Effect]) {
        for effect in effects {
            match effect {
                Effect::ReloadLocale(locale) => {
                    self.locale_provider.set(locale);
                    self.state.locale = self.locale_provider.current();
                }
                Effect::PushLog {
                    level,
                    source,
                    message,
                } => {
                    self.log_sink.push(*level, source, message);
                }
                Effect::FetchMetadata { url } => self.spawn_fetch(url.clone()),
                Effect::StartDownload { url, quality } => {
                    self.spawn_download(url.clone(), *quality)
                }
                Effect::CancelActive => {
                    if let Some(handle) = self.active_task.take() {
                        handle.abort();
                    }
                }
                Effect::CancelTranscript => {
                    if let Some(handle) = self.active_transcript_task.take() {
                        handle.abort();
                    }
                }
                Effect::FetchTranscript { order } => self.spawn_transcript(order.clone()),
                Effect::FetchChannelPreview { order } => self.spawn_channel_preview(order.clone()),
                Effect::CancelChannel => {
                    if let Some(handle) = self.active_channel_task.take() {
                        handle.abort();
                    }
                }
                Effect::DownloadBatch {
                    items,
                    include_video,
                    include_transcript,
                    quality,
                    transcript,
                } => self.spawn_batch(
                    items.clone(),
                    *include_video,
                    *include_transcript,
                    *quality,
                    transcript.clone(),
                ),
                Effect::RevealInFolder(path) => reveal_in_folder(path, &self.log_sink),
            }
        }
    }

    fn track(&mut self, handle: JoinHandle<()>) {
        if let Some(previous) = self.active_task.replace(handle) {
            previous.abort();
        }
    }

    /// Transcript work runs in its own slot so a video download and a
    /// transcript fetch (including the auto-fetch after a video
    /// completes) proceed independently; `CancelActive` stays
    /// video-only and `CancelTranscript` aborts this slot.
    fn track_transcript(&mut self, handle: JoinHandle<()>) {
        if let Some(previous) = self.active_transcript_task.replace(handle) {
            previous.abort();
        }
    }

    fn spawn_fetch(&mut self, url: String) {
        let Some(runtime) = self.runtime.clone() else {
            self.dispatch(AppIntent::Video(VideoIntent::MetadataReceived(Err(
                "video_error_fetch".to_string(),
            ))));
            return;
        };
        let provider = self.metadata.clone();
        let intent_tx = self.intent_tx.clone();
        self.track(runtime.spawn(async move {
            let result = provider.fetch_metadata(&url).await;
            let _ = intent_tx.send(AppIntent::Video(VideoIntent::MetadataReceived(result)));
        }));
    }

    fn spawn_download(&mut self, url: String, quality: crate::services::traits::VideoQuality) {
        let Some(runtime) = self.runtime.clone() else {
            self.dispatch(AppIntent::Video(VideoIntent::DownloadFinished(Err(
                "video_error_download".to_string(),
            ))));
            return;
        };
        let order = DownloadOrder {
            url,
            quality,
            output_dir: default_download_dir(),
        };
        let downloader = self.downloader.clone();
        let intent_tx = self.intent_tx.clone();
        self.track(runtime.spawn(async move {
            let (progress_tx, mut progress_rx) = unbounded_channel::<DownloadProgress>();
            let relay_tx = intent_tx.clone();
            let relay = tokio::spawn(async move {
                while let Some(tick) = progress_rx.recv().await {
                    let _ = relay_tx.send(AppIntent::Video(VideoIntent::ProgressTick(tick)));
                }
            });
            let result = downloader.download(order, progress_tx).await;
            relay.abort();
            let _ = intent_tx.send(AppIntent::Video(VideoIntent::DownloadFinished(result)));
        }));
    }

    fn spawn_transcript(&mut self, order: TranscriptOrder) {
        let Some(runtime) = self.runtime.clone() else {
            self.dispatch(AppIntent::Transcript(TranscriptIntent::TranscriptFinished(
                Err("transcript_error_fetch".to_string()),
            )));
            return;
        };
        let transcriber = self.transcriber.clone();
        let intent_tx = self.intent_tx.clone();
        self.track_transcript(runtime.spawn(async move {
            let result = transcriber.transcribe(&order).await;
            let _ = intent_tx.send(AppIntent::Transcript(TranscriptIntent::TranscriptFinished(
                result,
            )));
        }));
    }

    fn track_channel(&mut self, handle: JoinHandle<()>) {
        if let Some(previous) = self.active_channel_task.replace(handle) {
            previous.abort();
        }
    }

    fn spawn_channel_preview(&mut self, order: ChannelOrder) {
        let Some(runtime) = self.runtime.clone() else {
            self.dispatch(AppIntent::Channel(ChannelIntent::PreviewReceived(Err(
                "channel_error_fetch".to_string(),
            ))));
            return;
        };
        let provider = self.channel.clone();
        let intent_tx = self.intent_tx.clone();
        self.track_channel(runtime.spawn(async move {
            let result = provider.preview(&order).await;
            let _ = intent_tx.send(AppIntent::Channel(ChannelIntent::PreviewReceived(result)));
        }));
    }

    /// Batch downloads run as ONE task over items SEQUENTIALLY.
    /// Parallel downloads are an F6 (settings) concern; until then the
    /// single task keeps progress attribution trivial and avoids
    /// hammering the remote. Each item reuses `Downloader::download`
    /// with a per-item progress relay, then `Transcriber::transcribe`
    /// with the prefs snapshotted at dispatch time. Aborting this task
    /// drops the in-flight child (`kill_on_drop`), cancelling the
    /// whole batch.
    fn spawn_batch(
        &mut self,
        items: Vec<BatchItem>,
        include_video: bool,
        include_transcript: bool,
        quality: VideoQuality,
        transcript: TranscriptOrder,
    ) {
        let Some(runtime) = self.runtime.clone() else {
            for (index, _) in items.iter().enumerate() {
                self.dispatch(AppIntent::Channel(ChannelIntent::BatchItemFinished {
                    index,
                    video_result: include_video.then(|| Err("video_error_download".to_string())),
                    transcript_result: include_transcript
                        .then(|| Err("transcript_error_fetch".to_string())),
                }));
            }
            return;
        };
        let downloader = self.downloader.clone();
        let transcriber = self.transcriber.clone();
        let intent_tx = self.intent_tx.clone();
        self.track_channel(runtime.spawn(async move {
            for (index, item) in items.iter().enumerate() {
                let video_result = if include_video {
                    Some(download_one(&downloader, &intent_tx, index, item, quality).await)
                } else {
                    None
                };
                let transcript_result = if include_transcript {
                    Some(transcribe_one(&transcriber, &transcript, &item.url).await)
                } else {
                    None
                };
                let _ = intent_tx.send(AppIntent::Channel(ChannelIntent::BatchItemFinished {
                    index,
                    video_result,
                    transcript_result,
                }));
            }
        }));
    }
}

async fn download_one(
    downloader: &Arc<dyn Downloader>,
    intent_tx: &UnboundedSender<AppIntent>,
    index: usize,
    item: &BatchItem,
    quality: VideoQuality,
) -> Result<crate::services::traits::DownloadTicket, String> {
    let (progress_tx, mut progress_rx) = unbounded_channel::<DownloadProgress>();
    let relay_tx = intent_tx.clone();
    let relay = tokio::spawn(async move {
        while let Some(tick) = progress_rx.recv().await {
            let _ = relay_tx.send(AppIntent::Channel(ChannelIntent::BatchTick { index, tick }));
        }
    });
    let order = DownloadOrder {
        url: item.url.clone(),
        quality,
        output_dir: default_download_dir(),
    };
    let result = downloader.download(order, progress_tx).await;
    relay.abort();
    result
}

async fn transcribe_one(
    transcriber: &Arc<dyn Transcriber>,
    prefs: &TranscriptOrder,
    url: &str,
) -> Result<crate::services::traits::TranscriptResult, String> {
    let order = TranscriptOrder {
        url: url.to_string(),
        ..prefs.clone()
    };
    transcriber.transcribe(&order).await
}

fn build_runtime() -> Option<Arc<Runtime>> {
    match Builder::new_multi_thread().enable_all().build() {
        Ok(runtime) => Some(Arc::new(runtime)),
        Err(err) => {
            tracing::error!(target: "startup", "async runtime unavailable: {err}");
            None
        }
    }
}

fn reveal_in_folder(path: &Path, log_sink: &Arc<dyn LogSink>) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let spawned = std::process::Command::new("explorer")
            .arg("/select,")
            .arg(path)
            .creation_flags(0x08000000)
            .spawn()
            .is_ok();
        log_sink.push(
            if spawned {
                LogLevel::Info
            } else {
                LogLevel::Warn
            },
            "download",
            if spawned {
                "revealed output in file manager"
            } else {
                "unable to reveal output"
            },
        );
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        log_sink.push(
            LogLevel::Info,
            "download",
            "reveal not supported on this platform",
        );
    }
}

#[cfg(test)]
mod tests {
    use super::super::state::Screen;
    use super::*;
    use crate::services::locale::LocaleService;
    use crate::services::log_sink::BufferLogSink;
    use crate::services::stubs::{
        StubChannelProvider, StubDownloader, StubMetadataProvider, StubTranscriber,
    };

    fn test_store() -> Store {
        Store::new(
            Arc::new(LocaleService),
            Arc::new(BufferLogSink),
            Arc::new(StubMetadataProvider),
            Arc::new(StubDownloader),
            Arc::new(StubTranscriber),
            Arc::new(StubChannelProvider),
        )
    }

    #[test]
    fn navigate_changes_screen() {
        let mut store = test_store();
        assert_eq!(store.state().screen, Screen::Video);
        store.dispatch(AppIntent::Navigate(Screen::Console));
        assert_eq!(store.state().screen, Screen::Console);
        store.dispatch(AppIntent::Navigate(Screen::Settings));
        assert_eq!(store.state().screen, Screen::Settings);
    }

    #[test]
    fn background_failure_returns_as_intent() {
        let mut store = test_store();
        store.dispatch(AppIntent::Video(VideoIntent::SetUrl(
            "https://example.com/v".to_string(),
        )));
        store.dispatch(AppIntent::Video(VideoIntent::FetchMetadata));
        assert_eq!(store.state().video.status, VideoStatus::Resolving);
        for _ in 0..200 {
            store.drain_pending();
            if store.state().video.status == VideoStatus::Failed {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(store.state().video.status, VideoStatus::Failed);
    }

    #[test]
    fn transcript_failure_returns_as_intent() {
        let mut store = test_store();
        store.dispatch(AppIntent::Transcript(TranscriptIntent::SetInput(
            "https://example.com/v".to_string(),
        )));
        store.dispatch(AppIntent::Transcript(TranscriptIntent::FetchTranscript));
        assert_eq!(store.state().transcript.status, TranscriptStatus::Resolving);
        for _ in 0..200 {
            store.drain_pending();
            if store.state().transcript.status == TranscriptStatus::Failed {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(store.state().transcript.status, TranscriptStatus::Failed);
        assert_eq!(
            store.state().transcript.error.as_deref(),
            Some("transcript_error_fetch")
        );
    }

    #[test]
    fn channel_failure_returns_as_intent() {
        use super::super::state::ChannelStatus;
        use crate::core::intent::ChannelIntent;
        let mut store = test_store();
        store.dispatch(AppIntent::Channel(ChannelIntent::SetInput(
            "https://www.youtube.com/@demo/videos".to_string(),
        )));
        store.dispatch(AppIntent::Channel(ChannelIntent::FetchPreview));
        assert!(store.is_busy());
        for _ in 0..200 {
            store.drain_pending();
            if store.state().channel.status == ChannelStatus::Failed {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(store.state().channel.status, ChannelStatus::Failed);
        assert_eq!(
            store.state().channel.error_key.as_deref(),
            Some("channel_error_fetch")
        );
    }

    #[test]
    fn video_completion_auto_dispatches_transcript() {
        use crate::services::traits::DownloadTicket;
        let mut store = test_store();
        store.dispatch(AppIntent::Video(VideoIntent::SetUrl(
            "https://example.com/v".to_string(),
        )));
        store.dispatch(AppIntent::Video(VideoIntent::SetTranscript(true)));
        store.dispatch(AppIntent::Video(VideoIntent::StartDownload));
        store.dispatch(AppIntent::Video(VideoIntent::DownloadFinished(Ok(
            DownloadTicket {
                id: "vid".to_string(),
                path: std::path::PathBuf::from("/tmp/vid.mp4"),
            },
        ))));
        assert_eq!(store.state().video.status, VideoStatus::Completed);
        assert_eq!(
            store.state().transcript.input,
            "https://example.com/v".to_string()
        );
        assert!(store.state().transcript.status.is_busy());
    }
}
