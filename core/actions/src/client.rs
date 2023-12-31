use ambient_api::prelude::*;

use packages::this::messages::*;

mod shared;

#[main]
fn main() {
    PerformCraftingAction::subscribe(move |_, data| {
        data.send_server_reliable();
    });

    PerformTileAction::subscribe(move |_, data| {
        data.send_server_reliable();
    });

    PerformSwap::subscribe(move |_, data| {
        data.send_server_reliable();
    });
}
