use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::runtime::{Builder, Runtime};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Semaphore;
use tokio::task::{JoinHandle, JoinSet};

use super::effect::Effect;
use super::intent::{AppIntent, ChannelIntent, SettingsIntent, TranscriptIntent, VideoIntent};
use super::state::{
    apply_config_defaults, AppState, Notice, Screen, SettingsState, TranscriptStatus, VideoStatus,
};
use crate::features::{channel, console, history, settings, transcript, video};
use crate::services::log_buffer::{LevelHandle, LogLevel};
use crate::services::traits::{
    file_name, new_history_id, retry_plan, BatchItem, ChannelOrder, ChannelProvider, DownloadOrder,
    DownloadProgress, Downloader, HistoryEntry, HistoryStore, LocaleProvider, LogSink,
    MetadataProvider, Transcriber, TranscriptOrder, VideoQuality,
};
use crate::services::yt_dlp::binary::{hidden_command, resolve_binary};
use crate::storage::config::{
    log_file_for, logs_dir, prune_old_logs, resolve_download_dir, resolve_output_dir, AppConfig,
};

pub fn update(state: &mut AppState, intent: &AppIntent) -> Vec<Effect> {
    match intent {
        AppIntent::Navigate(screen) => {
            state.screen = *screen;
            let mut effects = vec![Effect::PushLog {
                level: LogLevel::Info,
                source: "navigation".to_string(),
                message: format!("navigated to {screen:?}"),
            }];
            if *screen == Screen::Settings {
                effects.push(Effect::VerifyAutostart);
                if state.settings.ytdlp_version.is_none() && !state.settings.ytdlp_checking {
                    state.settings.ytdlp_checking = true;
                    effects.push(Effect::CheckYtDlpVersion);
                }
            }
            effects
        }
        AppIntent::DismissNotice(id) => {
            state.notices.retain(|notice| notice.id != *id);
            Vec::new()
        }
        AppIntent::Video(inner) => {
            let mut effects = video::update::apply(&mut state.video, inner);
            if let VideoIntent::DownloadFinished(Ok(ticket)) = inner {
                if state.settings.notify_on_complete {
                    let detail = file_name(&ticket.path).unwrap_or_else(|| ticket.id.clone());
                    push_notice(state, "notice_download_done", detail);
                }
                if state.video.include_transcript {
                    let url = state.video.url.trim().to_string();
                    if !url.is_empty() {
                        state.transcript.input = url;
                        effects.extend(transcript::update::apply(
                            &mut state.transcript,
                            &TranscriptIntent::FetchTranscript,
                        ));
                    }
                }
            }
            effects
        }
        AppIntent::Transcript(inner) => {
            let effects = transcript::update::apply(&mut state.transcript, inner);
            if let TranscriptIntent::TranscriptFinished(Ok(ticket)) = inner {
                if state.settings.notify_on_complete {
                    let detail =
                        file_name(&ticket.path).unwrap_or_else(|| ticket.lang_used.clone());
                    push_notice(state, "notice_transcript_done", detail);
                }
            }
            effects
        }
        AppIntent::Channel(inner) => {
            let mut effects = channel::update::apply(&mut state.channel, inner);
            for effect in &mut effects {
                if let Effect::DownloadBatch { transcript, .. } = effect {
                    *transcript = transcript::update::order_from(&state.transcript);
                }
            }
            if let Some(order) = effects.iter().find_map(|effect| match effect {
                Effect::DownloadBatch { transcript, .. } => Some(transcript.clone()),
                _ => None,
            }) {
                state.channel.last_transcript_order = Some(order);
            }
            if matches!(inner, ChannelIntent::BatchItemFinished { .. })
                && state.channel.status == crate::core::state::ChannelStatus::Completed
                && state.settings.notify_on_complete
                && !state.channel.items.is_empty()
            {
                push_notice(
                    state,
                    "notice_batch_done",
                    format!("{}", state.channel.items.len()),
                );
            }
            effects
        }
        AppIntent::History(inner) => {
            let _ = inner;
            Vec::new()
        }
        AppIntent::Console(inner) => console::update::apply(&mut state.console, inner),
        AppIntent::Settings(inner) => settings::update::apply(&mut state.settings, inner),
    }
}

fn push_notice(state: &mut AppState, key: &str, detail: String) {
    state.notices.push(Notice {
        id: new_history_id(),
        message_key: key.to_string(),
        detail,
    });
    while state.notices.len() > 5 {
        state.notices.remove(0);
    }
}

