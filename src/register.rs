use crate::entity::Entity;
use crate::NoSuchEntity;

/// Type that can be registered. Can be used to set components in ECS Worlds generated by `genesis`.
pub trait Register<T> {
    /// Register the given item for the given entity.
    /// Returns Err(NoSuchEntity) if the given entity doesn't exist.
    /// Otherwise, returns the previously associated item.
    /// For normal components used to generate a World, this is equivalent to calling `.set()`
    /// on the corresponding storage field.
    fn register(&mut self, entity: Entity, item: T) -> Result<Option<T>, NoSuchEntity>;
}
