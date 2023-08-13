use std::sync::atomic::AtomicBool;

use ambient_api::{once_cell::sync::OnceCell, prelude::*};

use embers::{
    crops::components::{class, medium_crop, medium_crop_occupant, on_tile},
    game::assets::url,
    items::components::held_ref,
    map::components::{chunk, chunk_tile_refs},
    player::components::{left_hand_ref, right_hand_ref},
};

mod shared;

/// A single-instance, lazily-spawned entity for use with the Prototype pattern.
pub struct PrototypeEntity {
    entity: OnceCell<EntityId>,
    add_cb: Box<dyn Fn(EntityId) + Send + Sync + 'static>,
    added: AtomicBool,
}

impl PrototypeEntity {
    pub fn new(cb: impl Fn(EntityId) + Send + Sync + 'static) -> Self {
        Self {
            entity: OnceCell::new(),
            add_cb: Box::new(cb),
            added: AtomicBool::new(false),
        }
    }

    pub fn get(&self) -> EntityId {
        let e = *self.entity.get_or_init(|| Entity::new().spawn());

        if !self.added.swap(true, std::sync::atomic::Ordering::SeqCst) {
            (*self.add_cb)(e);
        }

        e
    }
}

macro_rules! expand_props {
    ($e:expr, $component:ident: $value:expr $(, $component_tail:ident: $value_tail:expr)* $(,)?) => {
        expand_props!(Entity::with($e, $component(), $value.into()) $(, $component_tail: $value_tail)*)
    };
    ($e:expr) => ($e);
}

macro_rules! def_entity {
    ($($component:ident: $value:expr),* $(,)?) => {
        expand_props!(Entity::new(), $($component: $value),*)
    }
}

macro_rules! def_prototype {
    ($item_name:ident $(, $component:ident: $value:expr)* $(,)?) => {
        ::lazy_static::lazy_static! {
            pub static ref $item_name: PrototypeEntity = PrototypeEntity::new(move |e| {
                entity::add_components(e, def_entity!($($component: $value),*));
            });
        }
    }
}

pub mod crops {
    use super::*;

    use crate::embers::crops::components::{
        model_prefab_url as prefab, next_growth_age as next_age, next_growth_stage as next_stage, *,
    };

    pub mod beans {
        use super::*;

        def_prototype!(
            YOUNG_0,
            prefab: url("crops/beans/Beans_0.fbx"),
            next_stage: YOUNG_1.get(),
            next_age: 2u16,
        );

        def_prototype!(
            YOUNG_1,
            prefab: url("crops/beans/Beans_1.fbx"),
            next_stage: YOUNG_2.get(),
            next_age: 3u16,
        );

        def_prototype!(
            YOUNG_2,
            prefab: url("crops/beans/Beans_2.fbx"),
            next_stage: FLOWERING.get(),
            next_age: 7u16,
        );

        def_prototype!(
            FLOWERING,
            prefab: url("crops/beans/Beans_3.fbx"),
            next_stage: FRUITING.get(),
            next_age: 21u16,
        );

        def_prototype!(
            FRUITING,
            prefab: url("crops/beans/Beans_4.fbx"),
            next_stage: DEAD.get(),
            next_age: 50u16,
            seed: YOUNG_0.get(),
            seeding_interval: 13u16,
        );

        def_prototype!(
            HARVESTED,
            prefab: url("crops/beans/Beans_5.fbx"),
            next_stage: DEAD.get(),
            next_age: 10u16,
        );

        def_prototype!(
            DEAD,
            prefab: url("crops/beans/Beans_6.fbx"),
            next_stage: EntityId::null(),
            next_age: 50u16,
        );
    }

    pub mod carrot {
        use super::*;
    }

    pub mod corn {
        use super::*;
    }

    pub mod garlic {
        use super::*;
    }

    pub mod pepper {
        use super::*;
    }

    pub mod potato {
        use super::*;
    }

    pub mod sugarcane {
        use super::*;
    }

    pub mod tomato {
        use super::*;
    }

    pub mod wheat {
        use super::*;
    }
}

pub mod items {
    use super::*;

    pub mod debug {
        use super::*;

        use ambient_api::{core::rendering::components::color, prelude::vec4};

        def_prototype!(
            BLUE,
            color: vec4(0.0, 0.0, 1.0, 1.0),
        );

        def_prototype!(
            YELLOW,
            color: vec4(1.0, 1.0, 0.0, 1.0),
        );

        def_prototype!(
            GREEN,
            color: vec4(0.0, 1.0, 0.0, 1.0),
        );
    }
}

#[main]
fn main() {
    spawn_query((chunk(), chunk_tile_refs())).bind(move |entities| {
        for (_, (chunk, tiles)) in entities {
            if chunk != IVec2::ZERO {
                continue;
            }

            let tile = tiles[0];

            let crop_class = crops::beans::YOUNG_0.get();
            let dummy_crop = Entity::new()
                .with_default(medium_crop())
                .with(class(), crop_class)
                .with(on_tile(), tile)
                .spawn();

            entity::add_component(tile, medium_crop_occupant(), dummy_crop);
        }
    });

    spawn_query((left_hand_ref(), right_hand_ref())).bind(move |entities| {
        for (_player, (left, right)) in entities {
            entity::add_component(left, held_ref(), items::debug::YELLOW.get());
            entity::add_component(right, held_ref(), items::debug::BLUE.get());
        }
    });

    use embers::crafting::components::*;
    def_entity!(
        recipe: (),
        primary_ingredient: items::debug::YELLOW.get(),
        secondary_ingredient: items::debug::BLUE.get(),
        primary_yield: items::debug::GREEN.get(),
        secondary_yield: EntityId::null(),
    )
    .spawn();
}
