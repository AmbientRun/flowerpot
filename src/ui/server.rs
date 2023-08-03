use ambient_api::prelude::*;

mod shared;

use messages::{JoinRequest, JoinDenied};

#[main]
fn main() {
    JoinRequest::subscribe(move |source, data| {
        let Some(player) = source.client_user_id() else { return };

        let reason = format!("Joining is unimplemented! Nothing personal, {}.", data.name);

        JoinDenied::new(reason).send_client_targeted_reliable(player);
    });
}
