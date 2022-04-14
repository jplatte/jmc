use std::ops::Deref;

use druid::{
    widget::{Axis, Controller, Either, Flex, Label, List, RawLabel, Scroll, SizedBox, Spinner},
    Color, Target, Widget, WidgetExt,
};
use ruma::events::{
    room::message::SyncRoomMessageEvent, AnySyncMessageLikeEvent, AnySyncRoomEvent,
};
use tokio_stream::StreamExt;
use tracing::error;

use crate::{
    data::active_room::{
        ActiveRoomState, EventGroupState, EventOrTxnId, EventState, EventTypeState,
    },
    ui::actions::PREPEND_EVENT,
};

pub fn timeline() -> SizedBox<ActiveRoomState> {
    Scroll::new(
        Flex::column()
            .with_child(Either::new(
                |st: &ActiveRoomState, _| st.show_spinner,
                Spinner::new(),
                SizedBox::empty(),
            ))
            .with_child(
                List::new(make_timeline_event_group)
                    .with_spacing(2.0)
                    .expand_width()
                    .lens(ActiveRoomState::timeline),
            ),
    )
    .vertical()
    .controller(TimelineController)
    .expand_height()
}

fn make_timeline_event_group() -> impl Widget<EventGroupState> {
    Flex::row()
        .with_child(RawLabel::new().lens(EventGroupState::sender_display_name))
        .with_child(List::new(make_timeline_event).lens(EventGroupState::events))
}

fn make_timeline_event() -> impl Widget<EventState> {
    Label::new(|state: &EventState, _env: &_| match &state.event_type {
        EventTypeState::RoomMessage { display_string } => display_string.clone(),
    })
    .on_added(|label, _ctx, state, _env| {
        if let EventOrTxnId::TxnId(_) = state.id {
            label.set_text_color(Color::grey(0.5));
        }
    })
}

struct TimelineController;

impl<W> Controller<ActiveRoomState, Scroll<ActiveRoomState, W>> for TimelineController
where
    W: Widget<ActiveRoomState>,
{
    fn event(
        &mut self,
        child: &mut Scroll<ActiveRoomState, W>,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut ActiveRoomState,
        env: &druid::Env,
    ) {
        child.event(ctx, event, data, env);

        if !matches!(event, druid::Event::Wheel(_)) {
            return;
        }

        if child.offset_for_axis(Axis::Vertical) >= 500.0 {
            return;
        }

        // fetch room state
        data.show_spinner = true;

        let room = data.room().to_owned();
        let ui_handle = ctx.get_external_handle();

        tokio::spawn(async move {
            // TODO: Store the stream in ActiveRoomState, this is just a hack to see that
            //       things generally work
            let mut stream = Box::pin(room.timeline_backward().await.unwrap());

            let mut max_remaining = 40;

            while let Some(event) = stream.try_next().await.unwrap() {
                let event = match event.event.deserialize() {
                    Ok(AnySyncRoomEvent::MessageLike(AnySyncMessageLikeEvent::RoomMessage(
                        SyncRoomMessageEvent::Original(ev),
                    ))) => ev,
                    _ => continue,
                };

                let event = (room.room_id().into(), event.sender.deref().into(), event.into());
                if let Err(e) = ui_handle.submit_command(PREPEND_EVENT, event, Target::Auto) {
                    error!("{e}");
                }

                max_remaining -= 1;
                if max_remaining == 0 {
                    break;
                }
            }
        });
    }
}
