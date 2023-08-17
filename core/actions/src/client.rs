use ambient_api::prelude::*;

use packages::actions::messages::PerformCraftingAction;

mod shared;

#[main]
fn main() {
    PerformCraftingAction::subscribe(move |_, data| {
        data.send_server_reliable();
    });
}
