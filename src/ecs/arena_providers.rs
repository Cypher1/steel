use super::component::Entity;
use super::providers::{ComponentId, EcsError, EntityId, Provider};
use crate::compact_arena::Arena;

// In future there may be other kinds of Providers.
pub trait ArenaProvider<T> {
    fn entities(&self) -> &Arena<Entity>;
    fn entities_mut(&mut self) -> &mut Arena<Entity>;
    fn make_entity(id: ComponentId<T>) -> Entity;
    fn arena(&self) -> (&Arena<Entity>, &Arena<(EntityId, T)>);
    fn arena_mut(&mut self) -> (&mut Arena<Entity>, &mut Arena<(EntityId, T)>);
    fn get_impl(&self, id: EntityId) -> Result<&T, EcsError>;
    fn get_mut_impl(&mut self, id: EntityId) -> Result<&mut T, EcsError>;
    fn remove_impl(&mut self, id: EntityId) -> Result<T, EcsError>;
}

impl<T, S: ArenaProvider<T>> Provider<T> for S {
    type ID = ComponentId<T>;
    fn overwrite_entity<F: FnOnce(EntityId) -> T>(
        &mut self,
        id: EntityId,
        value: F,
    ) -> Result<(), EcsError> {
        let (entities, arena) = self.arena_mut();
        let node: ComponentId<T> = ComponentId::new(arena.add((id, value(id)))); // ent id and ent component id.
        entities.set(id.id, Self::make_entity(node))?;
        Ok(())
    }
    fn add_with_id<F: FnOnce(EntityId) -> T>(&mut self, value: F) -> EntityId {
        let (entities, arena) = self.arena_mut();
        let index = entities.add_with_id(|id| {
            let id: EntityId = ComponentId::new(id); // Wrap the id with 'entity' information.
            let node: ComponentId<T> = ComponentId::new(arena.add((id, value(id)))); // ent id and ent component id.
            Self::make_entity(node)
        });
        let entity_id: EntityId = ComponentId::new(index);
        entity_id
    }
    fn get_component(&self, id: Self::ID) -> Result<&T, EcsError> {
        Ok(&self.arena().1.get(id.id)?.1)
    }
    fn get_component_mut(&mut self, id: Self::ID) -> Result<&mut T, EcsError> {
        Ok(&mut self.arena_mut().1.get_mut(id.id)?.1)
    }
    fn get_component_for_entity(&self, id: EntityId) -> Result<&T, EcsError> {
        self.get_impl(id)
    }
    fn get_component_for_entity_mut(&mut self, id: EntityId) -> Result<&mut T, EcsError> {
        self.get_mut_impl(id)
    }
    fn remove_component_for_entity(&mut self, id: EntityId) -> Result<T, EcsError> {
        self.remove_impl(id)
    }
}

#[macro_export]
macro_rules! make_arena_provider {
    ($ctx: ty, $type: ty, $kind: ident, $accessor: tt) => {
        impl ArenaProvider<$type> for $ctx {
            fn entities(&self) -> &Arena<Entity> {
                &self.entities
            }
            fn entities_mut(&mut self) -> &mut Arena<Entity> {
                &mut self.entities
            }
            fn make_entity(id: ComponentId<$type>) -> Entity {
                Entity {
                    $kind: Some(id),
                    ..Entity::default()
                }
            }
            fn arena(&self) -> (&Arena<Entity>, &Arena<(EntityId, $type)>) {
                (&self.entities, &self.$accessor)
            }
            fn arena_mut(&mut self) -> (&mut Arena<Entity>, &mut Arena<(EntityId, $type)>) {
                (&mut self.entities, &mut self.$accessor)
            }
            fn get_impl(&self, id: EntityId) -> Result<&$type, EcsError> {
                let ent = &*self.entities.get(id.id)?;
                if let Some(component_id) = ent.$kind {
                    Ok(self.get_component(component_id)?)
                } else {
                    Err(EcsError::ComponentNotFound(
                        std::any::type_name::<$type>().to_string(),
                        id,
                    ))
                }
            }
            fn get_mut_impl(&mut self, id: EntityId) -> Result<&mut $type, EcsError> {
                let ent = &*self.entities.get(id.id)?;
                if let Some(component_id) = ent.$kind {
                    Ok(self.get_component_mut(component_id)?)
                } else {
                    Err(EcsError::ComponentNotFound(
                        std::any::type_name::<$type>().to_string(),
                        id,
                    ))
                }
            }
            fn remove_impl(&mut self, id: EntityId) -> Result<$type, EcsError> {
                let ent = self.entities.get(id.id)?;
                if let Some(component_id) = ent.$kind {
                    let (entities, arena) = self.arena_mut();
                    let (_id, old_value) = arena.remove_by_swap(component_id.id)?;
                    let moved_component_owner = &arena.get(component_id.id)?.0;
                    // println!("id: {:?} rem: {:?} moved_component_owner: {:?} old_value: {:?}", &id, &removed_id, &moved_component_owner, &old_value);
                    // Update the owned component index.
                    entities.get_mut(moved_component_owner.id)?.$kind = Some(component_id);
                    Ok(old_value)
                } else {
                    Err(EcsError::ComponentNotFound(
                        std::any::type_name::<$type>().to_string(),
                        id,
                    ))
                }
            }
        }
    };
}

#[cfg(test)]
mod test {}
