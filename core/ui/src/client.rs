use std::f32::consts::{FRAC_PI_2, TAU};

use ambient_api::{
    core::{
        layout::components::*,
        rect::components::{background_color, line_from, line_to, line_width},
        rendering::components::color,
    },
    input::{Input, InputDelta},
    prelude::*,
};

mod shared;

use packages::{
    actions::messages::{PerformCraftingAction, PerformSwap, PerformTileAction},
    crops::components::medium_crop_occupant,
    fauna::components::{is_mod_loaded as is_fauna_loaded, pitch, yaw},
    map::components::{
        chunk, chunk_tile_index, chunk_tile_refs, in_chunk, is_mod_loaded as is_map_loaded,
        position,
    },
    this::{components::*, messages::*},
};

#[main]
fn main() {
    run_async(async_main());
}

async fn async_main() {
    eprintln!("UI mod loaded, waiting for fauna and map mods");
    entity::wait_for_component(entity::resources(), is_fauna_loaded()).await;
    entity::wait_for_component(entity::resources(), is_map_loaded()).await;
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
fn GameUI(_hooks: &mut Hooks) -> Element {
    Group::el([Crosshair::el(), Controls::el(), Chat::el(), Status::el()])
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
                let (delta, input) = input::get_delta();
                if delta.keys.contains(&KeyCode::Escape) {
                    eprintln!("escaping!");
                    input::set_cursor_lock(false);
                    input::set_cursor_visible(true);
                    set_locked(false);
                } else {
                    update_controls(delta, input);
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

fn update_controls(delta: InputDelta, input: Input) {
    use packages::player::components::*;

    let local_player_entity = entity::get_component(entity::resources(), local_player_ref())
        .expect("local_player_ref resource was deleted");

    let pitch_factor = 0.01;
    let yaw_factor = 0.01;

    entity::mutate_component_with_default(local_player_entity, yaw(), 0.0, |yaw| {
        *yaw = (*yaw + input.mouse_delta.x * yaw_factor) % TAU;
    });

    entity::mutate_component_with_default(local_player_entity, pitch(), 0.0, |pitch| {
        *pitch = (*pitch + input.mouse_delta.y * pitch_factor).clamp(-FRAC_PI_2, FRAC_PI_2);
    });

    entity::mutate_component_with_default(
        local_player_entity,
        direction(),
        Vec2::ZERO,
        |direction| {
            *direction = Vec2::ZERO;
            if input.keys.contains(&KeyCode::W) {
                direction.y -= 1.0;
            }
            if input.keys.contains(&KeyCode::S) {
                direction.y += 1.0;
            }
            if input.keys.contains(&KeyCode::A) {
                direction.x -= 1.0;
            }
            if input.keys.contains(&KeyCode::D) {
                direction.x += 1.0;
            }

            *direction = direction.clamp_length_max(1.0);
        },
    );

    if delta.keys.contains(&KeyCode::Q) {
        PerformCraftingAction::new().send_local_broadcast(false);
    }

    if delta.keys.contains(&KeyCode::F) {
        PerformSwap::new().send_local_broadcast(false);
    }

    if delta.mouse_buttons.contains(&MouseButton::Left) {
        if let Some(selected) = entity::get_component(player::get_local(), tile_selection_ref()) {
            let chunk_ref = entity::get_component(selected, in_chunk()).unwrap();
            let chunk_pos = entity::get_component(chunk_ref, chunk()).unwrap();
            let tile_idx = entity::get_component(selected, chunk_tile_index()).unwrap();
            let tiles = entity::get_component(chunk_ref, chunk_tile_refs()).unwrap();
            let tile = tiles[tile_idx as usize];

            let on_occupant = entity::get_component(tile, medium_crop_occupant())
                .unwrap_or_default()
                != EntityId::null();

            PerformTileAction {
                chunk_pos,
                tile_idx,
                on_occupant,
            }
            .send_local_broadcast(false);
        }
    }
}

#[element_component]
fn Chat(hooks: &mut Hooks) -> Element {
    #[derive(Clone, Debug)]
    struct MessageContent {
        author: Option<String>,
        content: String,
    }

    impl MessageContent {
        fn render(&self) -> Element {
            if let Some(author) = self.author.as_ref() {
                Text::el(format!("{}: {}", author, self.content))
                    .with(color(), Vec3::splat(0.8).extend(1.0))
            } else {
                Text::el(&self.content).with(color(), Vec3::splat(0.5).extend(1.0))
            }
        }
    }

    let (message, set_message) = hooks.use_state("".to_string());
    let (deny_reason, set_deny_reason) = hooks.use_state("".to_string());
    let (messages, set_messages) = hooks.use_state(Vec::<MessageContent>::new());

    if messages.len() > 10 {
        let len = messages.len();
        let start = len - 10;
        set_messages(messages[start..].to_owned());
    }

    hooks.use_module_message({
        let messages = messages.clone();
        let set_messages = set_messages.clone();
        move |_, _, data: &Announcement| {
            let new_message = MessageContent {
                author: None,
                content: data.content.clone(),
            };

            let mut messages = messages.clone();
            messages.push(new_message);
            set_messages(messages);
        }
    });

    hooks.use_module_message({
        let messages = messages.clone();
        let set_messages = set_messages.clone();
        move |_, _, data: &ChatMessage| {
            let new_message = MessageContent {
                author: Some(data.author.clone()),
                content: data.content.clone(),
            };

            let mut messages = messages.clone();
            messages.push(new_message);
            set_messages(messages);
        }
    });

    hooks.use_module_message({
        let set_deny_reason = set_deny_reason.clone();
        move |_, _, data: &ChatDenied| {
            set_deny_reason(data.reason.clone());
        }
    });

    let content = with_rect(
        Flow::el(messages.iter().map(MessageContent::render))
            .with(orientation(), Orientation::Vertical)
            .with(align_horizontal(), Align::Begin)
            .with(align_vertical(), Align::End)
            .with(fit_horizontal(), Fit::Parent)
            .with(fit_vertical(), Fit::Children)
            .with(space_between_items(), STREET)
            .with_padding_even(STREET),
    )
    .with(background_color(), Vec3::ZERO.extend(0.7));

    // let content = ScrollArea::el(ScrollAreaSizing::FitParentWidth, content);

    let editor = TextEditor::new(message, set_message.clone())
        .on_submit(move |new_message| {
            PlayerMessage::new(new_message).send_server_reliable();
            set_message("".to_string());
            set_deny_reason("".to_string());
        })
        .el()
        .with(fit_horizontal(), Fit::Parent);

    let entry = if deny_reason.is_empty() {
        FlowColumn::el([editor])
    } else {
        let deny = Text::el(deny_reason).error_text_style();
        FlowColumn::el([editor, deny])
    };

    let entry = with_rect(
        entry
            .with(space_between_items(), STREET)
            .with_padding_even(STREET)
            .with(min_width(), 600.0),
    )
    .with(background_color(), app_background_color().into());

    let window = Flow::el([content, entry])
        .with(orientation(), Orientation::Vertical)
        .with(align_horizontal(), Align::Begin)
        .with(align_vertical(), Align::End)
        .with(fit_horizontal(), Fit::Children)
        .with(fit_vertical(), Fit::Children)
        .with(min_width(), 600.0);

    FocusRoot::el([WindowSized::el([Dock::el([
        window.with(docking(), Docking::Bottom)
    ])])])
}

#[element_component]
fn Status(hooks: &mut Hooks) -> Element {
    let (coords, _) = hooks.use_entity_component(player::get_local(), position());

    let coords = coords
        .map(|coords| format!("Map position: ({:.1}, {:.1})", coords.x, coords.y))
        .unwrap_or_default();

    FlowColumn::el([Text::el(coords)])
        .with_padding_even(10.0)
        .with_background(Vec3::ZERO.extend(0.8))
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
