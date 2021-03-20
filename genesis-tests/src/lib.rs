use genesis::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Position {
    pub position: (u32, u32),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NameComponent {
    pub name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RareComponent {
    pub data: u32,
}

#[world(MyComponent, MyEntityTemplate)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct World {
    #[component(vec)] //default, optional
    #[template_name(position)]
    positions: Position,
    #[template_name(name)]
    names: NameComponent,
    #[component(map)]
    rare_data: RareComponent,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_bundle() {
        let comp_a = RareComponent { data: 17 };
        let component_bundle: MyComponent = comp_a.clone().into();
        assert_eq!(
            component_bundle,
            MyComponent::RareComponent(RareComponent { data: 17 })
        );
    }

    #[test]
    fn use_world() -> Result<(), NoSuchEntity> {
        let mut world = World::new(3);
        let entity_a = world.spawn();
        world.register(entity_a, Position { position: (1, 2) })?;

        let entity_b = world.spawn();
        world.register(
            entity_b,
            NameComponent {
                name: String::from("B"),
            },
        )?;

        let entity_c = world.spawn();
        world.register(entity_c, RareComponent { data: 69 })?;
        world.register(
            entity_c,
            NameComponent {
                name: String::from("C"),
            },
        )?;

        assert_eq!(
            world.positions.get(entity_a),
            Some(&Position { position: (1, 2) })
        );

        assert_eq!(world.names.get(entity_a), None);
        assert_eq!(
            world.names.get(entity_b),
            Some(&NameComponent {
                name: String::from("B")
            })
        );
        assert_eq!(
            world.names.get(entity_c),
            Some(&NameComponent {
                name: String::from("C")
            })
        );

        for entity in world.entities.read().unwrap().iter() {
            if let Some(name) = world.names.get(entity) {
                println!("Name: {:?}", name);
            }
        }

        world.positions.remove(entity_a)?;
        assert_eq!(world.positions.get(entity_a), None);

        world.clear();
        assert_eq!(world.names.get(entity_b), None);

        Ok(())
    }

    #[test]
    fn test_template() -> Result<(), NoSuchEntity> {
        let mut world = World::new(3);
        let id = world.spawn();

        let template = MyEntityTemplate {
            position: Some(Position { position: (10, 20) }),
            rare_data: Some(RareComponent { data: 42 }),
            ..Default::default()
        };

        // run with cargo test -- --nocapture to see Debug output
        println!("template: {:?}", template);

        assert_eq!(
            template,
            MyEntityTemplate {
                position: Some(Position { position: (10, 20) }),
                name: None,
                rare_data: Some(RareComponent { data: 42 }),
            }
        );

        let old_data_registered = world.register(id, template)?;
        assert_eq!(old_data_registered, Some(MyEntityTemplate::default()));

        let updated = MyEntityTemplate {
            position: Some(Position { position: (11, 21) }),
            ..Default::default()
        };

        let removed_data = world.register(id, updated)?;
        assert_eq!(
            removed_data,
            Some(MyEntityTemplate {
                position: Some(Position { position: (10, 20) }),
                ..Default::default()
            })
        );

        Ok(())
    }
}
