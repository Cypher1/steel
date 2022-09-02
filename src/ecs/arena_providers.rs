use super::component::{ComponentID, EcsError, Entity};
use super::providers::Provider;
use crate::arena::{Arena, ID};

// In future there may be other kinds of Providers.
pub trait ArenaProvider<'a, T> {
    fn entities(&self) -> &Arena<Entity>;
    fn entities_mut(&mut self) -> &mut Arena<Entity>;
    fn make_entity(id: ID) -> Entity;
    fn arena(&self) -> (&Arena<Entity>, &Arena<T>);
    fn arena_mut(&mut self) -> (&mut Arena<Entity>, &mut Arena<T>);
    fn get_impl(&self, id: ID) -> Result<&T, EcsError>;
    fn get_mut_impl(&mut self, id: ID) -> Result<&mut T, EcsError>;
}

impl<'a, T: 'a, S: ArenaProvider<'a, T>> Provider<'a, T> for S {
    type ID = ComponentID<T>;
    fn add_with_id<F: FnOnce(ID) -> T>(&mut self, value: F) -> ID {
        let (entities, arena) = self.arena_mut();
        entities.add_with_id(|id| {
            let node = arena.add(value(id)); // raw id and raw component id.
            Self::make_entity(node)
        })
    }
    fn get_component(&self, id: Self::ID) -> Result<&T, EcsError> {
        Ok(self.arena().1.get(id.id)?)
    }
    fn get_component_mut(&mut self, id: Self::ID) -> Result<&mut T, EcsError> {
        Ok(self.arena_mut().1.get_mut(id.id)?)
    }
    fn get_component_for_entity(&self, id: ID) -> Result<&T, EcsError> {
        self.get_impl(id)
    }
    fn get_component_for_entity_mut(&mut self, id: ID) -> Result<&mut T, EcsError> {
        self.get_mut_impl(id)
    }
}

#[macro_export]
macro_rules! make_arena_provider {
    ($ctx: ty, $type: ty, $kind: tt, $accessor: tt) => {
        impl<'a> ArenaProvider<'a, $type> for $ctx {
            fn entities(&self) -> &Arena<Entity> {
                &self.entities
            }
            fn entities_mut(&mut self) -> &mut Arena<Entity> {
                &mut self.entities
            }
            fn make_entity(id: ID) -> Entity {
                Entity::$kind(ComponentID {
                    id,
                    ty: PhantomData,
                })
            }
            fn arena(&self) -> (&Arena<Entity>, &Arena<$type>) {
                (&self.entities, &self.$accessor)
            }
            fn arena_mut(&mut self) -> (&mut Arena<Entity>, &mut Arena<$type>) {
                (&mut self.entities, &mut self.$accessor)
            }
            fn get_impl(&self, id: ID) -> Result<&$type, EcsError> {
                let ent = self.entities.get(id)?;
                match ent {
                    Entity::$kind(component_id) => Ok(self.get_component(*component_id)?),
                    _ => Err(EcsError::ComponentNotFound(id)),
                }
            }
            fn get_mut_impl(&mut self, id: ID) -> Result<&mut $type, EcsError> {
                let ent = self.entities.get(id)?;
                match ent {
                    Entity::$kind(component_id) => Ok(self.get_component_mut(*component_id)?),
                    _ => Err(EcsError::ComponentNotFound(id)),
                }
            }
        }
    };
}

#[cfg(test)]
mod test {}
