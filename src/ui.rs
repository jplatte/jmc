pub mod actions;

use druid::{
    lens,
    widget::{
        Align, Button, Controller, Flex, Image, Label, List, Maybe, Padding, Scroll, Split,
        TextBox, ViewSwitcher,
    },
    Color, Command, ImageBuf, Size, Target, Widget, WidgetExt as _,
};
use druid_widget_nursery::WidgetExt as _;
use matrix_sdk::{ruma::events::room::message::RoomMessageEventContent, uuid::Uuid};
use tracing::error;

use self::actions::{ADD_EVENT, ADD_OR_UPDATE_ROOM, FINISH_LOGIN, REMOVE_EVENT, SET_ACTIVE_ROOM};
use crate::data::{
    ActiveRoomState, AppState, EventOrTxnId, EventState, EventTypeState, JoinedRoomState,
    LoginState, MinRoomState, UserState, UuidWrap, View, LOGIN_TX,
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
        .min_size(32.0, 400.0)
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
}

fn make_room_item() -> impl Widget<MinRoomState> {
    Image::new(ImageBuf::empty())
        .controller(RoomItemController)
        //.on_added(|image, _ctx, state: &MinRoomState, _env| {
        //    image.set_image_data(state.icon.clone());
        //})
        .on_click(|ctx, state, _env| {
            ctx.submit_command(Command::new(SET_ACTIVE_ROOM, state.clone(), Target::Auto))
        })
    // FIXME: Tooltip widget is rather broken (not positioned correctly, can be focused)
    //.tooltip(|state: &MinRoomState, _env: &_| state.display_name.clone())
}

struct RoomItemController;

impl Controller<MinRoomState, Image> for RoomItemController {
    fn lifecycle(
        &mut self,
        child: &mut Image,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &MinRoomState,
        env: &druid::Env,
    ) {
        if let druid::LifeCycle::WidgetAdded = event {
            child.set_image_data(data.icon.clone());
        }

        child.lifecycle(ctx, event, data, env);
    }

    fn update(
        &mut self,
        child: &mut Image,
        ctx: &mut druid::UpdateCtx,
        old_data: &MinRoomState,
        data: &MinRoomState,
        env: &druid::Env,
    ) {
        if data.icon.size() != Size::ZERO {
            child.set_image_data(data.icon.clone());
        }

        child.update(ctx, old_data, data, env);
    }
}

fn room_view() -> impl Widget<ActiveRoomState> {
    Flex::column()
        .with_child(Label::new(|state: &ActiveRoomState, _env: &_| state.display_name.clone()))
        .with_child(Scroll::new(
            List::new(make_timeline_item).with_spacing(2.0).lens(ActiveRoomState::timeline),
        ))
        .with_child(message_input_area())
        .on_command(ADD_EVENT, |_ctx, (room_id, event), state| {
            if *state.id == *room_id {
                state.timeline.push_back(event.to_owned());
            }
        })
        .on_command(REMOVE_EVENT, |_ctx, id, state| {
            if let Some(idx) = state.timeline.iter().position(|ev| ev.id == *id) {
                state.timeline.remove(idx);
            } else {
                error!("Can't remove event {:?}", id);
            }
        })
}

fn make_timeline_item() -> impl Widget<EventState> {
    Label::new(|state: &EventState, _env: &_| match &state.event_type {
        EventTypeState::RoomMessage { display_string } => display_string.clone(),
    })
    .on_added(|label, _ctx, state, _env| {
        if let EventOrTxnId::TxnId(_) = state.id {
            label.set_text_color(Color::grey(0.5));
        }
    })
}

fn message_input_area() -> impl Widget<ActiveRoomState> {
    Maybe::new(active_input_area, || Label::new("invited and left rooms are not yet supported"))
        .lens(ActiveRoomState::kind)
}

fn active_input_area() -> impl Widget<JoinedRoomState> {
    Flex::row()
        .with_child(
            TextBox::new().with_placeholder("Send message…").lens(JoinedRoomState::message_input),
        )
        .with_child(Button::<JoinedRoomState>::new("➤").on_click(|ctx, state, _env| {
            let room = state.room.clone();
            let message_input = state.message_input.clone();
            let ui_handle = ctx.get_external_handle();

            tokio::spawn(async move {
                let msg = RoomMessageEventContent::text_markdown(message_input.as_str());
                let display_string = msg.body().into();

                let txn_id = Uuid::new_v4();

                let event_state = EventState {
                    id: EventOrTxnId::TxnId(UuidWrap(txn_id)),
                    event_type: EventTypeState::RoomMessage { display_string },
                };
                let event = (room.room_id().into(), event_state);

                if let Err(e) = ui_handle.submit_command(ADD_EVENT, event, Target::Auto) {
                    error!("{}", e);
                }

                // FIXME: Handle error
                let _ = room.send(msg, Some(txn_id)).await;
            });
        }))
}
