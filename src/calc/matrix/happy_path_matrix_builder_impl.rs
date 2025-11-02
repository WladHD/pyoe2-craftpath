use crate::api::{
    calculator::{ItemMatrix, MatrixBuilder},
    item::ItemSnapshot,
    provider::item_info::ItemInfoProvider,
};

pub struct HappyPathMatrixBuilderImpl;

impl MatrixBuilder for HappyPathMatrixBuilderImpl {
    fn get_name(&self) -> &'static str {
        "Happy Path Matrix Builder"
    }

    fn get_description(&self) -> &'static str {
        "Builds an optimized item matrix containing reachable items starting from \
        the given item, that only come closer to the target item (target_proximity). \
        Following currencies are applied: XXX"
    }

    fn generate_item_matrix(
        &self,
        starting_item: ItemSnapshot,
        target: ItemSnapshot,
        item_info: &ItemInfoProvider,
    ) -> anyhow::Result<ItemMatrix> {
        todo!()
    }
}
