use crate::entity::Entity;

pub trait Register<T> {
    fn register(&mut self, id: Entity, item: T);
}
