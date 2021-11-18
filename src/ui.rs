pub mod actions;

use druid::{
    widget::{Align, Button, Flex, Label, Padding, Split, TextBox, ViewSwitcher},
    LensExt as _, Widget, WidgetExt as _,
};
use druid_widget_nursery::WidgetExt as _;

use self::actions::FINISH_LOGIN;
use crate::data::{AppState, LoginState};

pub(crate) fn build_ui() -> impl Widget<AppState> {
    ViewSwitcher::<AppState, _>::new(
        |state, _| state.user_state.is_some(),
        |&logged_in, _, _| {
            if logged_in {
                main_ui().boxed()
            } else {
                login_screen().boxed()
            }
        },
    )
    .on_command(FINISH_LOGIN, |ctx, user_data, state| {
        state.login_state = Default::default();
        state.user_state = Some(user_data.into());
        ctx.set_handled();
    })
}

fn login_screen() -> impl Widget<AppState> {
    Padding::new(
        12.0,
        Flex::column()
            .with_child(Align::left(Label::new("Login").with_text_size(24.0)))
            .with_child(
                TextBox::new()
                    .with_placeholder("User ID")
                    .lens(AppState::login_state.then(LoginState::user_id)),
            )
            .with_child(
                TextBox::protected()
                    .with_placeholder("Password")
                    .lens(AppState::login_state.then(LoginState::password)),
            )
            .with_child(Button::<AppState>::new("Log in").on_click(|_, state, _| {
                state.login();
            })),
    )
}

fn main_ui() -> impl Widget<AppState> {
    Split::columns(rooms_sidebar(), room_view())
}

fn rooms_sidebar() -> impl Widget<AppState> {
    Label::new("S\nI\nD\nE\nB\nA\nR")
}

fn room_view() -> impl Widget<AppState> {
    Label::new("<room view>")
}
