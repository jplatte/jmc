use druid::{
    widget::{Controller, Image, Label, List, Scroll},
    Size, Target, Widget, WidgetExt as _,
};
use druid_widget_nursery::WidgetExt as _;
use tracing::error;

use crate::data::{MinRoomState, UserState};

use super::actions::{NewActiveRoomState, ADD_OR_UPDATE_ROOM, SET_ACTIVE_ROOM};

pub fn rooms_sidebar() -> impl Widget<UserState> {
    Scroll::new(List::new(make_room_item).with_spacing(6.0).lens(UserState::rooms))
        .vertical()
        .on_command(ADD_OR_UPDATE_ROOM, |_ctx, room_state, state| {
            state.rooms.insert(room_state.id.clone(), room_state.clone());
        })
}

fn make_room_item() -> impl Widget<MinRoomState> {
    //Image::new(ImageBuf::empty())
    Label::new(|data: &MinRoomState, _env: &_| data.display_name.clone())
        //.controller(RoomItemController)
        //.on_added(|image, _ctx, state: &MinRoomState, _env| {
        //    image.set_image_data(state.icon.clone());
        //})
        .on_click(|ctx, state, _env| {
            let ui_handle = ctx.get_external_handle();
            let state = state.clone();
            tokio::spawn(async move {
                let state = NewActiveRoomState::new(state).await;
                if let Err(e) = ui_handle.submit_command(SET_ACTIVE_ROOM, state, Target::Auto) {
                    error!("{}", e);
                }
            });
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
