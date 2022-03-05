use druid::{
    widget::{Flex, Label, List, RawLabel, Scroll},
    Color, Widget, WidgetExt,
};

use crate::data::active_room::{
    ActiveRoomState, EventGroupState, EventOrTxnId, EventState, EventTypeState,
};

pub fn timeline() -> druid::widget::SizedBox<ActiveRoomState> {
    Scroll::new(
        List::new(make_timeline_event_group)
            .with_spacing(2.0)
            .expand_width()
            .lens(ActiveRoomState::timeline),
    )
    .vertical()
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
