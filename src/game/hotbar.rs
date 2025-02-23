use graphics::CharacterCache;
use image::ImageReader;
use opengl_graphics::{Filter, Texture, TextureSettings};

use crate::{commons::id_vec::Id, engine::{gui::{button::{Button, ButtonEvent}, container::Container, hlist::HList, Anchor, GUINode, Position}, render::RenderContext, scene::Update, sprite::Sprite, Color}, resources::resources::Actions, GameContext};

use super::{action::ActionId, actor::Actor, inventory::inventory::Inventory, InputEvent};

pub struct Hotbar {
    background: Texture,
    available_actions: Vec<ActionId>,
    equipped_actions: Vec<ActionId>,
    pub selected_action: Option<ActionId>,
    action_buttons: HList
}

impl Hotbar {
    pub fn new() -> Hotbar {
        let settings = TextureSettings::new().filter(Filter::Nearest);
        let background = ImageReader::open("assets/sprites/gui/hotbar/background.png").unwrap().decode().unwrap();
        Hotbar {
            background: Texture::from_image(&background.to_rgba8(), &settings),
            available_actions: Vec::new(),
            equipped_actions: Vec::new(),
            selected_action: None,
            action_buttons: HList::new(Position::Anchored(Anchor::BottomCenter, 0., -24.))
        }
    }

    pub fn init(&mut self, inventory: &Inventory, ctx: &GameContext) {
        self.available_actions.push(ctx.resources.actions.id_of("act:talk"));
        self.available_actions.push(ctx.resources.actions.id_of("act:pickup"));
        self.available_actions.push(ctx.resources.actions.id_of("act:sleep"));
        self.available_actions.push(ctx.resources.actions.id_of("act:punch"));
        self.equip(inventory, ctx);
    }

    pub fn equip(&mut self, inventory: &Inventory, ctx: &GameContext) {
        self.equipped_actions = Vec::new();
        if let Some(equipped) = inventory.equipped() {
            self.equipped_actions = equipped.actions(&ctx.resources.actions);
        }
        self.update_buttons(&ctx.resources.actions);
    }

    fn update_buttons(&mut self, actions: &Actions) {
        self.action_buttons.clear();
        self.action_buttons.size = Some([128., 24.]);
        for action_id in self.available_actions.iter().chain(self.equipped_actions.iter()) {
            let action = actions.get(action_id);
            self.action_buttons.add_key(&format!("act_{}", action_id.as_usize()), Button::new_bg(Sprite::new(action.icon.clone()).texture, Position::Auto));
        }
    }

}

impl<'a> NodeWithState<HotbarState<'a>> for Hotbar {
    fn render(&mut self, state: HotbarState, ctx: &mut RenderContext, _ctx: &GameContext) {
        // Background
        let center = ctx.layout_rect[2] / 2.;
        let base_pos = [center - 128., ctx.layout_rect[3] - 34.];
        ctx.texture_ref(&self.background, base_pos);

        let mut hp_pos = base_pos.clone();
        hp_pos[0] = hp_pos[0] + 64.;
        hp_pos[1] = hp_pos[1] + 3.;

        let health_pct = (state.player.hp.health_points / state.player.hp.max_health_points as f32) as f64;
        ctx.rectangle_fill([hp_pos[0], hp_pos[1], (62. * health_pct).round(), 5.], Color::from_hex("994444"));

        let text = format!("{:.0}/{:.0}", state.player.hp.health_points, state.player.hp.max_health_points);
        let text_width = ctx.small_font.width(5, &text).unwrap_or(0.);
        ctx.text_small(&text, 5, [(hp_pos[0] + 31. - text_width / 2.).round(), hp_pos[1] + 5.], Color::from_hex("ffffff"));

        let mut ap_pos = base_pos.clone();
        ap_pos[0] = ap_pos[0] + 131.;
        ap_pos[1] = ap_pos[1] + 3.;

        let action_pct = (state.player.ap.action_points as f32 / state.player.ap.max_action_points as f32) as f64;
        ctx.rectangle_fill([ap_pos[0], ap_pos[1], (62. * action_pct).round(), 5.], Color::from_hex("446d99"));

        let text = format!("{:.0}/{:.0}", state.player.ap.action_points, state.player.ap.max_action_points);
        let text_width = ctx.small_font.width(5, &text).unwrap_or(0.);
        ctx.text_small(&text, 5, [(ap_pos[0] + 31. - text_width / 2.).round(), ap_pos[1] + 5.], Color::from_hex("ffffff"));

        self.action_buttons.render(ctx);

    }

    fn update(&mut self, _state: HotbarState, _update: &Update, _ctx: &GameContext) {
    }

    fn input(&mut self, _state: HotbarState, evt: &InputEvent, _ctx: &GameContext) {
        for action_id in self.available_actions.iter().chain(self.equipped_actions.iter()) {
            if let ButtonEvent::Click = self.action_buttons.get_mut::<Button>(&format!("act_{}", action_id.as_usize())).unwrap().event(evt) {
                self.selected_action = Some(*action_id);
            }
        }
    }
}


pub struct HotbarState<'a> {
    player: &'a Actor
}


impl<'a> HotbarState<'a> {
    pub fn new(player: &'a Actor) -> HotbarState<'a> {
        HotbarState { player }
    }
}

pub trait NodeWithState<T> {
    fn render(&mut self, _state: T, _ctx: &mut RenderContext, _game_ctx: &GameContext) {}
    fn update(&mut self, _state: T, _update: &Update, _ctx: &GameContext) {}
    fn input(&mut self, _state: T, _evt: &InputEvent, _ctx: &GameContext) {}
}