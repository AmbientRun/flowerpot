use ambient_api::{messages::Frame, prelude::*};

use crate::messages::PerformCraftingAction;

mod shared;

#[main]
fn main() {
    PerformCraftingAction::subscribe(move |_, data| {
        data.send_server_reliable();
    });
}
