use crate::nodes::*;
use super::providers::{ComponentId, EntityId};

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Entity {
    pub symbol: Option<ComponentId<Symbol>>,
    pub call: Option<ComponentId<Call<EntityId>>>,
    pub i_64: Option<ComponentId<i64>>,
    pub optimizer_data: Option<ComponentId<OptimizerData<EntityId>>>,
    pub shared: Shared<EntityId>,
}

#[cfg(test)]
mod test {}
