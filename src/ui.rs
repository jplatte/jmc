use druid::{
    widget::{Align, Flex, Label, Padding, TextBox},
    Widget, WidgetExt,
};

use crate::AppState;

pub(crate) fn build_ui() -> impl Widget<AppState> {
    login_screen()
}

fn login_screen() -> impl Widget<AppState> {
    Padding::new(
        12.0,
        Flex::column()
            .with_child(Align::left(Label::new("Login").with_text_size(24.0)))
            .with_child(
                TextBox::new()
                    .with_placeholder("User ID")
                    .lens(AppState::user_id),
            )
            .with_child(
                TextBox::protected()
                    .with_placeholder("Password")
                    .lens(AppState::password),
            ),
    )
}
