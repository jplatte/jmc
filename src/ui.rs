use druid::{
    lens,
    widget::{Align, Button, Flex, Label, Maybe, Padding, Split, TextBox, ViewSwitcher},
    Widget, WidgetExt as _,
};
use druid_widget_nursery::WidgetExt as _;
use tracing::error;

use crate::data::{AppState, LoginState, UserState, View, LOGIN_TX};

pub mod actions;
mod room_view;
mod rooms_sidebar;

use self::{
    actions::{FINISH_LOGIN, SET_ACTIVE_ROOM},
    room_view::room_view,
    rooms_sidebar::rooms_sidebar,
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
