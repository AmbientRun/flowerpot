use ambient_api::prelude::*;

use messages::UpdateMediumCrops;

mod shared;

#[main]
fn main() {
    UpdateMediumCrops::subscribe(move |_, data| {
        eprintln!("{:#?}", data);
    });
}
