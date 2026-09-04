use crate::core::effect::Effect;

use super::intent::ChannelIntent;
use super::model::ChannelModel;

pub fn apply(model: &mut ChannelModel, intent: &ChannelIntent) -> Vec<Effect> {
    match intent {
        ChannelIntent::SetInput(input) => {
            model.input = input.clone();
            Vec::new()
        }
    }
}
