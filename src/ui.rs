pub mod actions;

use std::sync::Arc;

use druid::{
    lens,
    widget::{
        Align, Button, Flex, Label, List, Maybe, Padding, Scroll, Split, TextBox, ViewSwitcher,
    },
    Command, Target, Widget, WidgetExt as _,
};
use druid_widget_nursery::WidgetExt as _;
use tracing::error;

use self::actions::{
    ADD_EVENT, ADD_OR_UPDATE_ROOM, ADD_OR_UPDATE_ROOMS, FINISH_LOGIN, SET_ACTIVE_ROOM,
};
use crate::data::{
    ActiveRoomState, AppState, EventState, EventTypeState, LoginState, MinRoomState, UserState,
    View, LOGIN_TX,
};

pub(crate) fn build_ui() -> impl Widget<AppState> {
    ViewSwitcher::<AppState, _>::new(
        |state, _| state.view,
        |view, _, _| match view {
            View::Login => login_screen().lens(AppState::login_state).boxed(),
            View::Loading => loading().lens(lens::Unit).boxed(),
            View::LoggedIn => main_ui().lens(AppState::user_state).boxed(),
        },
    )
    .on_command(FINISH_LOGIN, |ctx, _, state| {
        state.view = View::LoggedIn;
        state.login_state = Default::default();
        ctx.set_handled();
    })
}

fn login_screen() -> impl Widget<LoginState> {
    Padding::new(
        12.0,
        Flex::column()
            .with_child(Align::left(Label::new("Login").with_text_size(24.0)))
            .with_child(TextBox::new().with_placeholder("User ID").lens(LoginState::user_id))
            .with_child(
                TextBox::protected().with_placeholder("Password").lens(LoginState::password),
            )
            .with_child(Button::<LoginState>::new("Log in").on_click(|_, state, env| {
                let send_res = env.get(LOGIN_TX).try_send(state.to_owned());

                if let Err(e) = send_res {
                    error!("Sending login data failed: {}", e);
                }
            })),
    )
}

fn loading() -> impl Widget<()> {
    druid::widget::Spinner::new()
}

fn main_ui() -> impl Widget<UserState> {
    let right_pane = Maybe::new(room_view, || Label::new("<no room selected>")).on_command(
        SET_ACTIVE_ROOM,
        |_ctx, room_state, active_room| {
            *active_room = Some(room_state.into());
        },
    );

    Split::columns(rooms_sidebar(), right_pane.lens(UserState::active_room))
        .min_size(200.0, 400.0)
        .split_point(0.0)
        .solid_bar(true)
        .bar_size(1.0)
}

fn rooms_sidebar() -> impl Widget<UserState> {
    Scroll::new(
        Flex::column()
            .with_child(Label::new("<sidebar>"))
            .with_child(List::new(make_room_item).with_spacing(6.0).lens(UserState::rooms)),
    )
    .vertical()
    .on_command(ADD_OR_UPDATE_ROOM, |_ctx, room_state, state| {
        state.rooms.insert(room_state.id.clone(), room_state.clone());
    })
    .on_command(ADD_OR_UPDATE_ROOMS, |_ctx, rooms, state| {
        state
            .rooms
            .extend(rooms.iter().map(|room_state| (room_state.id.clone(), room_state.clone())));
    })
}

fn make_room_item() -> impl Widget<MinRoomState> {
    Label::new(|state: &MinRoomState, _env: &_| state.display_name.clone()).on_click(
        |ctx, state, _env| {
            ctx.submit_command(Command::new(SET_ACTIVE_ROOM, state.clone(), Target::Auto))
        },
    )
}

fn room_view() -> impl Widget<ActiveRoomState> {
    Flex::column()
        .with_child(Label::new(|state: &ActiveRoomState, _env: &_| state.display_name.clone()))
        .with_child(Scroll::new(
            List::new(make_timeline_item).with_spacing(2.0).lens(ActiveRoomState::timeline),
        ))
        .with_child(message_input_area().lens(ActiveRoomState::message_input))
        .on_command(ADD_EVENT, |_ctx, (room_id, event), state| {
            if *state.id == **room_id {
                state.timeline.insert(event.event_id.clone(), event.clone());
            }
        })
}

fn make_timeline_item() -> impl Widget<EventState> {
    Label::new(|state: &EventState, _env: &_| match &state.event_type {
        EventTypeState::RoomMessage { display_string } => display_string.clone(),
    })
}

fn message_input_area() -> impl Widget<Arc<String>> {
    TextBox::new()
}
