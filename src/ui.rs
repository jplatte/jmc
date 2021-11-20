pub mod actions;

use druid::{
    lens,
    widget::{Align, Button, Flex, Label, List, Padding, Scroll, Split, TextBox, ViewSwitcher},
    LensExt as _, Widget, WidgetExt as _,
};
use druid_widget_nursery::WidgetExt as _;
use tracing::error;

use self::actions::{ADD_OR_UPDATE_ROOM, ADD_OR_UPDATE_ROOMS, FINISH_LOGIN};
use crate::data::{AppState, LoginState, RoomState, UserState, View, LOGIN_TX};

pub(crate) fn build_ui() -> impl Widget<AppState> {
    ViewSwitcher::<AppState, _>::new(
        |state, _| state.view,
        |view, _, _| match view {
            View::Login => login_screen().lens(AppState::login_state).boxed(),
            View::Loading => loading().lens(lens::Unit).boxed(),
            View::Main => main_ui()
                .lens(AppState::user_state.map(
                    |state| state.clone().unwrap(),
                    |state_a, state_b| *state_a = Some(state_b),
                ))
                .boxed(),
        },
    )
    .on_command(FINISH_LOGIN, |ctx, user_data, state| {
        state.view = View::Main;
        state.login_state = Default::default();
        state.user_state = Some(user_data.into());
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
    Split::columns(rooms_sidebar(), room_view())
        .min_size(200.0, 400.0)
        .split_point(0.0)
        .bar_size(0.0)
}

fn rooms_sidebar() -> impl Widget<UserState> {
    Flex::column()
        .with_child(Label::new("<sidebar>"))
        .with_child(Scroll::new(List::new(make_room_item).lens(UserState::rooms)).vertical())
        .on_command(ADD_OR_UPDATE_ROOM, |_ctx, room_state, state| {
            state.rooms.insert(room_state.id.clone(), room_state.clone());
        })
        .on_command(ADD_OR_UPDATE_ROOMS, |_ctx, rooms, state| {
            state
                .rooms
                .extend(rooms.iter().map(|room_state| (room_state.id.clone(), room_state.clone())));
        })
}

fn make_room_item() -> impl Widget<RoomState> {
    Label::new(|state: &RoomState, _env: &_| state.display_name.clone())
}

fn room_view() -> impl Widget<UserState> {
    Label::new("<room view>")
}
