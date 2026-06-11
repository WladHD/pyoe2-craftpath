use thiserror::Error;

use crate::api::{
    item::Item,
    types::{
        AffixId, AffixLocationEnum, AffixSpecifier, BaseGroupId, BaseItemId, EssenceId, ItemLevel,
    },
};

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
    #[error("Could not find definition for '{0:?}'.")]
    BaseGroupWithoutDefinition(BaseGroupId),
    #[error("Could not find essence definition for '{0:?}'.")]
    EssenceWithoutDefinition(EssenceId),
    #[error("Base item '{0:?}' without base group.")]
    BaseItemWithoutBaseGroup(BaseItemId),
    #[error(
        "The target item could not be reached from the given starting item. If you think that it is a bug, open an issue at https://github.com/WladHD/pyoe2-craftpath/issues"
    )]
    ItemMatrixCouldNotReachTarget(),
    #[error(
        "Could not reach required affix due to level constraints. Minimal item level is '{0:?}' (current {1:?}) for required affix '{2:?}' ..."
    )]
    ItemUnreachableMinLevelConstraint(ItemLevel, ItemLevel, AffixId),
    #[error("Affix '{1:?}' is unreachable with the item configuration provided in {0:?}.")]
    ItemUnreachable(Item, AffixSpecifier),
    #[error("Perfect Essence requires intermediary step to be applied.")]
    EssenceIntermediaryStepRequired(AffixLocationEnum),
    #[error("Defined RAM limit of '{0}' was reached and program was aborted.")]
    RamLimitReached(String),
}
