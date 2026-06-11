use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

use crate::{
    api::{
        provider::item_info::ItemInfoProvider,
        types::{BaseGroupId, BaseItemId, EssenceId, THashSet},
    },
    utils::hash_utils::hash_set_unordered,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "python",
    pyo3_stub_gen::derive::gen_stub_pyclass_complex_enum
)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(
    feature = "python",
    pyo3(eq, weakref, from_py_object, frozen, hash, get_all, str)
)]
pub enum CraftCurrencyEnum {
    // CURRENCIES
    OrbOfTransmutationNormal(), // cauz of complex enum type () is needed for python rep
    OrbOfTransmutationGreater(),
    OrbOfTransmutationPerfect(),
    OrbOfAugmentationNormal(),
    OrbOfAugmentationGreater(),
    OrbOfAugmentationPerfect(),
    RegalOrbNormal(),
    RegalOrbGreater(),
    RegalOrbPerfect(),
    ExaltedOrbNormal(),
    ExaltedOrbGreater(),
    ExaltedOrbPerfect(),
    OrbOfAnnulment(),
    ChaosOrbNormal(),
    ChaosOrbGreater(),
    ChaosOrbPerfect(),
    ArtificersOrb(),
    VaalOrb(),
    OmenOfCorruption(),

    FracturingOrb(),

    // DESECRATION
    Desecrator(BaseItemId, BaseGroupId),
    // CAN REROLL ONCE
    AbyssalEchoes(),
    // KURGAL MOD
    TheBlackblooded(),
    // ULAMAN MOD
    TheSovereign(),
    // AMANAMU MOD
    TheLiege(),
    // FORCE SUFFIX
    DextralNecromancy(),
    // FORCE PREFIX
    SinistralNecromancy(),

    // OMENS
    // REGAL ORB, EX ORB
    HomogenisingCoronation(),
    HomogenisingExaltation(),
    // EX ORB ONLY SUFFIX
    DextralExaltation(),
    // EX ORB ONLY PREFIX
    SinistralExaltation(),

    // Annuli ONLY SUFFIX
    DextralAnnulment(),
    // Annuli ONLY PREFIX
    SinistralAnnulment(),

    // CHAOS SUFFIX REMOVE
    DextralErasure(),
    // CHAOS PREFIX REMOVE
    SinistralErasure(),
    // CHAOS REMOVE LOWEST ILVL AFFIX
    Whittling(),

    Essence(EssenceId),
    DextralCrystallisation(),
    SinistralCrystallisation(),
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[cfg_attr(feature = "python", pyo3::pymethods)]
impl CraftCurrencyEnum {
    #[pyo3(name = "get_item_name")]
    fn get_item_name_py<'a>(&self, item_info: &'a ItemInfoProvider) -> String {
        self.get_item_name(&item_info).to_string()
    }
}

