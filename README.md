# Genesis - an ECS generator library 

[![Documentation](https://docs.rs/genesis/badge.svg)](https://docs.rs/genesis/)
[![Crates.io](https://img.shields.io/crates/v/genesis.svg)](https://crates.io/crates/genesis)

## Overview 

`genesis` provides an ECS-generating macro.  
Unlike other ECS libraries and frameworks, which do dynamic borrow-checking at runtime,
you define all your components upfront and generate a completely statically typed ECS,
with borrow checking done at compile time.  
Gone are the days of passing a World between functions, only to encounter a dynamic borrow checking problem!

`genesis` is a lightweight ECS library that doesn't provide any scheduling capabilities.
Instead, you can query the storage for each component type directly.

```rust
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

    // spawn an entity
    let id_a = world.spawn();
    // set the components directly on the corresponding storage
    world.indices.set(id_a, IndexComponent { index: 42 })?;

    // spawn another entity
    let id_b = world.spawn();
    // alternative way to set components: using the utility trait Register<T>.
    world.register(id_b, IndexComponent { index: 0 })?;
    world.register(
        id_b,
        NameComponent {
            name: String::from("B"),
        },
    )?;

    let id_c = world.spawn();
    // third way of setting components: using the generated Template struct.
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
```

## Goals
The main goal of `genesis` is to provide a type-safe ECS with compile time borrow checking.  
This can help avoid writing code where you pass an ECS world from one function to another 
while looping over entities and holding onto mutable references in the calling function,
which results in a runtime borrow checking problems.

## Other ECS
There are a lot of different ECS in Rust, all with their own pros and cons. 
Check out these in particular:
- [hecs]
- [specs]
- [legion]

## Benchmarks
Performance is not the primary goal of `genesis`; the main benefit of an ECS is the data modelling enabled 
by the paradigm.  
If you have severe time constraints and need to iterate over hundreds of thousands of entities,
you should look into using an archetype ECS like `hecs` for the better iteration speed gained through 
better data locality and cache friendliness.
Note that archetype-based ECS generally trade off better iteration time for worse performance when it comes 
to adding/removing components than ECS like `specs` or `genesis`, which have different storages.
For more information, check out [benchmarks].
  

## Licence
Licensed under 
* MIT licence
* Apache Licence 2.0

## Contributions 
Contributions are welcome! Unless explicitly stated otherwise, your contribution is 
assumed to be licensed under the same licences as `genesis` (see above).

[hecs]: https://github.com/Ralith/hecs
[specs]: https://github.com/amethyst/specs
[legion]: https://github.com/amethyst/legion
[benchmarks]: https://github.com/rust-gamedev/ecs_bench_suite
