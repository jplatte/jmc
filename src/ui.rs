pub mod actions;

use druid::{
    lens,
    widget::{Align, Button, Flex, Label, Padding, Split, TextBox, ViewSwitcher},
    LensExt as _, Widget, WidgetExt as _,
};
use druid_widget_nursery::WidgetExt as _;

use self::actions::FINISH_LOGIN;
use crate::data::{AppState, LoginState, View};

pub(crate) fn build_ui() -> impl Widget<AppState> {
    ViewSwitcher::<AppState, _>::new(
        |state, _| state.view,
        |view, _, _| match view {
            View::Login => login_screen().boxed(),
            View::Loading => loading().lens(lens::Unit).boxed(),
            View::Main => main_ui().boxed(),
        },
    )
    .on_command(FINISH_LOGIN, |ctx, user_data, state| {
        state.view = View::Main;
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

fn loading() -> impl Widget<()> {
    druid::widget::Spinner::new()
}

fn main_ui() -> impl Widget<AppState> {
    Split::columns(rooms_sidebar(), room_view())
        .min_size(42.0, 400.0)
        .split_point(0.0)
        .bar_size(0.0)
}

fn rooms_sidebar() -> impl Widget<AppState> {
    Label::new("S\nI\nD\nE\nB\nA\nR")
}

fn room_view() -> impl Widget<AppState> {
    Label::new("<room view>")
}
