use crate::core::effect::Effect;

use super::intent::VideoIntent;
use super::model::VideoModel;

pub fn apply(model: &mut VideoModel, intent: &VideoIntent) -> Vec<Effect> {
    match intent {
        VideoIntent::SetUrl(url) => {
            model.url = url.clone();
            Vec::new()
        }
    }
}
