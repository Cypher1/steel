use super::providers::{ComponentId, EntityId};
use crate::nodes::*;

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Entity {
    pub symbol: Option<ComponentId<Symbol>>,
    pub call: Option<ComponentId<Call<EntityId>>>,
    pub i_64: Option<ComponentId<i64>>,
}

#[cfg(test)]
mod test {}
