use genesis::*;

#[derive(Clone, Debug, Eq, PartialEq)]
struct IndexComponent {
    index: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NameComponent {
    name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RareComponent {
    data: u32,
}

#[world(MyComponent, Template)]
#[derive(Clone, Debug, Eq, PartialEq)]
struct World {
    #[component(vec)] //default, optional
    #[template_name(index)]
    indices: IndexComponent,
    #[template_name(name)]
    names: NameComponent,
    #[component(map)]
    rare_data: RareComponent,
}

fn main() -> Result<(), NoSuchEntity> {
    let initial_capacity = 1024;
    let mut world = World::new(initial_capacity);

    let id_a = world.spawn();
    world.indices.set(id_a, IndexComponent { index: 42 })?;

    let id_b = world.spawn();
    world.register(id_b, IndexComponent { index: 0 })?;
    world.register(
        id_b,
        NameComponent {
            name: String::from("B"),
        },
    )?;

    let id_c = world.spawn();
    world.register(
        id_c,
        Template {
            index: Some(IndexComponent { index: 100 }),
            ..Default::default()
        },
    )?;

    if let Some(a_index) = world.indices.get(id_a) {
        println!("first entity has index {:?}", a_index);
    }

    for id in world.entities.read().unwrap().iter() {
        if let Some(index) = world.indices.get(id) {
            println!("entity {:?} has index {:?}", id, index);
        }
    }

    Ok(())
}
