use std::mem;

use druid::{
    lens,
    widget::{Align, Flex, Label, Padding, TextBox, ViewSwitcher},
    Widget, WidgetExt as _,
};
use druid_widget_nursery::WidgetExt as _;

use crate::{AppState, LoginState, UserState, FINISH_LOGIN};

pub(crate) fn build_ui() -> impl Widget<AppState> {
    ViewSwitcher::new(
        |state, _| mem::discriminant(state),
        |_, state, _| match state {
            AppState::LoggedOut(_) => login_screen()
                .lens(lens::Map::new(
                    |state| match state {
                        AppState::LoggedOut(st) => st.clone(),
                        _ => unreachable!(),
                    },
                    |state, login_state| *state = AppState::LoggedOut(login_state),
                ))
                .boxed(),
            AppState::LoggedIn(_) => main_ui()
                .lens(lens::Map::new(
                    |state| match state {
                        AppState::LoggedIn(st) => st.clone(),
                        _ => unreachable!(),
                    },
                    |state, user_state| *state = AppState::LoggedIn(user_state),
                ))
                .boxed(),
        },
    )
    .on_command(FINISH_LOGIN, |ctx, _, state| {
        *state = AppState::LoggedIn(UserState {});
        ctx.set_handled();
    })
}

fn login_screen() -> impl Widget<LoginState> {
    Padding::new(
        12.0,
        Flex::column()
            .with_child(Align::left(Label::new("Login").with_text_size(24.0)))
            .with_child(
                TextBox::new()
                    .with_placeholder("User ID")
                    .lens(LoginState::user_id),
            )
            .with_child(
                TextBox::protected()
                    .with_placeholder("Password")
                    .lens(LoginState::password),
            ),
    )
}

fn main_ui() -> impl Widget<UserState> {
    Label::new("Logged in.")
}
