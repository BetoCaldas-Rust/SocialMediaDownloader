use std::sync::Arc;

use super::effect::Effect;
use super::intent::AppIntent;
use super::state::AppState;
use crate::features::{channel, console, history, settings, transcript, video};
use crate::services::log_buffer::LogLevel;
use crate::services::traits::{LocaleProvider, LogSink};

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
        AppIntent::Video(inner) => video::update::apply(&mut state.video, inner),
        AppIntent::Transcript(inner) => transcript::update::apply(&mut state.transcript, inner),
        AppIntent::Channel(inner) => channel::update::apply(&mut state.channel, inner),
        AppIntent::History(inner) => history::update::apply(&mut state.history, inner),
        AppIntent::Console(inner) => console::update::apply(&mut state.console, inner),
        AppIntent::Settings(inner) => settings::update::apply(&mut state.settings, inner),
    }
}

pub struct Store {
    state: AppState,
    locale_provider: Arc<dyn LocaleProvider>,
    log_sink: Arc<dyn LogSink>,
}

impl Store {
    pub fn new(locale_provider: Arc<dyn LocaleProvider>, log_sink: Arc<dyn LogSink>) -> Self {
        let state = AppState {
            locale: locale_provider.current(),
            ..Default::default()
        };
        Self {
            state,
            locale_provider,
            log_sink,
        }
    }

    pub fn state(&self) -> &AppState {
        &self.state
    }

    pub fn available_locales(&self) -> Vec<String> {
        self.locale_provider.available()
    }

    pub fn dispatch(&mut self, intent: AppIntent) {
        let effects = update(&mut self.state, &intent);
        self.execute(&effects);
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
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::state::Screen;
    use super::*;
    use crate::services::locale::LocaleService;
    use crate::services::log_sink::BufferLogSink;

    fn test_store() -> Store {
        Store::new(Arc::new(LocaleService), Arc::new(BufferLogSink))
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
}
