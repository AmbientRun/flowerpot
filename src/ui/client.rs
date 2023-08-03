use std::f32::consts::FRAC_PI_2;

use ambient_api::{
    components::core::{
        app::name,
        layout::{max_height, min_height, space_between_items},
        rendering::color,
    },
    element::Setter,
    messages::Frame,
    prelude::*,
};

mod shared;

use crate::{
    components::{fauna, map, player::local_player_ref},
    messages::{
        AcceptJoin, JoinDenied, JoinRequest, ReleaseInput, RequestInput, UpdatePlayerAngle,
    },
};

#[main]
fn main() {
    run_async(async_main());
}

async fn async_main() {
    let local_player_entity = entity::get_component(entity::resources(), local_player_ref())
        .expect("local_player_ref resource was deleted");

    /*let mut yaw = 0.0;
    let mut pitch = 0.0;
    Frame::subscribe(move |_| {
        let (delta, input) = input::get_delta();

        let pitch_factor = 0.01;
        let yaw_factor = 0.01;
        yaw = (yaw + input.mouse_delta.x * yaw_factor) % TAU;
        pitch = (pitch + input.mouse_delta.y * pitch_factor).clamp(-FRAC_PI_2, FRAC_PI_2);

        UpdatePlayerAngle::new(pitch, yaw).send_server_reliable();

        use components::player::{pitch as pitch_component, yaw as yaw_component};
        entity::add_component(local_player_entity, yaw_component(), yaw);
        entity::add_component(local_player_entity, pitch_component(), pitch);

        let mut new_direction = Vec2::ZERO;
        if input.keys.contains(&KeyCode::W) {
            new_direction.y -= 1.0;
        }
        if input.keys.contains(&KeyCode::S) {
            new_direction.y += 1.0;
        }
        if input.keys.contains(&KeyCode::A) {
            new_direction.x -= 1.0;
        }
        if input.keys.contains(&KeyCode::D) {
            new_direction.x += 1.0;
        }

        let new_direction = new_direction.clamp_length_max(1.0);
        entity::add_component(local_player_entity, direction(), new_direction);
        UpdatePlayerDirection::new(new_direction).send_server_reliable();

        if delta.keys.contains(&KeyCode::Q) {
            PerformCraftingAction::new().send_local_broadcast(true);
        }
    });*/

    eprintln!("UI mod loaded, waiting for fauna and map mods");
    entity::wait_for_component(entity::resources(), fauna::mod_loaded()).await;
    entity::wait_for_component(entity::resources(), map::mod_loaded()).await;
    eprintln!("UI, map, and fauna mods loaded; showing UI");

    App.el().spawn_interactive();
}

#[element_component]
fn App(hooks: &mut Hooks) -> Element {
    let (joined, set_joined) = hooks.use_state(false);

    hooks.use_module_message(move |_, _, _msg: &AcceptJoin| {
        set_joined(true);
    });

    if joined {
        FocusRoot::el([])
    } else {
        JoinScreen::el()
    }
}

#[element_component]
fn JoinScreen(hooks: &mut Hooks) -> Element {
    use_input_request(hooks);

    let (name, set_name) = hooks.use_state("".to_string());
    let (denied_reason, set_denied_reason) = hooks.use_state("".to_string());

    hooks.use_module_message(move |_, _, msg: &JoinDenied| {
        set_denied_reason(msg.reason.clone());
    });

    FocusRoot::el([WindowSized::el([FlowColumn::el([
        Text::el("Flowerpot").header_style(),
        Separator { vertical: false }.el(),
        Text::el("Enter your name below. Press enter to join the game."),
        TextEditor::new(name.clone(), set_name.clone())
            .auto_focus()
            .on_submit(|name| JoinRequest::new(name).send_server_reliable())
            .el()
            .with(min_height(), 16.0)
            .with(max_height(), 100.0),
        Text::el(denied_reason).with(color(), vec4(1.0, 0.6, 0.6, 1.0)),
        Separator { vertical: false }.el(),
    ])
    .with(space_between_items(), STREET)])
    .with_padding_even(20.0)])
}

fn use_input_request(hooks: &mut Hooks) {
    hooks.use_spawn(|_| {
        RequestInput::new().send_local_broadcast(true);
        |_| {
            ReleaseInput::new().send_local_broadcast(true);
        }
    });
}
