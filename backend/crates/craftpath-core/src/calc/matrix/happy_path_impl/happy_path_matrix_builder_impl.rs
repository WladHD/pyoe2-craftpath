use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::Result;

use crate::{
    api::{
        calculator::{ItemMatrix, ItemMatrixNode, MatrixBuilder, PropagationTarget},
        currency::CraftCurrencyList,
        item::{Item, ItemSnapshot, ItemTechnicalMeta},
        matrix_propagator::MatrixPropagator,
        provider::{item_info::ItemInfoProvider, market_prices::MarketPriceProvider},
        types::{THashMap, THashSet},
    },
    calc::matrix::happy_path_impl::propagators::{
        artificers_orb::ArtificersOrbPropagator, chaos_orb::ChaosOrbPropagator,
        desecration::DesecrationPropagator, exalted_orb::ExaltedOrbPropagator,
        fracturing_orb::FracturingOrbPropagator, normal_essences::NormalEssencePropagator,
        orb_of_annulment::OrbOfAnnulmentPropagator,
        orb_of_augmentation::OrbOfAugmentationPropagator,
        orb_of_transmutation::OrbOfTransmutationPropagator,
        perfect_essences::PerfectEssencePropagator, regal_orb::RegalOrbPropagator,
        vaal_orb_socket::VaalOrbSocketPropagator,
    },
    utils::{fraction_utils::Fraction, hash_utils::hash_value},
};

#[derive(Clone, Debug)]
pub struct HappyPathMatrixBuilderImpl;

impl MatrixBuilder for HappyPathMatrixBuilderImpl {
    fn get_name(&self) -> &'static str {
        "Happy Path Matrix Builder"
    }

    fn get_description(&self) -> &'static str {
        "Builds an optimized item matrix containing reachable items starting from \
        the given item, that only come closer to the target item (target_proximity)."
    }

    fn generate_item_matrix(
        &self,
        starting_item: ItemSnapshot,
        target_item: ItemSnapshot,
        item_info: &ItemInfoProvider,
        market_info: &MarketPriceProvider,
    ) -> Result<ItemMatrix> {
        generate_item_matrix(starting_item, target_item, item_info, market_info)
    }
}

