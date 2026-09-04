use super::traits::LocaleProvider;

pub struct LocaleService;

impl LocaleProvider for LocaleService {
    fn available(&self) -> Vec<String> {
        crate::i18n::registry::available_locales()
    }

    fn current(&self) -> String {
        crate::i18n::registry::current_locale()
    }

    fn set(&self, locale: &str) -> bool {
        crate::i18n::registry::set_locale(locale)
    }

    fn translate(&self, key: &str) -> String {
        crate::i18n::registry::t(key)
    }
}
