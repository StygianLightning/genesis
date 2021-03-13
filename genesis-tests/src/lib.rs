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

#[world(MyComponent)]
#[derive(Clone, Debug, Eq, PartialEq)]
struct World {
    #[component(vec)] //default, optional
    positions: Position,
    names: NameComponent,
    #[component(map)]
    data: RareComponent,
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
}
