use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use ambient_api::{
    core::player::components::{is_player, user_id},
    prelude::*,
};

mod shared;

use packages::{
    fauna::components::{is_fauna, name},
    ui::messages::*,
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
        query((user_id(), name()))
            .requires((is_player(), is_fauna()))
            .build()
    };

    let names_to_players: Arc<Mutex<HashMap<String, EntityId>>> = Default::default();

    JoinRequest::subscribe({
        let names_to_players = names_to_players.clone();
        move |source, data| {
            let Some(player_entity) = source.client_entity_id() else { return };
            let Some(uid) = source.client_user_id() else { return };

            let new_name = data.name.trim().to_string();
            let mut names_to_players = names_to_players.lock().unwrap();

            let deny_reason = if new_name.is_empty() {
                Some("Name must not be empty".to_string())
            } else if new_name.chars().count() > 32 {
                Some("Name must be 32 characters or less".to_string())
            } else if let Some(moderated) = moderate_content(&new_name) {
                Some(moderated)
            } else if names_to_players.contains_key(&new_name) {
                Some("Name is taken by another player".to_string())
            } else {
                None
            };

            if let Some(deny_reason) = deny_reason {
                JoinDenied::new(deny_reason).send_client_targeted_reliable(uid);
                return;
            }

            AcceptJoin::new().send_client_targeted_reliable(uid);
            entity::add_component(player_entity, is_fauna(), ());
            entity::add_component(player_entity, name(), new_name.clone());

            let content = format!("Player {} has joined the game", new_name);
            Announcement::new(content).send_local_broadcast(true);

            names_to_players.insert(new_name, player_entity);
        }
    });

    despawn_query(name())
        .requires((is_player(), is_fauna()))
        .bind({
            let names_to_players = names_to_players.clone();
            move |entities| {
                let mut names_to_players = names_to_players.lock().unwrap();
                for (_e, name) in entities {
                    let content = format!("Player {} has left the game", name);
                    Announcement::new(content).send_local_broadcast(true);
                    names_to_players.remove(&name);
                }
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
        let Some(name) = entity::get_component(player, name()) else { return };

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