impl CraftCurrencyEnum {
    pub fn get_item_name<'a>(&self, item_info: &'a ItemInfoProvider) -> &'a str {
        match self {
            CraftCurrencyEnum::FracturingOrb() => "Fracturing Orb",
            CraftCurrencyEnum::ArtificersOrb() => "Artificer's Orb",
            CraftCurrencyEnum::VaalOrb() => "Vaal Orb",
            CraftCurrencyEnum::OmenOfCorruption() => "Omen of Corruption",
            CraftCurrencyEnum::AbyssalEchoes() => "Omen of Abyssal Echoes",
            CraftCurrencyEnum::ChaosOrbGreater() => "Greater Chaos Orb",
            CraftCurrencyEnum::ChaosOrbNormal() => "Chaos Orb",
            CraftCurrencyEnum::ChaosOrbPerfect() => "Perfect Chaos Orb",
            CraftCurrencyEnum::DextralAnnulment() => "Omen of Dextral Annulment",
            CraftCurrencyEnum::DextralCrystallisation() => "Omen of Dextral Crystallisation",
            CraftCurrencyEnum::DextralErasure() => "Omen of Dextral Erasure",
            CraftCurrencyEnum::DextralExaltation() => "Omen of Dextral Exaltation",
            CraftCurrencyEnum::DextralNecromancy() => "Omen of Dextral Necromancy",
            CraftCurrencyEnum::ExaltedOrbGreater() => "Greater Exalted Orb",
            CraftCurrencyEnum::ExaltedOrbNormal() => "Exalted Orb",
            CraftCurrencyEnum::ExaltedOrbPerfect() => "Perfect Exalted Orb",
            CraftCurrencyEnum::HomogenisingCoronation() => "Omen of Homogenising Coronation",
            CraftCurrencyEnum::HomogenisingExaltation() => "Omen of Homogenising Exaltation",
            CraftCurrencyEnum::OrbOfAnnulment() => "Orb of Annulment",
            CraftCurrencyEnum::OrbOfAugmentationGreater() => "Greater Orb of Augmentation",
            CraftCurrencyEnum::OrbOfAugmentationNormal() => "Orb of Augmentation",
            CraftCurrencyEnum::OrbOfAugmentationPerfect() => "Perfect Orb of Augmentation",
            CraftCurrencyEnum::OrbOfTransmutationGreater() => "Greater Orb of Transmutation",
            CraftCurrencyEnum::OrbOfTransmutationNormal() => "Orb of Transmutation",
            CraftCurrencyEnum::OrbOfTransmutationPerfect() => "Perfect Orb of Transmutation",
            CraftCurrencyEnum::RegalOrbGreater() => "Greater Regal Orb",
            CraftCurrencyEnum::RegalOrbNormal() => "Regal Orb",
            CraftCurrencyEnum::RegalOrbPerfect() => "Perfect Regal Orb",
            CraftCurrencyEnum::SinistralAnnulment() => "Omen of Sinistral Annulment",
            CraftCurrencyEnum::SinistralCrystallisation() => "Omen of Sinistral Crystallisation",
            CraftCurrencyEnum::SinistralErasure() => "Omen of Sinistral Erasure",
            CraftCurrencyEnum::SinistralExaltation() => "Omen of Sinistral Exaltation",
            CraftCurrencyEnum::SinistralNecromancy() => "Omen of Sinistral Necromancy",
            CraftCurrencyEnum::TheBlackblooded() => "Omen of the Blackblooded",
            CraftCurrencyEnum::TheLiege() => "Omen of the Liege",
            CraftCurrencyEnum::TheSovereign() => "Omen of the Sovereign",
            CraftCurrencyEnum::Whittling() => "Omen of Whittling",
            CraftCurrencyEnum::Desecrator(base_item_id, base_group_id) => {
                // Thanks for the info in https://www.craftofexile.com/js/poe2.js !
                // My alg. doesnt handle loose propagation, so Ancient Bones are not needed for now
                match base_item_id.get_raw_value() {
                    // Jawbones
                    4 | 11 | 12 | 13 | 15 | 16 | 17 | 18 | 20 | 21 | 22 | 23 | 24 | 25 | 216
                    | 217 | 218 | 219 | 220 | 221 | 222 | 223 | 224 | 225 | 226 | 227 | 228 => {
                        "Preserved Jawbone"
                    }

                    // Ribs
                    5 | 6 | 8 | 9 | 33 | 34 | 35 | 36 | 37 | 38 | 39 | 40 | 41 | 42 | 43 | 44
                    | 45 | 46 | 47 | 48 | 49 | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 | 229
                    | 230 | 231 | 232 => "Preserved Rib",

                    // Fallback for base items that are not jawbones or ribs
                    _ => match base_group_id.get_raw_value() {
                        1 => "Preserved Collarbone",
                        9 => "Preserved Cranium",
                        11 => "Preserved Spine",
                        _ => todo!("Unhandled bone type"),
                    },
                }
            }

            CraftCurrencyEnum::Essence(bid) => item_info
                .cache_essence_def
                .get(bid)
                .unwrap()
                .name_essence
                .as_str(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(
    feature = "python",
    pyo3(weakref, frozen, eq, hash, from_py_object, get_all, str)
)]
pub struct CraftCurrencyList {
    pub list: THashSet<CraftCurrencyEnum>,
}

impl Hash for CraftCurrencyList {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let currency_list_hash = hash_set_unordered(&self.list);
        currency_list_hash.hash(state);
    }
}

#[cfg(feature = "python")]
crate::derive_DebugDisplay!(CraftCurrencyList, CraftCurrencyEnum);
