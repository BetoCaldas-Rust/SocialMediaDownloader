use crate::core::effect::Effect;

use super::intent::HistoryIntent;
use super::model::HistoryModel;

pub fn apply(model: &mut HistoryModel, intent: &HistoryIntent) -> Vec<Effect> {
    match intent {
        HistoryIntent::SetFilter(filter) => {
            model.filter = filter.clone();
            Vec::new()
        }
    }
}
