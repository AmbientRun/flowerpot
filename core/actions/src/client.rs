use ambient_api::prelude::*;

use embers::actions::messages::PerformCraftingAction;

mod shared;

#[main]
fn main() {
    PerformCraftingAction::subscribe(move |_, data| {
        data.send_server_reliable();
    });
}
