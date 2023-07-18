use ambient_api::{messages::Frame, prelude::*};

use crate::messages::PerformCraftingAction;

mod shared;

#[main]
fn main() {
    Frame::subscribe(move |_| {
        // TODO interoperate with cursor lock?
        let (delta, _) = input::get_delta();

        if delta.keys.contains(&KeyCode::Q) {
            eprintln!("sending crafting action");
            PerformCraftingAction::new().send_server_reliable();
        }
    });
}
