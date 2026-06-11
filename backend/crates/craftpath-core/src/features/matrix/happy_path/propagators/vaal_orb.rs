//! Vaal Orb corruption (game patch 0.5.0 model, MECHANICS.md V1-V3).
//!
//! Four uniform outcome branches: add socket / add corruption implicit /
//! partial mod reroll / no change. Exactly one branch fires per corruption
//! (V3), so a target may ask for either +1 socket or one corrupted implicit —
//! never both. On non-socketable bases (jewellery) the socket branch acts as
//! "no change" but the branch count stays 4.
//!
//! Omen of Corruption removes the "no change" branch (1/3 each). NOTE: the
//! omen is unobtainable in the 0.5.0 league (legacy items only) — routes
//! using it are still enumerated for Standard-league legacy stock.

use anyhow::Result;

use crate::{
    api::{
        calculator::PropagationTarget,
        calculator_utils::calculate_target_proximity::calculate_target_proximity,
        currency::{CraftCurrencyEnum, CraftCurrencyList},
        item::{Item, ItemSnapshot},
        matrix_propagator::MatrixPropagator,
        provider::item_info::ItemInfoProvider,
        types::{
            AffixLocationEnum, AffixSpecifier, AffixTierLevelBoundsEnum, BaseGroupId, THashMap,
            THashSet,
        },
    },
    utils::fraction_utils::Fraction,
};

static VAAL_ORB: &CraftCurrencyEnum = &CraftCurrencyEnum::VaalOrb();

static CORRUPTION_OMEN: &[Option<CraftCurrencyEnum>] =
    &[Some(CraftCurrencyEnum::OmenOfCorruption()), None];

pub struct VaalOrbPropagator;

enum Category {
    SocketableEquipment,
    Ignore,
}

fn classify(id_bgroup: &BaseGroupId) -> Category {
    match id_bgroup.get_raw_value() {
        // Equipment
        2  // Body Armours
        | 3  // Boots
        | 5  // Gloves
        | 4  // Helmets
        | 8  // Offhands
        | 6  // One-Handed Weapons
        | 7  // Two-Handed Weapons
            => Category::SocketableEquipment,
        // Everything else
        _ => Category::Ignore,
    }
}

enum VaalBranch {
    /// +1 socket (ignoring socket limits), weapons/armour only.
    AddSocket,
    /// Add one corruption implicit, weight-drawn from the base's pool.
    AddImplicit(AffixSpecifier),
}

impl VaalOrbPropagator {
    /// Which target-approaching branch (if any) this corruption can take.
    fn wanted_branch(
        item_instance: &Item,
        target: &ItemSnapshot,
        provider: &ItemInfoProvider,
    ) -> Result<Option<VaalBranch>> {
        let snapshot = &item_instance.snapshot;

        let wants_socket = target.allowed_sockets > snapshot.allowed_sockets;

        let missing_implicits: Vec<&AffixSpecifier> = target
            .affixes
            .iter()
            .filter(|t| {
                provider
                    .lookup_affix_definition(&t.affix)
                    .map(|def| def.affix_location == AffixLocationEnum::Corrupted)
                    .unwrap_or(false)
                    && !snapshot.affixes.iter().any(|have| have.affix == t.affix)
            })
            .collect();

        if wants_socket && !missing_implicits.is_empty() {
            tracing::warn!(
                "Target wants both an extra socket and a corruption implicit — a single Vaal Orb cannot grant both (MECHANICS.md V3); no Vaal route emitted."
            );
            return Ok(None);
        }
        if missing_implicits.len() > 1 {
            tracing::warn!(
                "Target wants {} corruption implicits — one corruption adds exactly one (MECHANICS.md V2); no Vaal route emitted.",
                missing_implicits.len()
            );
            return Ok(None);
        }

        if wants_socket {
            let base_group_id = provider.lookup_base_group(&snapshot.base_id)?;
            if !matches!(classify(&base_group_id), Category::SocketableEquipment) {
                // on jewellery the socket branch is "no change"
                return Ok(None);
            }
            if target.allowed_sockets - snapshot.allowed_sockets != 1 {
                return Ok(None); // one corruption adds exactly one socket
            }
            return Ok(Some(VaalBranch::AddSocket));
        }

        if let Some(implicit) = missing_implicits.first() {
            return Ok(Some(VaalBranch::AddImplicit((*implicit).clone())));
        }

        Ok(None)
    }

