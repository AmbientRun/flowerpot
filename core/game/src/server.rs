use std::sync::atomic::AtomicBool;

use ambient_api::{once_cell::sync::OnceCell, prelude::*};

use flowerpot_common::CHUNK_SIZE;
use packages::{
    crops::components::{class, is_medium_crop, medium_crop_occupant, on_tile},
    game::assets::url,
    items::components::held_ref,
    map::components::{chunk, chunk_tile_refs},
    nameplate::components::name,
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

    use crate::packages::crops::components::{
        model_prefab_url as prefab, next_growth_age as next_age, next_growth_stage as next_stage, *,
    };

    lazy_static::lazy_static! {
        pub static ref SHOWCASE: Vec<Vec<EntityId>> = vec![
            // beans
            vec![
                beans::STAGE_0.get(),
                beans::STAGE_1.get(),
                beans::STAGE_2.get(),
                beans::STAGE_3.get(),
                beans::STAGE_4.get(),
                beans::STAGE_5.get(),
                beans::STAGE_6.get(),

            ],

            // carrots
            vec![
                carrots::STAGE_0.get(),
                carrots::STAGE_1.get(),
                carrots::STAGE_2.get(),
                carrots::STAGE_3.get(),
                carrots::STAGE_4.get(),
                carrots::STAGE_5.get(),
            ],

            // corn
            vec![
                corn::STAGE_0.get(),
                corn::STAGE_1.get(),
                corn::STAGE_2.get(),
                corn::STAGE_3.get(),
                corn::STAGE_4.get(),
                corn::STAGE_5.get(),
                corn::STAGE_6.get(),
                corn::STAGE_7.get(),
            ],

            // garlic
            vec![
                garlic::STAGE_0.get(),
                garlic::STAGE_1.get(),
                garlic::STAGE_2.get(),
                garlic::STAGE_3.get(),
                garlic::STAGE_4.get(),
                garlic::STAGE_5.get(),
                garlic::STAGE_6.get(),
                garlic::STAGE_7.get(),
            ],

            // peppers
            vec![
                peppers::STAGE_0.get(),
                peppers::STAGE_1.get(),
                peppers::STAGE_2.get(),
                peppers::STAGE_3.get(),
                peppers::STAGE_4.get(),
                peppers::STAGE_5.get(),
                peppers::STAGE_6.get(),
                peppers::STAGE_7.get(),
                peppers::STAGE_8.get(),
            ],

            // potatos
            vec![
                potatos::STAGE_0.get(),
                potatos::STAGE_1.get(),
                potatos::STAGE_2.get(),
                potatos::STAGE_3.get(),
                potatos::STAGE_4.get(),
                potatos::STAGE_5.get(),
            ],

            // sugarcane
            vec![
                sugarcane::STAGE_0.get(),
                sugarcane::STAGE_1.get(),
                sugarcane::STAGE_2.get(),
                sugarcane::STAGE_3.get(),
                sugarcane::STAGE_4.get(),
                sugarcane::STAGE_5.get(),
            ],

            // tomatoes
            vec![
                tomatoes::STAGE_0.get(),
                tomatoes::STAGE_1.get(),
                tomatoes::STAGE_2.get(),
                tomatoes::STAGE_3.get(),
                tomatoes::STAGE_4.get(),
                tomatoes::STAGE_5.get(),
                tomatoes::STAGE_6.get(),
                tomatoes::STAGE_7.get(),
                tomatoes::STAGE_8.get(),
            ],

            // wheat
            vec![
                wheat::STAGE_0.get(),
                wheat::STAGE_1.get(),
                wheat::STAGE_2.get(),
                wheat::STAGE_3.get(),
                wheat::STAGE_4.get(),
                wheat::STAGE_5.get(),
                wheat::STAGE_6.get(),
                wheat::STAGE_7.get(),
            ],
        ];
    }

    pub mod beans {
        use super::*;

        def_prototype!(
            STAGE_0,
            prefab: url("crops/medium/Beans/Beans_0.fbx"),
            name: "Beans_0"
        );

        def_prototype!(
            STAGE_1,
            prefab: url("crops/medium/Beans/Beans_1.fbx"),
            name: "Beans_1"
        );

        def_prototype!(
            STAGE_2,
            prefab: url("crops/medium/Beans/Beans_2.fbx"),
            name: "Beans_2"
        );

        def_prototype!(
            STAGE_3,
            prefab: url("crops/medium/Beans/Beans_3.fbx"),
            name: "Beans_3"
        );

        def_prototype!(
            STAGE_4,
            prefab: url("crops/medium/Beans/Beans_4.fbx"),
            name: "Beans_4"
        );

        def_prototype!(
            STAGE_5,
            prefab: url("crops/medium/Beans/Beans_5.fbx"),
            name: "Beans_5"
        );

        def_prototype!(
            STAGE_6,
            prefab: url("crops/medium/Beans/Beans_6.fbx"),
            name: "Beans_6"
        );
    }

    pub mod carrots {
        use super::*;

        def_prototype!(
            STAGE_0,
            prefab: url("crops/medium/Carrots/Carrot_0.fbx"),
            name: "Carrot_0"
        );

        def_prototype!(
            STAGE_1,
            prefab: url("crops/medium/Carrots/Carrot_1.fbx"),
            name: "Carrot_1"
        );

        def_prototype!(
            STAGE_2,
            prefab: url("crops/medium/Carrots/Carrot_2.fbx"),
            name: "Carrot_2"
        );

        def_prototype!(
            STAGE_3,
            prefab: url("crops/medium/Carrots/Carrot_3.fbx"),
            name: "Carrot_3"
        );

        def_prototype!(
            STAGE_4,
            prefab: url("crops/medium/Carrots/Carrot_4.fbx"),
            name: "Carrot_4"
        );

        def_prototype!(
            STAGE_5,
            prefab: url("crops/medium/Carrots/Carrot_5.fbx"),
            name: "Carrot_5"
        );
    }

    pub mod corn {
        use super::*;

        def_prototype!(
            STAGE_0,
            prefab: url("crops/medium/Corn/Corn_0.fbx"),
            name: "Corn_0"
        );

        def_prototype!(
            STAGE_1,
            prefab: url("crops/medium/Corn/Corn_1.fbx"),
            name: "Corn_1"
        );

        def_prototype!(
            STAGE_2,
            prefab: url("crops/medium/Corn/Corn_2.fbx"),
            name: "Corn_2"
        );

        def_prototype!(
            STAGE_3,
            prefab: url("crops/medium/Corn/Corn_3.fbx"),
            name: "Corn_3"
        );

        def_prototype!(
            STAGE_4,
            prefab: url("crops/medium/Corn/Corn_4.fbx"),
            name: "Corn_4"
        );

        def_prototype!(
            STAGE_5,
            prefab: url("crops/medium/Corn/Corn_5.fbx"),
            name: "Corn_5"
        );

        def_prototype!(
            STAGE_6,
            prefab: url("crops/medium/Corn/Corn_6.fbx"),
            name: "Corn_6"
        );

        def_prototype!(
            STAGE_7,
            prefab: url("crops/medium/Corn/Corn_7.fbx"),
            name: "Corn_7"
        );

        def_prototype!(
            STAGE_8,
            prefab: url("crops/medium/Corn/Corn_8.fbx"),
            name: "Corn_8"
        );
    }

    pub mod garlic {
        use super::*;

        def_prototype!(
            STAGE_0,
            prefab: url("crops/medium/Garlic/Garlic_0.fbx"),
            name: "Garlic_0"
        );

        def_prototype!(
            STAGE_1,
            prefab: url("crops/medium/Garlic/Garlic_1.fbx"),
            name: "Garlic_1"
        );

        def_prototype!(
            STAGE_2,
            prefab: url("crops/medium/Garlic/Garlic_2.fbx"),
            name: "Garlic_2"
        );

        def_prototype!(
            STAGE_3,
            prefab: url("crops/medium/Garlic/Garlic_3.fbx"),
            name: "Garlic_3"
        );

        def_prototype!(
            STAGE_4,
            prefab: url("crops/medium/Garlic/Garlic_4.fbx"),
            name: "Garlic_4"
        );

        def_prototype!(
            STAGE_5,
            prefab: url("crops/medium/Garlic/Garlic_5.fbx"),
            name: "Garlic_5"
        );

        def_prototype!(
            STAGE_6,
            prefab: url("crops/medium/Garlic/Garlic_6.fbx"),
            name: "Garlic_6"
        );

        def_prototype!(
            STAGE_7,
            prefab: url("crops/medium/Garlic/Garlic_7.fbx"),
            name: "Garlic_7"
        );
    }

    pub mod peppers {
        use super::*;

        def_prototype!(
            STAGE_0,
            prefab: url("crops/medium/Peppers/Peppers_0.fbx"),
            name: "Peppers_0"
        );

        def_prototype!(
            STAGE_1,
            prefab: url("crops/medium/Peppers/Peppers_1.fbx"),
            name: "Peppers_1"
        );

        def_prototype!(
            STAGE_2,
            prefab: url("crops/medium/Peppers/Peppers_2.fbx"),
            name: "Peppers_2"
        );

        def_prototype!(
            STAGE_3,
            prefab: url("crops/medium/Peppers/Peppers_3.fbx"),
            name: "Peppers_3"
        );

        def_prototype!(
            STAGE_4,
            prefab: url("crops/medium/Peppers/Peppers_4.fbx"),
            name: "Peppers_4"
        );

        def_prototype!(
            STAGE_5,
            prefab: url("crops/medium/Peppers/Peppers_5.fbx"),
            name: "Peppers_5"
        );

        def_prototype!(
            STAGE_6,
            prefab: url("crops/medium/Peppers/Peppers_6.fbx"),
            name: "Peppers_6"
        );

        def_prototype!(
            STAGE_7,
            prefab: url("crops/medium/Peppers/Peppers_7.fbx"),
            name: "Peppers_7"
        );

        def_prototype!(
            STAGE_8,
            prefab: url("crops/medium/Peppers/Peppers_8.fbx"),
            name: "Peppers_8"
        );
    }

    pub mod potatos {
        use super::*;

        def_prototype!(
            STAGE_0,
            prefab: url("crops/medium/Potatos/Potato_0.fbx"),
            name: "Potato_0"
        );

        def_prototype!(
            STAGE_1,
            prefab: url("crops/medium/Potatos/Potato_1.fbx"),
            name: "Potato_1"
        );

        def_prototype!(
            STAGE_2,
            prefab: url("crops/medium/Potatos/Potato_2.fbx"),
            name: "Potato_2"
        );

        def_prototype!(
            STAGE_3,
            prefab: url("crops/medium/Potatos/Potato_3.fbx"),
            name: "Potato_3"
        );

        def_prototype!(
            STAGE_4,
            prefab: url("crops/medium/Potatos/Potato_4.fbx"),
            name: "Potato_4"
        );

        def_prototype!(
            STAGE_5,
            prefab: url("crops/medium/Potatos/Potato_5.fbx"),
            name: "Potato_5"
        );
    }

    pub mod sugarcane {
        use super::*;

        def_prototype!(
            STAGE_0,
            prefab: url("crops/medium/Sugarcane/Sugarcane_0.fbx"),
            name: "Sugarcane_0"
        );

        def_prototype!(
            STAGE_1,
            prefab: url("crops/medium/Sugarcane/Sugarcane_1.fbx"),
            name: "Sugarcane_1"
        );

        def_prototype!(
            STAGE_2,
            prefab: url("crops/medium/Sugarcane/Sugarcane_2.fbx"),
            name: "Sugarcane_2"
        );

        def_prototype!(
            STAGE_3,
            prefab: url("crops/medium/Sugarcane/Sugarcane_3.fbx"),
            name: "Sugarcane_3"
        );

        def_prototype!(
            STAGE_4,
            prefab: url("crops/medium/Sugarcane/Sugarcane_4.fbx"),
            name: "Sugarcane_4"
        );

        def_prototype!(
            STAGE_5,
            prefab: url("crops/medium/Sugarcane/Sugarcane_5.fbx"),
            name: "Sugarcane_5"
        );
    }

    pub mod tomatoes {
        use super::*;

        def_prototype!(
            STAGE_0,
            prefab: url("crops/medium/Tomatos/Tomatos_0.fbx"),
            name: "Tomatos_0"
        );

        def_prototype!(
            STAGE_1,
            prefab: url("crops/medium/Tomatos/Tomatos_1.fbx"),
            name: "Tomatos_1"
        );

        def_prototype!(
            STAGE_2,
            prefab: url("crops/medium/Tomatos/Tomatos_2.fbx"),
            name: "Tomatos_2"
        );

        def_prototype!(
            STAGE_3,
            prefab: url("crops/medium/Tomatos/Tomatos_3.fbx"),
            name: "Tomatos_3"
        );

        def_prototype!(
            STAGE_4,
            prefab: url("crops/medium/Tomatos/Tomatos_4.fbx"),
            name: "Tomatos_4"
        );

        def_prototype!(
            STAGE_5,
            prefab: url("crops/medium/Tomatos/Tomatos_5.fbx"),
            name: "Tomatos_5"
        );

        def_prototype!(
            STAGE_6,
            prefab: url("crops/medium/Tomatos/Tomatos_6.fbx"),
            name: "Tomatos_6"
        );

        def_prototype!(
            STAGE_7,
            prefab: url("crops/medium/Tomatos/Tomatos_7.fbx"),
            name: "Tomatos_7"
        );

        def_prototype!(
            STAGE_8,
            prefab: url("crops/medium/Tomatos/Tomatos_8.fbx"),
            name: "Tomatos_8"
        );
    }

    pub mod wheat {
        use super::*;

        def_prototype!(
            STAGE_0,
            prefab: url("crops/medium/Wheat/Wheat_0.fbx"),
            name: "Wheat_0"
        );

        def_prototype!(
            STAGE_1,
            prefab: url("crops/medium/Wheat/Wheat_1.fbx"),
            name: "Wheat_1"
        );

        def_prototype!(
            STAGE_2,
            prefab: url("crops/medium/Wheat/Wheat_2.fbx"),
            name: "Wheat_2"
        );

        def_prototype!(
            STAGE_3,
            prefab: url("crops/medium/Wheat/Wheat_3.fbx"),
            name: "Wheat_3"
        );

        def_prototype!(
            STAGE_4,
            prefab: url("crops/medium/Wheat/Wheat_4.fbx"),
            name: "Wheat_4"
        );

        def_prototype!(
            STAGE_5,
            prefab: url("crops/medium/Wheat/Wheat_5.fbx"),
            name: "Wheat_5"
        );

        def_prototype!(
            STAGE_6,
            prefab: url("crops/medium/Wheat/Wheat_6.fbx"),
            name: "Wheat_6"
        );

        def_prototype!(
            STAGE_7,
            prefab: url("crops/medium/Wheat/Wheat_7.fbx"),
            name: "Wheat_7"
        );
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
            if chunk.x < 0 || chunk.y < 0 {
                continue;
            }

            let row_spacing = 3;
            let col_spacing = 3;

            let chunk_offset = chunk * CHUNK_SIZE as i32;
            for y in 0..CHUNK_SIZE {
                let wy = y as i32 + chunk_offset.y;
                if wy % row_spacing != 0 {
                    continue;
                }

                let row_idx = wy / row_spacing;
                let Some(row) = crops::SHOWCASE.get(row_idx as usize) else {
                    continue;
                };

                for x in 0..CHUNK_SIZE {
                    let wx = x as i32 + chunk_offset.x;
                    if wx % col_spacing != 0 {
                        continue;
                    }

                    let col_idx = wx / col_spacing;
                    let Some(crop) = row.get(col_idx as usize) else {
                        continue;
                    };

                    let tile_idx = y * CHUNK_SIZE + x;
                    let tile = tiles[tile_idx];

                    let crop = Entity::new()
                        .with(is_medium_crop(), ())
                        .with(class(), *crop)
                        .with(on_tile(), tile)
                        .spawn();

                    entity::add_component(tile, medium_crop_occupant(), crop);
                }
            }
        }
    });

    spawn_query((left_hand_ref(), right_hand_ref())).bind(move |entities| {
        for (_player, (left, right)) in entities {
            entity::add_component(left, held_ref(), items::debug::YELLOW.get());
            entity::add_component(right, held_ref(), items::debug::BLUE.get());
        }
    });

    use packages::crafting::components::*;
    def_entity!(
        is_recipe: (),
        primary_ingredient: items::debug::YELLOW.get(),
        secondary_ingredient: items::debug::BLUE.get(),
        primary_yield: items::debug::GREEN.get(),
        secondary_yield: EntityId::null(),
    )
    .spawn();
}
