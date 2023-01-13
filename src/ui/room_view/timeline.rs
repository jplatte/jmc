use druid::{
    widget::{Axis, Controller, Either, Flex, Label, List, RawLabel, Scroll, SizedBox, Spinner},
    Color, Widget, WidgetExt,
};
use matrix_sdk::room::timeline::PaginationOptions;
use tracing::error;

use crate::data::active_room::{
    ActiveRoomState, EventGroupState, EventOrTxnId, EventState, EventTypeState,
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

        let sdk_timeline = data.sdk_timeline.clone();
        tokio::spawn(async move {
            if let Err(e) =
                sdk_timeline.paginate_backwards(PaginationOptions::single_request(25)).await
            {
                error!("{e}");
            }
        });
    }
}
