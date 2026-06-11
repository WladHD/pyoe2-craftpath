use craftpath_core::api::currency::CraftCurrencyEnum;
use craftpath_core::api::item::ItemSnapshot;
use craftpath_core::api::types::{
    AffixId, AffixSpecifier, AffixTierConstraints, AffixTierLevel, AffixTierLevelBoundsEnum,
    BaseGroupId, BaseItemId, EssenceId, ItemLevel, ItemRarityEnum, THashSet,
};
use craftpath_core::calc::matrix::presets::matrix_builder_presets::MatrixBuilderPreset;
use craftpath_core::calc::statistics::presets::statistic_analyzer_currency_group_presets::StatisticAnalyzerCurrencyGroupPreset;
use craftpath_core::calc::statistics::presets::statistic_analyzer_path_presets::StatisticAnalyzerPathPreset;
use craftpath_proto::convert::craft_currency_to_proto;
use craftpath_proto::v1;
use prost::Message;

fn sample_snapshot() -> ItemSnapshot {
    let mut affixes: THashSet<AffixSpecifier> = THashSet::default();
    affixes.insert(AffixSpecifier {
        affix: AffixId::from(5119),
        fractured: true,
        tier: AffixTierConstraints {
            tier: AffixTierLevel::from(3),
            bounds: AffixTierLevelBoundsEnum::Minimum,
        },
    });
    affixes.insert(AffixSpecifier {
        affix: AffixId::from(42),
        fractured: false,
        tier: AffixTierConstraints {
            tier: AffixTierLevel::from(1),
            bounds: AffixTierLevelBoundsEnum::Exact,
        },
    });

    ItemSnapshot {
        item_level: ItemLevel::from(81),
        rarity: ItemRarityEnum::Rare,
        base_id: BaseItemId::from(20),
        affixes,
        corrupted: false,
        allowed_sockets: 2,
        sockets: THashSet::default(),
    }
}

#[test]
fn test_item_snapshot_roundtrip_through_proto_binary_and_json() {
    let original = sample_snapshot();

    let proto = v1::ItemSnapshot::from(&original);

    // binary round-trip
    let bytes = proto.encode_to_vec();
    let decoded = v1::ItemSnapshot::decode(bytes.as_slice()).unwrap();
    assert_eq!(proto, decoded);

    // canonical JSON round-trip (pbjson serde impls)
    let json = serde_json::to_string(&proto).unwrap();
    let from_json: v1::ItemSnapshot = serde_json::from_str(&json).unwrap();
    assert_eq!(proto, from_json);

    // back to the domain type
    let roundtripped = ItemSnapshot::try_from(&decoded).unwrap();
    assert_eq!(original, roundtripped);
}

#[test]
fn test_item_snapshot_json_uses_canonical_field_names() {
    let json = serde_json::to_value(v1::ItemSnapshot::from(&sample_snapshot())).unwrap();
    assert!(json.get("itemLevel").is_some(), "expected camelCase: {json}");
    assert!(json.get("baseId").is_some());
}

#[test]
fn test_item_snapshot_rejects_out_of_range_values() {
    let mut proto = v1::ItemSnapshot::from(&sample_snapshot());
    proto.item_level = 300; // > u8::MAX
    let err = ItemSnapshot::try_from(&proto).unwrap_err();
    assert!(err.to_string().contains("item_level"), "got: {err}");

    let mut proto = v1::ItemSnapshot::from(&sample_snapshot());
    proto.rarity = v1::ItemRarity::Unspecified as i32;
    assert!(ItemSnapshot::try_from(&proto).is_err());
}

