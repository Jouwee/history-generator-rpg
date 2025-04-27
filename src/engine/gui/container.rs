use std::{any::Any, collections::BTreeMap};

use crate::{engine::{render::RenderContext, scene::Update}, GameContext};

use super::{button::Button, dialog::Dialog, label::Label, vlist::VList, GUINode};

pub(crate) struct InnerContainer {
    pub(crate) keys: BTreeMap<String, usize>,
    pub(crate) children: Vec<Box<dyn Any>>
}

impl InnerContainer {

    pub(crate) fn new() -> InnerContainer {
        InnerContainer { 
            keys: BTreeMap::new(),
            children: Vec::new(),
        }
    }

}

pub(crate) trait Container {

    fn container_mut(&mut self) -> &mut InnerContainer;

    fn add<T>(&mut self, node: T) where T: GUINode + 'static {
        let container = self.container_mut();
        let boxed = Box::new(node);
        container.children.push(boxed);
    }

    fn add_key<T>(&mut self, key: &str, node: T) where T: GUINode + 'static {
        let boxed = Box::new(node);
        let container = self.container_mut();
        container.keys.insert(key.to_string(), container.children.len());
        container.children.push(boxed);
    }

    fn clear(&mut self) {
        self.container_mut().children.clear();
    }

    fn get_mut<T>(&mut self, key: &str) -> Option<&mut T> where T: GUINode + 'static {
        let container = self.container_mut();
        let index = container.keys.get(key)?;
        let child = container.children.get_mut(*index)?;
        child.downcast_mut::<T>()
    }

    fn render_children(&mut self, ctx: &mut RenderContext, game_ctx: &mut GameContext, my_rect: [f64; 4]) {
        let layout_rect = ctx.layout_rect;
        ctx.layout_rect = my_rect;
        for child in self.container_mut().children.iter_mut() {
            if let Some(gui_node) = Self::to_gui_node(child) {
                gui_node.render(ctx, game_ctx);
            }
        }
        ctx.layout_rect = layout_rect;
    }

    fn update_children(&mut self, update: &Update, ctx: &mut GameContext) {
        for child in self.container_mut().children.iter_mut() {
            if let Some(gui_node) = Self::to_gui_node(child) {
                gui_node.update(update, ctx);
            }
        }
    }

    fn to_gui_node<'a>(unknown: &'a mut Box<dyn Any>) -> Option<&'a mut dyn GUINode> {
        // TODO: Find a way of not having these
        if unknown.is::<Label>() {
            return Some(unknown.downcast_mut::<Label>().unwrap())
        }
        if unknown.is::<Button>() {
            return Some(unknown.downcast_mut::<Button>().unwrap())
        }
        if unknown.is::<Dialog>() {
            return Some(unknown.downcast_mut::<Dialog>().unwrap())
        }
        if unknown.is::<VList>() {
            return Some(unknown.downcast_mut::<VList>().unwrap())
        }
        return None
    }

}