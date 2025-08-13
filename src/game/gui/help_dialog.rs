use crate::{engine::gui::{containers::SimpleContainer, label::Label, layout_component::LayoutComponent, UIEvent, UINode}, globals::perf::perf, GameContext, RenderContext};

pub(crate) struct HelpDialog {
    layout: LayoutComponent,
    container: SimpleContainer,
}

impl HelpDialog {
    
    pub(crate) fn new() -> Self {
        let mut layout = LayoutComponent::new();
        layout.anchor_center().size([316., 226.]).padding([8.; 4]);

        let mut container = SimpleContainer::new();
        container.layout_component().anchor_top_center(0., 0.).size([300., 210.]);

        Self {
            layout,
            container,
        }
    }

}

impl UINode for HelpDialog {
    type State = ();
    type Input = UIEvent;

    fn layout_component(&mut self) -> &mut LayoutComponent {
        return &mut self.layout
    }

    fn init(&mut self, _state: &Self::State, _game_ctx: &mut GameContext) {
        self.container.add(Label::text("Welcome to Tales of Kathay!"));
        self.container.add(Label::text("The game is still early in development. Explore the world and interact with it's history."));
        self.container.add(Label::text("Controls: Use your mouse to move around and the hotbar to perform combat actions. You can also right-click to view more options."));
        self.container.add(Label::text("Turn-based vs realtime: The game works in both modes. While exploring, you'll usually play in realtime, but combat will happen in turn-based mode. You can swith the modes with the \"Trn\" button."));
        self.container.add(Label::text("Sleep: You can sleep in any bed to recover your health, or wait a long time."));
        self.container.add(Label::text("Quests: You can talk to the ruler of each city to get quests and rewards."));
        self.container.add(Label::text("Map: Use the map to orient yourself and fast travel to discovered locations. All major cities are already marked for you."));
        self.container.add(Label::text("Codex: Everything you discover on the world will be automatically recorded in your codex. The codex will be reset upon death."));
        self.container.add(Label::text("You can view this screen again by pressing the \"?\" button."));
    }

    fn render(&mut self, _state: &Self::State, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        perf().start("help");

        let copy = ctx.layout_rect;
        ctx.layout_rect = self.layout.compute_inner_layout_rect(ctx.layout_rect);
        self.container.render(&(), ctx, game_ctx);
        ctx.layout_rect = copy;

        perf().end("help");
    }

}