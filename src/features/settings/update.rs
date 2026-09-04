use crate::core::effect::Effect;

use super::intent::SettingsIntent;
use super::model::SettingsModel;

pub fn apply(_model: &mut SettingsModel, intent: &SettingsIntent) -> Vec<Effect> {
    match intent {
        SettingsIntent::ChangeLocale(locale) => vec![Effect::ReloadLocale(locale.clone())],
    }
}
