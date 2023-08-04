use std::f32::consts::FRAC_PI_2;

use ambient_api::{
    components::core::{
        app::name,
        layout::{
            self, align_horizontal_begin, align_vertical_end, docking_bottom, docking_left,
            fit_horizontal_children, fit_horizontal_none, fit_horizontal_parent,
            fit_vertical_children, fit_vertical_none, max_height, min_height, min_width,
            orientation_vertical, space_between_items, width, max_width,
        },
        rect::{background_color, line_from, line_to, line_width},
        rendering::color,
    },
    element::Setter,
    input::CursorLockGuard,
    messages::{Frame, WindowKeyboardInput},
    prelude::*,
};
use components::terrain::height;

mod shared;

use crate::{
    components::{fauna, map, player::local_player_ref, ui::*},
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
    let (joined, set_joined) = hooks.use_entity_component(entity::resources(), joined());

    hooks.use_module_message(move |_, _, _msg: &AcceptJoin| {
        set_joined(true);
    });

    if joined.unwrap_or(false) {
        GameUI::el()
    } else {
        JoinScreen::el()
    }
}

#[element_component]
fn GameUI(hooks: &mut Hooks) -> Element {
    Group::el([Crosshair::el(), Controls::el(), Chat::el()])
}

// TODO: either yoink a better crosshair from AFPS when it has one or make one ourselves and share
#[element_component]
fn Crosshair(hooks: &mut Hooks) -> Element {
    let size = hooks.use_window_logical_resolution();
    let center_x = size.x as f32 / 2.;
    let center_y = size.y as f32 / 2.;

    Group::el([
        Line.el()
            .with(line_from(), vec3(center_x - 10., center_y, 0.))
            .with(line_to(), vec3(center_x + 10., center_y, 0.))
            .with(line_width(), 2.)
            .with(background_color(), vec4(1., 1., 1., 1.)),
        Line.el()
            .with(line_from(), vec3(center_x, center_y - 10., 0.))
            .with(line_to(), vec3(center_x, center_y + 10., 0.))
            .with(line_width(), 2.)
            .with(background_color(), vec4(1., 1., 1., 1.)),
    ])
}

#[element_component]
fn Controls(hooks: &mut Hooks) -> Element {
    let (locked, set_locked) = hooks.use_state(false);

    hooks.consume_context::<Focus>();

    hooks.use_frame({
        let set_locked = set_locked.clone();
        move |_| {
            if locked {
                let (delta, _input) = input::get_delta();
                if delta.keys.contains(&KeyCode::Escape) {
                    eprintln!("escaping!");
                    input::set_cursor_lock(false);
                    input::set_cursor_visible(true);
                    set_locked(false);
                }
            }
        }
    });

    ClickArea::new(WindowSized::el([]))
        .on_mouse_down(move |_, _, _| {
            if !locked {
                eprintln!("clicked!");
                input::set_cursor_lock(true);
                input::set_cursor_visible(false);
                set_locked(true);
            }
        })
        .el()
}

#[element_component]
fn Chat(hooks: &mut Hooks) -> Element {
    let (message, set_message) = hooks.use_state("".to_string());
    let (messages, set_messages) = hooks.use_state(Vec::<String>::new());

    let content = Flow::el(messages.iter().map(|message| Text::el(message)))
        .with_default(orientation_vertical())
        .with_default(align_horizontal_begin())
        .with_default(align_vertical_end())
        .with_default(fit_horizontal_parent())
        .with_default(fit_vertical_children())
        .with(space_between_items(), STREET)
        .with_padding_even(STREET)
        .with(background_color(), Vec3::ZERO.extend(0.1));

    // let content = ScrollArea::el(ScrollAreaSizing::FitParentWidth, content);

    let editor = TextEditor::new(message, set_message.clone())
        .on_submit(move |new_message| {
            let mut messages = messages.clone();
            messages.push(new_message);
            set_messages(messages);
            set_message("".to_string());
        })
        .el()
        .with_default(fit_horizontal_parent())
        .with_padding_even(STREET)
        .with(min_height(), 40.0)
        .with(min_width(), 300.0);

    let window = Flow::el([content, editor])
        .with_default(orientation_vertical())
        .with_default(align_horizontal_begin())
        .with_default(align_vertical_end())
        .with_default(fit_horizontal_children())
        .with_default(fit_vertical_children())
        .with(min_width(), 600.0);

    FocusRoot::el([WindowSized::el([Dock::el([
        window.with_default(docking_bottom())
    ])])])
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
            .with(min_width(), 100.0)
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
