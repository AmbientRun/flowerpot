use ambient_api::prelude::*;

use crate::packages::this::components::*;

pub fn init_shared() {
    spawn_query(class_ref())
        .excludes(is_thing())
        .bind(move |things| {
            for (e, class) in things {
                // retrieve the class's components
                let mut base = entity::get_all_components(class);
                base.remove(is_class());
                base.set(is_thing(), ());

                // override class components with the original thing's components
                let child = entity::get_all_components(class);

                // add the class's components onto the newly christened thing
                entity::add_components(e, base.with_merge(child));
            }
        });
}