    /// Chance that the implicit branch yields the wanted implicit:
    /// `w_acceptable / W_pool` over the base's corrupted-mod weights
    /// (uniform in practice — all corrupted tier weights are 1, V2).
    fn implicit_pick_chance(
        wanted: &AffixSpecifier,
        snapshot: &ItemSnapshot,
        provider: &ItemInfoProvider,
    ) -> Result<Option<Fraction>> {
        let pool = provider.lookup_base_item_mods(&snapshot.base_id)?;

        let mut total_weight: u32 = 0;
        let mut acceptable_weight: u32 = 0;

        for (affix_id, tiers) in pool.iter() {
            let Ok(def) = provider.lookup_affix_definition(affix_id) else {
                continue;
            };
            if def.affix_location != AffixLocationEnum::Corrupted {
                continue;
            }

            let full: u32 = tiers
                .iter()
                .fold(0u32, |a, b| a + b.1.weight.get_raw_value().clone());
            total_weight += full;

            if affix_id == &wanted.affix {
                acceptable_weight = match wanted.tier.bounds {
                    AffixTierLevelBoundsEnum::Minimum => tiers
                        .iter()
                        .filter(|(tier, _)| **tier <= wanted.tier.tier)
                        .fold(0u32, |a, b| a + b.1.weight.get_raw_value().clone()),
                    AffixTierLevelBoundsEnum::Exact => tiers
                        .get(&wanted.tier.tier)
                        .map(|meta| meta.weight.get_raw_value().clone())
                        .unwrap_or(0),
                };
            }
        }

        if total_weight == 0 || acceptable_weight == 0 {
            return Ok(None);
        }

        Ok(Some(Fraction::new(acceptable_weight, total_weight)))
    }
}

impl MatrixPropagator for VaalOrbPropagator {
    fn propagate_step(
        &self,
        item_instance: &Item,
        target: &ItemSnapshot,
        provider: &ItemInfoProvider,
    ) -> Result<THashMap<CraftCurrencyList, Vec<PropagationTarget>>> {
        let mut propagation_result: THashMap<CraftCurrencyList, Vec<PropagationTarget>> =
            THashMap::default();

        // only apply the orb as the very last step — corruption is terminal
        if calculate_target_proximity(&item_instance.snapshot, &target, &provider)? != 1 {
            return Ok(propagation_result);
        }

        let Some(branch) = Self::wanted_branch(item_instance, target, provider)? else {
            return Ok(propagation_result);
        };

        // probability that the wanted branch ALSO yields the wanted result
        let (next_item_snapshot, branch_success) = match &branch {
            VaalBranch::AddSocket => (
                ItemSnapshot {
                    rarity: item_instance.snapshot.rarity.clone(),
                    base_id: item_instance.snapshot.base_id.clone(),
                    item_level: item_instance.snapshot.item_level.clone(),
                    affixes: item_instance.snapshot.affixes.clone(),
                    allowed_sockets: item_instance.snapshot.allowed_sockets + 1,
                    corrupted: true,
                    sockets: item_instance.snapshot.sockets.clone(),
                },
                Fraction::one(),
            ),
            VaalBranch::AddImplicit(wanted) => {
                let Some(pick_chance) =
                    Self::implicit_pick_chance(wanted, &item_instance.snapshot, provider)?
                else {
                    return Ok(propagation_result);
                };

                let mut affixes = item_instance.snapshot.affixes.clone();
                affixes.insert(wanted.clone());

                (
                    ItemSnapshot {
                        rarity: item_instance.snapshot.rarity.clone(),
                        base_id: item_instance.snapshot.base_id.clone(),
                        item_level: item_instance.snapshot.item_level.clone(),
                        affixes,
                        allowed_sockets: item_instance.snapshot.allowed_sockets,
                        corrupted: true,
                        sockets: item_instance.snapshot.sockets.clone(),
                    },
                    pick_chance,
                )
            }
        };

        for corruption_omen in CORRUPTION_OMEN {
            // 4 uniform outcome branches; the omen removes "no change" -> 3
            // (MECHANICS.md V1)
            let branch_chance = match corruption_omen {
                Some(_) => Fraction::new(1, 3),
                None => Fraction::new(1, 4),
            };

            let chance = branch_chance * branch_success.clone();

            let mut unique_currency_list = CraftCurrencyList {
                list: THashSet::default(),
            };
            unique_currency_list.list.insert(VAAL_ORB.clone());
            if let Some(vaal_omen) = corruption_omen {
                unique_currency_list.list.insert(vaal_omen.clone());
            }

            propagation_result.insert(
                unique_currency_list,
                vec![PropagationTarget::new(chance, next_item_snapshot.clone())],
            );
        }

        Ok(propagation_result)
    }

    fn is_applicable(&self, item: &Item, provider: &ItemInfoProvider) -> bool {
        if item.snapshot.corrupted {
            return false;
        }

        let Ok(base_group_id) = provider.lookup_base_group(&item.snapshot.base_id) else {
            return false;
        };
        let Ok(base_group_def) = provider.lookup_base_group_definition(&base_group_id) else {
            return false;
        };

        // socket progress possible on socketable equipment ...
        let socket_possible = matches!(classify(&base_group_id), Category::SocketableEquipment)
            && base_group_def.max_sockets >= item.snapshot.allowed_sockets;

        // ... or the base has corruption implicits to gain
        let implicit_possible = provider
            .lookup_base_item_mods(&item.snapshot.base_id)
            .map(|pool| {
                pool.keys().any(|affix_id| {
                    provider
                        .lookup_affix_definition(affix_id)
                        .map(|def| def.affix_location == AffixLocationEnum::Corrupted)
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);

        socket_possible || implicit_possible
    }
}
