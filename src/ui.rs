use druid::{
    widget::{Align, Button, Flex, Label, Maybe, Padding, Split, TextBox},
    Widget, WidgetExt as _,
};
use druid_widget_nursery::{enum_switcher::LazySwitcher, WidgetExt as _};
use tracing::error;

use crate::data::{
    AppState, AppStateLoggedIn, AppStateLoggingIn, AppStateLogin, LoginState, UserState, LOGIN_TX,
};

pub mod actions;
mod room_view;
mod rooms_sidebar;

use self::{
    actions::{FINISH_LOGIN, SET_ACTIVE_ROOM},
    room_view::room_view,
    rooms_sidebar::rooms_sidebar,
};

pub(crate) fn build_ui() -> impl Widget<AppState> {
    LazySwitcher::new()
        .with_variant(AppStateLogin, login_screen)
        .with_variant(AppStateLoggingIn, loading)
        .with_variant(AppStateLoggedIn, main_ui)
        .on_command(FINISH_LOGIN, |_ctx, _user_data, state| {
            *state = AppState::LoggedIn(UserState::default());
        })
}

fn login_screen() -> impl Widget<LoginState> {
    Padding::new(
        12.0,
        Flex::column()
            .with_child(Align::left(Label::new("Login").with_text_size(24.0)))
            .with_child(TextBox::new().with_placeholder("User ID").lens(LoginState::user_id))
            .with_child(
                // temporarily a regular friggin text box to get this thing to compile at all
                // druid will be replaced xilem soon (or this project will be archived)
                TextBox::new().with_placeholder("Password").lens(LoginState::password),
            )
            .with_child(Button::new("Log in").on_click(|_, state: &mut LoginState, env| {
                let send_res = env.get(LOGIN_TX).try_send(state.to_owned());

                if let Err(e) = send_res {
                    error!("Sending login data failed: {e}");
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
