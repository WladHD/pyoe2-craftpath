use thiserror::Error;

use crate::api::types::{AffixId, BaseItemId, EssenceId};

#[derive(Debug, Error)]
pub enum CraftPathError {
    #[error(
        "Could not find affixes that can be put on base item '{0:?}'. Item info provider correct?"
    )]
    ItemWithoutAffixInformation(BaseItemId),
    #[error("Could not find affix definition for '{0:?}'.")]
    AffixWithoutDefinition(AffixId),
    #[error("Could not find affix essence for '{0:?}'.")]
    AffixWithoutEssence(AffixId),
    #[error("Could not find essence definition for '{0:?}'.")]
    EssenceWithoutDefinition(EssenceId),
    #[error(
        "The target item could not be reached from the given starting item. If you think that it is a bug, open an issue at https://github.com/WladHD/pyoe2-craftpath/issues"
    )]
    ItemMatrixCouldNotReachTarget(),
}