#[test]
fn test_all_craft_currency_variants_roundtrip() {
    let variants = vec![
        CraftCurrencyEnum::OrbOfTransmutationNormal(),
        CraftCurrencyEnum::OrbOfTransmutationGreater(),
        CraftCurrencyEnum::OrbOfTransmutationPerfect(),
        CraftCurrencyEnum::OrbOfAugmentationNormal(),
        CraftCurrencyEnum::OrbOfAugmentationGreater(),
        CraftCurrencyEnum::OrbOfAugmentationPerfect(),
        CraftCurrencyEnum::RegalOrbNormal(),
        CraftCurrencyEnum::RegalOrbGreater(),
        CraftCurrencyEnum::RegalOrbPerfect(),
        CraftCurrencyEnum::ExaltedOrbNormal(),
        CraftCurrencyEnum::ExaltedOrbGreater(),
        CraftCurrencyEnum::ExaltedOrbPerfect(),
        CraftCurrencyEnum::OrbOfAnnulment(),
        CraftCurrencyEnum::ChaosOrbNormal(),
        CraftCurrencyEnum::ChaosOrbGreater(),
        CraftCurrencyEnum::ChaosOrbPerfect(),
        CraftCurrencyEnum::ArtificersOrb(),
        CraftCurrencyEnum::VaalOrb(),
        CraftCurrencyEnum::OmenOfCorruption(),
        CraftCurrencyEnum::FracturingOrb(),
        CraftCurrencyEnum::Desecrator(BaseItemId::from(7), BaseGroupId::from(9)),
        CraftCurrencyEnum::AbyssalEchoes(),
        CraftCurrencyEnum::TheBlackblooded(),
        CraftCurrencyEnum::TheSovereign(),
        CraftCurrencyEnum::TheLiege(),
        CraftCurrencyEnum::DextralNecromancy(),
        CraftCurrencyEnum::SinistralNecromancy(),
        CraftCurrencyEnum::HomogenisingCoronation(),
        CraftCurrencyEnum::HomogenisingExaltation(),
        CraftCurrencyEnum::DextralExaltation(),
        CraftCurrencyEnum::SinistralExaltation(),
        CraftCurrencyEnum::DextralAnnulment(),
        CraftCurrencyEnum::SinistralAnnulment(),
        CraftCurrencyEnum::DextralErasure(),
        CraftCurrencyEnum::SinistralErasure(),
        CraftCurrencyEnum::Whittling(),
        CraftCurrencyEnum::Essence(EssenceId::from(11)),
        CraftCurrencyEnum::DextralCrystallisation(),
        CraftCurrencyEnum::SinistralCrystallisation(),
        CraftCurrencyEnum::OmenOfGreaterExaltation(),
        CraftCurrencyEnum::OmenOfLight(),
    ];

    for original in variants {
        let proto = craft_currency_to_proto(&original, None);
        let roundtripped = CraftCurrencyEnum::try_from(&proto)
            .unwrap_or_else(|e| panic!("failed for {original:?}: {e}"));
        assert_eq!(original, roundtripped);
    }
}

#[test]
fn test_currency_payload_validation() {
    // DESECRATOR without payload must be rejected
    let proto = v1::CraftCurrency {
        kind: v1::CraftCurrencyKind::Desecrator as i32,
        desecrator: None,
        essence_id: None,
        display_name: String::new(),
    };
    assert!(CraftCurrencyEnum::try_from(&proto).is_err());

    // ESSENCE without payload must be rejected
    let proto = v1::CraftCurrency {
        kind: v1::CraftCurrencyKind::Essence as i32,
        desecrator: None,
        essence_id: None,
        display_name: String::new(),
    };
    assert!(CraftCurrencyEnum::try_from(&proto).is_err());
}

#[test]
fn test_preset_enums_roundtrip() {
    for p in [
        StatisticAnalyzerPathPreset::UniquePathChance,
        StatisticAnalyzerPathPreset::UniquePathEfficiency,
        StatisticAnalyzerPathPreset::UniquePathCost,
        StatisticAnalyzerPathPreset::UniquePathChanceMemoryHeavy,
    ] {
        let proto = v1::StatisticAnalyzerPathPreset::from(&p);
        assert_eq!(StatisticAnalyzerPathPreset::try_from(proto).unwrap(), p);
    }

    for p in [
        StatisticAnalyzerCurrencyGroupPreset::CurrencyGroupChance,
        StatisticAnalyzerCurrencyGroupPreset::CurrencyGroupChanceMemoryHeavy,
    ] {
        let proto = v1::StatisticAnalyzerCurrencyGroupPreset::from(&p);
        assert_eq!(
            StatisticAnalyzerCurrencyGroupPreset::try_from(proto).unwrap(),
            p
        );
    }

    let proto = v1::MatrixBuilderPreset::from(&MatrixBuilderPreset::HappyPathMatrixBuilder);
    assert_eq!(
        MatrixBuilderPreset::try_from(proto).unwrap(),
        MatrixBuilderPreset::HappyPathMatrixBuilder
    );
}

#[test]
fn test_submit_job_request_json_shape() {
    // Guard the canonical JSON wire shape of the job envelope: 64-bit ints as
    // strings, enums as SCREAMING_SNAKE strings.
    let request = v1::SubmitJobRequest {
        league: "Standard".to_string(),
        start: Some(v1::ItemSnapshot::from(&sample_snapshot())),
        target: Some(v1::ItemSnapshot::from(&sample_snapshot())),
        matrix_builder: v1::MatrixBuilderPreset::HappyPath as i32,
        path_analyzers: vec![v1::StatisticAnalyzerPathPreset::UniquePathChance as i32],
        group_analyzers: vec![],
        limits: Some(v1::Limits {
            max_routes: 5,
            max_ram_in_bytes: 1_000_000_000,
            timeout_seconds: None,
        }),
        result_options: Some(v1::ResultOptions {
            include_pretty_strings: true,
            include_route_snapshots: false,
            top_n_pretty: Some(5),
        }),
    };

    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(json["matrixBuilder"], "MATRIX_BUILDER_PRESET_HAPPY_PATH");
    assert_eq!(json["limits"]["maxRamInBytes"], "1000000000");

    let back: v1::SubmitJobRequest = serde_json::from_value(json).unwrap();
    assert_eq!(request, back);
}