fn generate_item_matrix(
    starting_item: ItemSnapshot,
    target_item: ItemSnapshot,
    item_info: &ItemInfoProvider,
    market_info: &MarketPriceProvider,
) -> Result<ItemMatrix> {
    let mut matrix = ItemMatrix::default();
    let mut todo_items: THashSet<PropagationTarget> = THashSet::default();

    todo_items.insert(PropagationTarget {
        next: starting_item,
        chance: Fraction::one(),
        meta: ItemTechnicalMeta::default(),
    });

    // setup propagators
    let propagators: Vec<Box<dyn MatrixPropagator>> = vec![
        Box::new(FracturingOrbPropagator),
        Box::new(OrbOfTransmutationPropagator),
        Box::new(OrbOfAugmentationPropagator),
        Box::new(RegalOrbPropagator),
        Box::new(ExaltedOrbPropagator),
        Box::new(ChaosOrbPropagator),
        Box::new(OrbOfAnnulmentPropagator),
        Box::new(PerfectEssencePropagator),
        Box::new(DesecrationPropagator),
        Box::new(NormalEssencePropagator),
        // finishers
        Box::new(ArtificersOrbPropagator),
        Box::new(VaalOrbSocketPropagator),
    ];

    let essence_only: Vec<Box<dyn MatrixPropagator>> = vec![Box::new(PerfectEssencePropagator)];

    tracing::info!("Starting propagation ...");

    let count_removed: AtomicUsize = AtomicUsize::new(0usize);

    while !todo_items.is_empty() {
        let items = todo_items
            .iter()
            .filter_map(|propagation_target| {
                let Ok(mut item) = Item::build_with(propagation_target.next.clone(), &target_item, &item_info)
                else {
                    return None;
                };

                item.meta = propagation_target.meta.clone();

                let mut hm: THashMap<CraftCurrencyList, Vec<PropagationTarget>> =
                    THashMap::default();

                let propagators = if item.meta.mark_for_essence_only {
                    &essence_only
                } else {
                    &propagators
                };

                if item.helper.target_proximity != 0 && !item.snapshot.corrupted {
                    // propagate all items starting from item_snapshot
                    // should also check for same chance, but higher cost -> remove
                    for some_propagator in propagators.iter() {
                        if !some_propagator.is_applicable(&item, &item_info) {
                            continue;
                        }

                        match some_propagator.propagate_step(&item, &target_item, &item_info) {
                            Ok(mut prop) => {
                                let mut reached: THashMap<(ItemSnapshot, Fraction), f64> =
                                    THashMap::default();

                                let mut sorted_groups_by_cost = prop
                                    .keys()
                                    .map(|e| {
                                        (
                                            e.clone(),
                                            e.list.iter().fold(0f64, |a, b| {
                                                a + market_info
                                                    .try_lookup_currency_in_divines_default_if_fail(b, item_info).get_divine_value()
                                            }),
                                        )
                                    })
                                    .collect::<Vec<(CraftCurrencyList, f64)>>();

                                sorted_groups_by_cost.sort_by(|a, b| {
                                    a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
                                });

                                for (sorted_group, group_cost) in sorted_groups_by_cost {
                                    match prop.get_mut(&sorted_group) {
                                        None => panic!(
                                            "Could not find group anymore, that was just handled."
                                        ),
                                        Some(sorted_group) => {
                                            sorted_group.retain(|test| {
                                                let key = (test.next.clone(), test.chance.clone());

                                                match reached.get(&key) {
                                                    Some(cheapest_cost) => {
                                                        if cheapest_cost > &group_cost {
                                                            tracing::warn!(
                                                                "Unexpectedly after sorting by currency, a cheaper route was found. Program will proceed."
                                                            );
                                                            return true;
                                                        }
                                                        count_removed.fetch_add(1, Ordering::Relaxed);
                                                        false
                                                    }
                                                    None => {
                                                        reached.insert(key, group_cost.clone());
                                                        true
                                                    }
                                                }
                                            });
                                        }
                                    }
                                }

                                hm.extend(prop.drain());
                            }
                            Err(e) => {
                                tracing::error!("Propagation failed, skipping ... {:#?}", e)
                            }
                        }
                    }
                }

                Some((
                    item.snapshot.clone(),
                    ItemMatrixNode {
                        item,
                        propagate: hm,
                    },
                ))
            })
            .collect::<Vec<(ItemSnapshot, ItemMatrixNode)>>();

        todo_items.clear();

        // add every next item -> unchecked
        for (snapshot, node) in items {
            node.propagate.values().for_each(|targets| {
                targets.iter().for_each(|target| {
                    todo_items.insert(target.clone());
                })
            });

            matrix
                .entry(hash_value(&snapshot))
                .and_modify(|existing_node| {
                    // Merge propagate maps, should not happen though
                    for (k, v) in node.propagate.iter() {
                        existing_node
                            .propagate
                            .entry(k.clone())
                            .and_modify(|existing_vec| existing_vec.extend(v.clone()))
                            .or_insert(v.clone());

                        // dedup just in case
                        if let Some(e) = existing_node.propagate.get_mut(k) {
                            let mut set = THashSet::default();
                            e.retain(|x| set.insert(x.clone()));
                        }
                    }
                })
                .or_insert(node);
        }

        // remove already calculated items from todo
        todo_items.retain(|test| !matrix.contains_key(&hash_value(&test.next)));
    }

    let fetched = count_removed.load(Ordering::Relaxed);

    if fetched > 0 {
        tracing::info!(
            "Excluded {} more expensive routes with same chance successfully",
            fetched
        );
    }

    Ok(matrix)
}
