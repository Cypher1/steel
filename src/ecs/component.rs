use crate::nodes::*;
use super::providers::ComponentID;
use crate::arena::ID;

#[derive(Default, Debug, Clone)]
pub struct Entity {
    pub symbol: Option<ComponentID<Symbol>>,
    pub call: Option<ComponentID<Call<ID>>>,
    pub i_64: Option<ComponentID<i64>>,
    pub optimizer_data: Option<ComponentID<OptimizerData<ID>>>,
    pub shared: Shared<ID>,
}

#[cfg(test)]
mod test {}
