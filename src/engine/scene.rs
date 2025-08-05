use std::ops::ControlFlow;

use crate::{engine::geometry::Coord2, game::{actor::actor::Actor, chunk::TileMetadata, InputEvent}, world::{creature::CreatureId, item::Item}, GameContext};
use super::render::RenderContext;

pub(crate) struct Update {
    pub(crate) delta_time: f64,
    pub(crate) max_update_time: f64,
    pub(crate) mouse_pos_cam: [f64; 2],
}

pub(crate) trait Scene {
    fn init(&mut self, _ctx: &mut GameContext) {}
    fn render(&mut self, ctx: &mut RenderContext, game_ctx: &mut GameContext);
    fn update(&mut self, update: &Update, ctx: &mut GameContext);
    fn input(&mut self, evt: &InputEvent, ctx: &mut GameContext) -> ControlFlow<()>;
    fn event(&mut self, evt: &BusEvent, ctx: &mut GameContext) -> ControlFlow<()>;
}

pub(crate) enum BusEvent {
    ShowInspectDialog(ShowInspectDialogData),
    ShowChatDialog(ShowChatDialogData),
    CreatureKilled(CreatureId),
    AddItemToPlayer(Item)
}

#[derive(Clone)]
pub(crate) struct ShowInspectDialogData {
    pub(crate) actor: Option<Actor>,
    pub(crate) item: Option<Item>,
    pub(crate) tile_metadata: Option<TileMetadata>
}

#[derive(Clone)]
pub(crate) struct ShowChatDialogData {
    pub(crate) world_coord: Coord2,
    pub(crate) actor: Actor,
}