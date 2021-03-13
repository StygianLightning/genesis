use crate::entity::Entity;
use crate::NoSuchEntity;

pub trait Register<T> {
    fn register(&mut self, id: Entity, item: T) -> Result<Option<T>, NoSuchEntity>;
}
