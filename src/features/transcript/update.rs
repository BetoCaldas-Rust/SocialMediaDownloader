use crate::core::effect::Effect;

use super::intent::TranscriptIntent;
use super::model::TranscriptModel;

pub fn apply(model: &mut TranscriptModel, intent: &TranscriptIntent) -> Vec<Effect> {
    match intent {
        TranscriptIntent::SetInput(input) => {
            model.input = input.clone();
            Vec::new()
        }
    }
}
