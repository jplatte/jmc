use druid::{
    im::vector,
    text::{Attribute, RichText},
    widget::{Button, Flex, Label, Maybe, Padding, TextBox},
    FontWeight, Target, Widget, WidgetExt as _,
};
use druid_widget_nursery::WidgetExt as _;
use matrix_sdk::{ruma::events::room::message::RoomMessageEventContent, uuid::Uuid};
use tracing::error;

use super::actions::{ADD_EVENT, REMOVE_EVENT};
use crate::data::{
    ActiveRoomState, EventGroupState, EventOrTxnId, EventState, EventTypeState, JoinedRoomState,
    UuidWrap,
};

mod timeline;

use self::timeline::timeline;

pub fn room_view() -> impl Widget<ActiveRoomState> {
    Flex::column()
        .with_child(Label::new(|state: &ActiveRoomState, _env: &_| state.display_name.clone()))
        .with_flex_child(timeline(), 1.0)
        .with_child(message_input_area())
        .on_command(ADD_EVENT, |_ctx, (room_id, sender, event), state| {
            if *state.id == *room_id {
                // FIXME: Use if-let-chain once possible
                match state.timeline.back_mut() {
                    Some(group) if *sender == group.sender => {
                        group.events.push_back(event.clone());
                    }
                    _ => {
                        let event_group_state = EventGroupState {
                            sender: sender.clone(),
                            // FIXME: Get display name from ADD_EVENT
                            sender_display_name: RichText::new(sender.as_str().into())
                                .with_attribute(.., Attribute::Weight(FontWeight::SEMI_BOLD)),
                            // FIXME: Put in last group if same sender
                            events: vector![event.clone()],
                        };

                        state.timeline.push_back(event_group_state);
                    }
                }
            }
        })
        .on_command(REMOVE_EVENT, |_ctx, id, state| {
            let evt_group_evt_idx = state.timeline.iter().enumerate().find_map(|(idx1, group)| {
                let idx2 = group.events.iter().position(|ev| ev.id == *id)?;
                Some((idx1, idx2))
            });

            if let Some((group_idx, evt_idx)) = evt_group_evt_idx {
                let events = &mut state.timeline[group_idx].events;

                if events.len() == 1 {
                    // If this is the only event, remove the whole group.
                    state.timeline.remove(group_idx);
                } else {
                    // Otherwise, keep the group (only remove the event from it).
                    events.remove(evt_idx);
                }
            } else {
                error!("Can't remove event {:?}", id);
            }
        })
}

fn message_input_area() -> impl Widget<ActiveRoomState> {
    Maybe::new(active_input_area, || Label::new("invited and left rooms are not yet supported"))
        .lens(ActiveRoomState::kind)
}

fn active_input_area() -> impl Widget<JoinedRoomState> {
    Padding::new(
        (10.0, 6.0),
        Flex::row()
            .with_flex_child(
                TextBox::new()
                    .with_placeholder("Send message…")
                    .expand_width()
                    .lens(JoinedRoomState::message_input),
                1.0,
            )
            .with_default_spacer()
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
                    let event = (room.room_id().into(), room.own_user_id().into(), event_state);

                    if let Err(e) = ui_handle.submit_command(ADD_EVENT, event, Target::Auto) {
                        error!("{}", e);
                    }

                    // FIXME: Handle error
                    let _ = room.send(msg, Some(txn_id)).await;
                });
            })),
    )
}
