use thiserror::Error;

use crate::api::types::BaseItemId;

#[derive(Debug, Error)]
pub enum CraftPathError {
    #[error("Could not find affixes that can be put on {0:?}. Item info provider correct?")]
    ItemWithoutAffixInformation(BaseItemId),
}
