use genesis::*;

#[derive(Clone, Debug, Eq, PartialEq)]
struct Position {
    position: (u32, u32),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NameComponent {
    name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RareComponent {
    data: u32,
}

#[world(World, MyComponent)]
#[derive(Clone, Debug, Eq, PartialEq)]
struct Template {
    #[component(vec)] //default, optional
    positions: Position,
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
        let id_a = world.spawn();
        world.register(id_a, Position { position: (1, 2) })?;

        let id_b = world.spawn();
        world.register(
            id_b,
            NameComponent {
                name: String::from("B"),
            },
        )?;

        let id_c = world.spawn();
        world.register(id_c, RareComponent { data: 69 })?;
        world.register(
            id_c,
            NameComponent {
                name: String::from("C"),
            },
        )?;

        assert_eq!(
            world.positions.get(id_a),
            Some(&Position { position: (1, 2) })
        );

        assert_eq!(world.names.get(id_a), None);
        assert_eq!(
            world.names.get(id_b),
            Some(&NameComponent {
                name: String::from("B")
            })
        );
        assert_eq!(
            world.names.get(id_c),
            Some(&NameComponent {
                name: String::from("C")
            })
        );

        world.positions.remove(id_a)?;
        assert_eq!(world.positions.get(id_a), None);

        world.clear();
        assert_eq!(world.names.get(id_b), None);

        Ok(())
    }

    #[test]
    fn test_template() -> Result<(), NoSuchEntity> {
        let mut world = World::new(3);
        let id = world.spawn();

        let template = Template {
            positions: Some(Position { position: (10, 20) }),
            rare_data: Some(RareComponent { data: 42 }),
            ..Default::default()
        };

        // run with cargo test -- --nocapture to see Debug output
        println!("template: {:?}", template);

        assert_eq!(
            template,
            Template {
                positions: Some(Position { position: (10, 20) }),
                names: None,
                rare_data: Some(RareComponent { data: 42 }),
            }
        );

        let old_data_registered = world.register(id, template)?;
        assert_eq!(old_data_registered, Some(Template::default()));

        let updated = Template {
            positions: Some(Position { position: (11, 21) }),
            ..Default::default()
        };

        let removed_data = world.register(id, updated)?;
        assert_eq!(
            removed_data,
            Some(Template {
                positions: Some(Position { position: (10, 20) }),
                ..Default::default()
            })
        );

        Ok(())
    }
}