pub struct Store {
    state: AppState,
    config: AppConfig,
    saver: Arc<dyn Fn(&AppConfig) + Send + Sync>,
    level_handle: Option<LevelHandle>,
    log_file: Option<BufWriter<std::fs::File>>,
    log_file_day: Option<String>,
    locale_provider: Arc<dyn LocaleProvider>,
    log_sink: Arc<dyn LogSink>,
    metadata: Arc<dyn MetadataProvider>,
    downloader: Arc<dyn Downloader>,
    transcriber: Arc<dyn Transcriber>,
    channel: Arc<dyn ChannelProvider>,
    history: Arc<dyn HistoryStore>,
    runtime: Option<Arc<Runtime>>,
    intent_tx: UnboundedSender<AppIntent>,
    intent_rx: UnboundedReceiver<AppIntent>,
    active_task: Option<JoinHandle<()>>,
    active_transcript_task: Option<JoinHandle<()>>,
    active_channel_task: Option<JoinHandle<()>>,
}

impl Store {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        locale_provider: Arc<dyn LocaleProvider>,
        log_sink: Arc<dyn LogSink>,
        metadata: Arc<dyn MetadataProvider>,
        downloader: Arc<dyn Downloader>,
        transcriber: Arc<dyn Transcriber>,
        channel: Arc<dyn ChannelProvider>,
        history: Arc<dyn HistoryStore>,
        config: AppConfig,
        level_handle: Option<LevelHandle>,
    ) -> Self {
        Self::new_with_saver(
            locale_provider,
            log_sink,
            metadata,
            downloader,
            transcriber,
            channel,
            history,
            config,
            level_handle,
            Arc::new(|config| config.save()),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_saver(
        locale_provider: Arc<dyn LocaleProvider>,
        log_sink: Arc<dyn LogSink>,
        metadata: Arc<dyn MetadataProvider>,
        downloader: Arc<dyn Downloader>,
        transcriber: Arc<dyn Transcriber>,
        channel: Arc<dyn ChannelProvider>,
        history: Arc<dyn HistoryStore>,
        config: AppConfig,
        level_handle: Option<LevelHandle>,
        saver: Arc<dyn Fn(&AppConfig) + Send + Sync>,
    ) -> Self {
        let mut state = AppState {
            locale: locale_provider.current(),
            ..Default::default()
        };
        state.settings = SettingsState::from_config(&config);
        apply_config_defaults(&mut state, &config);
        let (intent_tx, intent_rx) = unbounded_channel();
        Self {
            state,
            config,
            saver,
            level_handle,
            log_file: None,
            log_file_day: None,
            locale_provider,
            log_sink,
            metadata,
            downloader,
            transcriber,
            channel,
            history,
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

    pub fn download_dir(&self) -> PathBuf {
        resolve_download_dir(&self.config)
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

    pub fn history_entries(&self) -> Vec<HistoryEntry> {
        self.history.entries()
    }

    pub fn history_count(&self) -> usize {
        self.history.len()
    }

    pub fn history_totals(&self) -> crate::features::history::update::HistoryTotals {
        crate::features::history::update::totals(&self.history.entries())
    }

    pub fn dispatch(&mut self, intent: AppIntent) {
        if let AppIntent::History(inner) = &intent {
            let entries = self.history.entries();
            let dir = resolve_download_dir(&self.config);
            let effects = history::update::apply_in(&mut self.state.history, inner, &entries, &dir);
            self.execute(&effects);
            return;
        }
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
                    self.append_log_file(*level, source, message);
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
                Effect::FetchTranscript { order } => {
                    let mut adjusted = order.clone();
                    adjusted.output_dir = resolve_download_dir(&self.config);
                    self.spawn_transcript(adjusted);
                }
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
                } => {
                    let mut adjusted = transcript.clone();
                    adjusted.output_dir = resolve_download_dir(&self.config);
                    self.spawn_batch(
                        items.clone(),
                        *include_video,
                        *include_transcript,
                        *quality,
                        adjusted,
                    );
                }
                Effect::RevealInFolder(path) => reveal_in_folder(path, &self.log_sink),
                Effect::RecordHistory(entry) => {
                    self.history.record((**entry).clone());
                }
                Effect::RetryEntry(entry) => self.exec_retry((**entry).clone()),
                Effect::ClearHistory => {
                    self.history.clear();
                }
                Effect::SaveSettings => {
                    self.state.settings.apply_to_config(&mut self.config);
                    (self.saver)(&self.config);
                    if !self.config.save_logs {
                        self.close_log_file();
                    }
                }
                Effect::ApplyLogLevel(level) => {
                    if let Some(handle) = &self.level_handle {
                        let filter = level.tracing_filter();
                        let _ = handle.modify(|current| *current = filter);
                    }
                }
                Effect::PickDownloadDir => self.spawn_folder_picker(),
                Effect::CheckYtDlpVersion => self.spawn_ytdlp_check(false),
                Effect::UpdateYtDlp => self.spawn_ytdlp_update(),
                Effect::VerifyAutostart => self.spawn_autostart_verify(),
                Effect::SetAutostart(enabled) => self.spawn_autostart_set(*enabled),
            }
        }
    }

    /// Flush strategy: every `PushLog` appends one JSON line and flushes
    /// immediately, so a crash loses nothing. The buffered writer is
    /// held across lines for the current day only; it is flushed and
    /// closed when `save_logs` is toggled off and on day rollover.
    /// Rotation (delete logs older than 7 days) runs once at startup.
    fn append_log_file(&mut self, level: LogLevel, source: &str, message: &str) {
        if !self.config.save_logs {
            self.close_log_file();
            return;
        }
        let Some(dir) = logs_dir() else {
            return;
        };
        if std::fs::create_dir_all(&dir).is_err() {
            return;
        }
        let today = chrono::Local::now().date_naive();
        let today_str = today.format("%Y-%m-%d").to_string();
        if self.log_file_day.as_deref() != Some(today_str.as_str()) {
            self.close_log_file();
            let path = log_file_for(&dir, today);
            let file = OpenOptions::new().create(true).append(true).open(path).ok();
            if let Some(file) = file {
                self.log_file = Some(BufWriter::new(file));
                self.log_file_day = Some(today_str.clone());
            }
        }
        if let Some(writer) = self.log_file.as_mut() {
            let line = serde_json::json!({
                "ts": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                "level": level.label(),
                "source": source,
                "message": message,
            });
            let _ = writeln!(writer, "{line}");
            let _ = writer.flush();
        }
    }

    fn close_log_file(&mut self) {
        if let Some(mut writer) = self.log_file.take() {
            let _ = writer.flush();
        }
        self.log_file_day = None;
    }

    fn exec_retry(&mut self, entry: HistoryEntry) {
        match retry_plan(&entry) {
            crate::services::traits::RetryPlan::Video { url } => {
                self.state.video.url = url.clone();
                self.state.video.status = VideoStatus::Downloading;
                self.state.video.progress = 0.0;
                self.state.video.speed = None;
                self.state.video.eta = None;
                self.state.video.error_key = None;
                self.state.screen = Screen::Video;
                let quality = self.state.video.quality;
                self.spawn_download(url, quality);
            }
            crate::services::traits::RetryPlan::Transcript { order } => {
                self.state.transcript.input = order.url.clone();
                self.state.transcript.lang = order.lang.clone();
                self.state.transcript.format = order.format;
                self.state.transcript.fallback =
                    order.fallback.clone().unwrap_or_else(|| "off".to_string());
                self.state.transcript.accept_auto = order.accept_auto;
                self.state.transcript.timestamps = order.timestamps;
                self.state.transcript.error = None;
                self.state.transcript.status = TranscriptStatus::Resolving;
                self.state.screen = Screen::Transcript;
                let mut adjusted = order;
                adjusted.output_dir = resolve_download_dir(&self.config);
                self.spawn_transcript(adjusted);
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
        let channel = self
            .state
            .video
            .metadata
            .as_ref()
            .map(|meta| meta.channel.clone());
        let output_dir = resolve_output_dir(&self.config, channel.as_deref());
        let order = DownloadOrder {
            url,
            quality,
            output_dir,
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

    /// Batch downloads run as ONE outer task over items, with at most
    /// `simultaneous` items in flight via a semaphore + `JoinSet`.
    /// Progress relays stay keyed by item index (`BatchTick{index,..}`),
    /// so completion order never matters for attribution. Dropping the
    /// outer task (cancel) drops the set, aborting children; each child
    /// owns a `kill_on_drop` yt-dlp process, so cancel kills downloads.
    /// `simultaneous == 1` serializes through the same path, matching
    /// the old sequential behavior.
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
        let bound = self.config.simultaneous.clamp(1, 5) as usize;
        let channel_name = if self.config.organize_by_channel {
            self.state
                .channel
                .preview
                .as_ref()
                .map(|preview| preview.name.clone())
        } else {
            None
        };
        let output_dir = resolve_output_dir(&self.config, channel_name.as_deref());
        let downloader = self.downloader.clone();
        let transcriber = self.transcriber.clone();
        let intent_tx = self.intent_tx.clone();
        self.track_channel(runtime.spawn(async move {
            let semaphore = Arc::new(Semaphore::new(bound));
            let mut set = JoinSet::new();
            for (index, item) in items.into_iter().enumerate() {
                let semaphore = semaphore.clone();
                let downloader = downloader.clone();
                let transcriber = transcriber.clone();
                let intent_tx = intent_tx.clone();
                let item_transcript = TranscriptOrder {
                    url: item.url.clone(),
                    output_dir: output_dir.clone(),
                    ..transcript.clone()
                };
                let item_dir = output_dir.clone();
                set.spawn(async move {
                    let Ok(_permit) = semaphore.acquire_owned().await else {
                        return;
                    };
                    let video_result = if include_video {
                        Some(
                            download_one(&downloader, &intent_tx, index, &item, quality, &item_dir)
                                .await,
                        )
                    } else {
                        None
                    };
                    let transcript_result = if include_transcript {
                        Some(transcribe_one(&transcriber, &item_transcript).await)
                    } else {
                        None
                    };
                    let _ = intent_tx.send(AppIntent::Channel(ChannelIntent::BatchItemFinished {
                        index,
                        video_result,
                        transcript_result,
                    }));
                });
            }
            while set.join_next().await.is_some() {}
        }));
    }

    /// `rfd::FileDialog::pick_folder` blocks, so it runs on a blocking
    /// thread and reports back as an intent; the UI thread never waits.
    fn spawn_folder_picker(&mut self) {
        let Some(runtime) = self.runtime.clone() else {
            return;
        };
        let intent_tx = self.intent_tx.clone();
        runtime.spawn_blocking(move || {
            let picked = rfd::FileDialog::new().pick_folder();
            let _ = intent_tx.send(AppIntent::Settings(SettingsIntent::DownloadDirPicked(
                picked,
            )));
        });
    }

    fn spawn_ytdlp_check(&mut self, report_updated: bool) {
        let Some(runtime) = self.runtime.clone() else {
            return;
        };
        let intent_tx = self.intent_tx.clone();
        runtime.spawn(async move {
            let result = ytdlp_version().await;
            let intent = if report_updated {
                SettingsIntent::YtDlpUpdated(result)
            } else {
                SettingsIntent::YtDlpVersionReceived(result)
            };
            let _ = intent_tx.send(AppIntent::Settings(intent));
        });
    }

    fn spawn_ytdlp_update(&mut self) {
        let Some(runtime) = self.runtime.clone() else {
            return;
        };
        let intent_tx = self.intent_tx.clone();
        let sink = self.log_sink.clone();
        runtime.spawn(async move {
            let result = update_binary(&*sink).await;
            let _ = intent_tx.send(AppIntent::Settings(SettingsIntent::YtDlpUpdated(result)));
        });
    }

    fn spawn_autostart_verify(&mut self) {
        #[cfg(windows)]
        {
            let Some(runtime) = self.runtime.clone() else {
                return;
            };
            let intent_tx = self.intent_tx.clone();
            runtime.spawn_blocking(move || {
                let actual = crate::services::autostart::is_enabled();
                let _ = intent_tx.send(AppIntent::Settings(SettingsIntent::AutostartVerified(
                    actual,
                )));
            });
        }
        #[cfg(not(windows))]
        {
            self.state.settings.autostart_actual = None;
        }
    }

    fn spawn_autostart_set(&mut self, enabled: bool) {
        #[cfg(windows)]
        {
            let Some(runtime) = self.runtime.clone() else {
                return;
            };
            let intent_tx = self.intent_tx.clone();
            let sink = self.log_sink.clone();
            runtime.spawn_blocking(move || {
                match crate::services::autostart::set_enabled(enabled) {
                    Ok(()) => {
                        let actual = crate::services::autostart::is_enabled();
                        let _ = intent_tx.send(AppIntent::Settings(
                            SettingsIntent::AutostartVerified(actual),
                        ));
                    }
                    Err(_) => {
                        sink.push(
                            LogLevel::Error,
                            "settings",
                            "failed to update autostart entry",
                        );
                        let _ = intent_tx
                            .send(AppIntent::Settings(SettingsIntent::AutostartVerified(None)));
                    }
                }
            });
        }
        #[cfg(not(windows))]
        {
            let _ = enabled;
            self.log_sink.push(
                LogLevel::Warn,
                "settings",
                "autostart not supported on this platform",
            );
        }
    }
}

async fn ytdlp_version() -> Result<String, String> {
    let binary = resolve_binary().map_err(|_| "settings_ytdlp_missing".to_string())?;
    let output = hidden_command(&binary)
        .arg("--version")
        .kill_on_drop(true)
        .output()
        .await
        .map_err(|_| "settings_ytdlp_error_check".to_string())?;
    if !output.status.success() {
        return Err("settings_ytdlp_error_check".to_string());
    }
    let version = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .to_string();
    if version.is_empty() {
        Err("settings_ytdlp_error_check".to_string())
    } else {
        Ok(version)
    }
}

async fn update_binary(sink: &dyn LogSink) -> Result<String, String> {
    let binary = resolve_binary().map_err(|_| "settings_ytdlp_missing".to_string())?;
    let output = hidden_command(&binary)
        .arg("-U")
        .kill_on_drop(true)
        .output()
        .await
        .map_err(|_| "settings_ytdlp_error_update".to_string())?;
    for line in String::from_utf8_lossy(&output.stderr).lines() {
        if !line.trim().is_empty() {
            sink.push(LogLevel::Debug, "yt-dlp", line.trim());
        }
    }
    if !output.status.success() {
        return Err("settings_ytdlp_error_update".to_string());
    }
    ytdlp_version()
        .await
        .map_err(|_| "settings_ytdlp_error_update".to_string())
}

async fn download_one(
    downloader: &Arc<dyn Downloader>,
    intent_tx: &UnboundedSender<AppIntent>,
    index: usize,
    item: &BatchItem,
    quality: VideoQuality,
    output_dir: &Path,
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
        output_dir: output_dir.to_path_buf(),
    };
    let result = downloader.download(order, progress_tx).await;
    relay.abort();
    result
}

async fn transcribe_one(
    transcriber: &Arc<dyn Transcriber>,
    prefs: &TranscriptOrder,
) -> Result<crate::services::traits::TranscriptResult, String> {
    transcriber.transcribe(prefs).await
}

pub fn prune_startup_logs() {
    let Some(dir) = logs_dir() else {
        return;
    };
    let today = chrono::Local::now().date_naive();
    prune_old_logs(&dir, today, 7);
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
        StubChannelProvider, StubDownloader, StubHistoryStore, StubMetadataProvider,
        StubTranscriber,
    };

    fn test_store() -> Store {
        Store::new_with_saver(
            Arc::new(LocaleService),
            Arc::new(BufferLogSink),
            Arc::new(StubMetadataProvider),
            Arc::new(StubDownloader),
            Arc::new(StubTranscriber),
            Arc::new(StubChannelProvider),
            Arc::new(StubHistoryStore::default()),
            AppConfig::default(),
            None,
            Arc::new(|_| {}),
        )
    }

    fn test_store_with(config: AppConfig) -> Store {
        Store::new_with_saver(
            Arc::new(LocaleService),
            Arc::new(BufferLogSink),
            Arc::new(StubMetadataProvider),
            Arc::new(StubDownloader),
            Arc::new(StubTranscriber),
            Arc::new(StubChannelProvider),
            Arc::new(StubHistoryStore::default()),
            config,
            None,
            Arc::new(|_| {}),
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
    fn settings_seed_initial_tab_defaults() {
        let config = AppConfig {
            quality_default: VideoQuality::Capped720,
            transcript_lang: "en".to_string(),
            transcript_fallback: None,
            ..Default::default()
        };
        let store = test_store_with(config);
        assert_eq!(store.state().video.quality, VideoQuality::Capped720);
        assert_eq!(store.state().channel.quality, VideoQuality::Capped720);
        assert_eq!(store.state().transcript.lang, "en");
        assert_eq!(store.state().transcript.fallback, "off");
        assert_eq!(store.state().settings.transcript_fallback, None);
    }

    #[test]
    fn settings_save_persists_through_reducer() {
        let mut store = test_store();
        store.dispatch(AppIntent::Settings(SettingsIntent::SetSimultaneous(5)));
        assert_eq!(store.state().settings.simultaneous, 5);
    }

    #[test]
    fn completion_pushes_notice_when_enabled() {
        use crate::services::traits::DownloadTicket;
        let mut store = test_store();
        assert!(store.state().notices.is_empty());
        store.dispatch(AppIntent::Video(VideoIntent::SetUrl(
            "https://example.com/v".to_string(),
        )));
        store.dispatch(AppIntent::Video(VideoIntent::DownloadFinished(Ok(
            DownloadTicket {
                id: "vid".to_string(),
                path: std::path::PathBuf::from("/tmp/vid.mp4"),
            },
        ))));
        assert_eq!(store.state().notices.len(), 1);
        let id = store.state().notices[0].id.clone();
        store.dispatch(AppIntent::DismissNotice(id));
        assert!(store.state().notices.is_empty());
    }

    #[test]
    fn completion_notice_suppressed_when_disabled() {
        use crate::services::traits::DownloadTicket;
        let config = AppConfig {
            notify_on_complete: false,
            ..Default::default()
        };
        let mut store = test_store_with(config);
        store.dispatch(AppIntent::Video(VideoIntent::SetUrl(
            "https://example.com/v".to_string(),
        )));
        store.dispatch(AppIntent::Video(VideoIntent::DownloadFinished(Ok(
            DownloadTicket {
                id: "vid".to_string(),
                path: std::path::PathBuf::from("/tmp/vid.mp4"),
            },
        ))));
        assert!(store.state().notices.is_empty());
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
        assert_eq!(store.history_count(), 1);
    }

    #[test]
    fn history_retry_missing_id_warns_without_navigation() {
        use crate::core::intent::HistoryIntent;
        let mut store = test_store();
        store.dispatch(AppIntent::History(HistoryIntent::RetryEntry(
            "missing-id".to_string(),
        )));
        assert_eq!(store.state().screen, Screen::Video);
        assert!(store.history_entries().is_empty());
    }

    #[test]
    fn history_clear_round_trip() {
        use crate::core::intent::HistoryIntent;
        use crate::services::traits::{EntryKind, EntryStatus, HistoryEntry, HistoryStore};
        let history = Arc::new(StubHistoryStore::default());
        history.record(HistoryEntry {
            id: "e1".to_string(),
            kind: EntryKind::Video,
            name: "Video".to_string(),
            source: "example.com".to_string(),
            url: "https://example.com/v".to_string(),
            detail: None,
            finished_at_ms: 1,
            size_bytes: None,
            status: EntryStatus::Completed,
            error: None,
            path: None,
            transcript_order: None,
        });
        let mut store = Store::new_with_saver(
            Arc::new(LocaleService),
            Arc::new(BufferLogSink),
            Arc::new(StubMetadataProvider),
            Arc::new(StubDownloader),
            Arc::new(StubTranscriber),
            Arc::new(StubChannelProvider),
            history,
            AppConfig::default(),
            None,
            Arc::new(|_| {}),
        );
        assert_eq!(store.history_count(), 1);
        store.dispatch(AppIntent::History(HistoryIntent::RequestClear));
        store.dispatch(AppIntent::History(HistoryIntent::ClearHistory));
        assert!(store.history_entries().is_empty());
    }

    async fn run_limited(bound: usize, delays_ms: Vec<u64>) -> Vec<(usize, &'static str)> {
        let semaphore = Arc::new(Semaphore::new(bound));
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<(usize, &'static str)>();
        let mut set = JoinSet::new();
        for (index, delay) in delays_ms.into_iter().enumerate() {
            let semaphore = semaphore.clone();
            let tx = tx.clone();
            set.spawn(async move {
                let Ok(_permit) = semaphore.acquire_owned().await else {
                    return;
                };
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                let _ = tx.send((index, "ok"));
            });
        }
        drop(tx);
        let mut out = Vec::new();
        while let Some(received) = rx.recv().await {
            out.push(received);
        }
        while set.join_next().await.is_some() {}
        out.sort_by_key(|(index, _)| *index);
        out
    }

    #[tokio::test]
    async fn limited_batch_lands_on_correct_indices() {
        let out = run_limited(3, vec![60, 10, 30, 5, 40]).await;
        let indices: Vec<usize> = out.iter().map(|(index, _)| *index).collect();
        assert_eq!(indices, vec![0, 1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn bound_one_serializes_in_spawn_order() {
        let out = run_limited(1, vec![30, 10, 20]).await;
        let indices: Vec<usize> = out.iter().map(|(index, _)| *index).collect();
        assert_eq!(indices, vec![0, 1, 2]);
    }
}
