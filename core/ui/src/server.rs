use ambient_api::{
    components::core::player::{player, user_id},
    prelude::*,
};

mod shared;

use components::fauna;
use messages::{
    AcceptJoin, Announcement, ChatDenied, ChatMessage, JoinDenied, JoinRequest, PlayerMessage,
};
use rustrict::Censor;

fn moderate_content(input: &str) -> Option<String> {
    let analysis = Censor::from_str(input).analyze();

    if analysis.is(rustrict::Type::ANY) {
        Some(format!("Inappropriate: {:?}", analysis))
    } else {
        None
    }
}

#[main]
fn main() {
    let make_player_query = || {
        query((user_id(), fauna::name()))
            .requires((player(), fauna::fauna()))
            .build()
    };

    JoinRequest::subscribe(move |source, data| {
        let Some(player_entity) = source.client_entity_id() else { return };
        let Some(uid) = source.client_user_id() else { return };

        let name = data.name.trim().to_string();

        let deny_reason = if name.is_empty() {
            Some("Name must not be empty".to_string())
        } else if name.chars().count() > 32 {
            Some("Name must be 32 characters or less".to_string())
        } else {
            moderate_content(&name)
        };

        if let Some(deny_reason) = deny_reason {
            JoinDenied::new(deny_reason).send_client_targeted_reliable(uid);
            return;
        }

        AcceptJoin::new().send_client_targeted_reliable(uid);
        entity::add_component(player_entity, fauna::fauna(), ());
        entity::add_component(player_entity, fauna::name(), data.name);
    });

    spawn_query(fauna::name())
        .requires((player(), fauna::fauna()))
        .bind(move |entities| {
            for (_e, name) in entities {
                let content = format!("Player {} has joined the game", name);
                Announcement::new(content).send_local_broadcast(true);
            }
        });

    despawn_query(fauna::name())
        .requires((player(), fauna::fauna()))
        .bind(move |entities| {
            for (_e, name) in entities {
                let content = format!("Player {} has left the game", name);
                Announcement::new(content).send_local_broadcast(true);
            }
        });

    let players = make_player_query();
    Announcement::subscribe(move |source, data| {
        if source.local().is_none() {
            return;
        }

        for (_e, (uid, _name)) in players.evaluate() {
            data.send_client_targeted_reliable(uid);
        }
    });

    let players = make_player_query();
    PlayerMessage::subscribe(move |source, data| {
        let Some(player) = source.client_entity_id() else { return };
        let Some(uid) = source.client_user_id() else { return };
        let Some(name) = entity::get_component(player, fauna::name()) else { return };

        let deny_reason = if data.content.is_empty() {
            Some("empty chat message".to_string())
        } else {
            moderate_content(&data.content)
        };

        if let Some(deny_reason) = deny_reason {
            ChatDenied::new(deny_reason).send_client_targeted_reliable(uid);
            return;
        }

        let message = ChatMessage::new(name, data.content);
        for (_e, (uid, _name)) in players.evaluate() {
            message.send_client_targeted_reliable(uid);
        }
    });
}