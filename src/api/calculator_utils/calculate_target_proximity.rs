use anyhow::Result;

use crate::api::{
    item::ItemSnapshot, provider::item_info::ItemInfoProvider, types::AffixTierLevelBoundsEnum,
};

pub fn calculate_target_proximity(
    start: &ItemSnapshot,
    target: &ItemSnapshot,
    provider: &ItemInfoProvider, // maybe will be needed sometime in the future idk
) -> Result<u8> {
    Ok(
        calculate_target_proximity_affixes(&start, &target, &provider)?
            + calculate_target_proximity_socket_amount(&start, &target, &provider)?,
    )
}

pub fn calculate_target_proximity_affixes(
    start: &ItemSnapshot,
    target: &ItemSnapshot,
    _provider: &ItemInfoProvider, // maybe will be needed sometime in the future idk
) -> Result<u8> {
    let mut unwanted_affix_counter = 0_u8;

    for specifier in &start.affixes {
        // Determine if this affix is unwanted
        let unwanted = match target.affixes.iter().find(|t| t.affix == specifier.affix) {
            Some(t) => match t.tier.bounds {
                AffixTierLevelBoundsEnum::Exact if t.tier.tier != specifier.tier.tier => true,
                AffixTierLevelBoundsEnum::Minimum if t.tier.tier < specifier.tier.tier => true,
                _ => false,
            },
            None => true,
        };

        if unwanted {
            unwanted_affix_counter += 1;
        }
    }
    // unwanted_affix_counter reflects how many affixes from the *starting* item are unwanted.
    unwanted_affix_counter += (target.affixes.len() as u8).abs_diff(start.affixes.len() as u8);

    Ok(unwanted_affix_counter)
}

pub fn calculate_target_proximity_socket_amount(
    start: &ItemSnapshot,
    target: &ItemSnapshot,
    _provider: &ItemInfoProvider,
) -> Result<u8> {
    Ok(target.allowed_sockets.abs_diff(start.allowed_sockets))
}
