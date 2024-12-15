use std::{any::Any, collections::HashMap};

use crate::engine::render::RenderContext;

use super::{button::Button, dialog::Dialog, label::Label, GUINode};

pub struct InnerContainer {
    children: HashMap<String, Box<dyn Any>>,
    next_key: usize
}

impl InnerContainer {

    pub fn new() -> InnerContainer {
        InnerContainer { 
            children: HashMap::new(),
            next_key: 0,
        }
    }

}

pub trait Container {

    fn container(&self) -> &InnerContainer;

    fn container_mut(&mut self) -> &mut InnerContainer;

    fn add<T>(&mut self, node: T) where T: GUINode + 'static {
        let container = self.container_mut();
        let boxed = Box::new(node);
        container.children.insert(container.next_key.to_string(), boxed);
        container.next_key += 1;
    }

    fn add_key<T>(&mut self, key: &str, node: T) where T: GUINode + 'static {
        let boxed = Box::new(node);
        self.container_mut().children.insert(key.to_string(), boxed);
    }

    fn get_mut<T>(&mut self, key: &str) -> Option<&mut T> where T: GUINode + 'static {
        let child = self.container_mut().children.get_mut(key)?;
        child.downcast_mut::<T>()
    }

    fn render_children(&mut self, ctx: &mut RenderContext, my_rect: [f64; 4]) {
        let layout_rect = ctx.layout_rect;
        ctx.layout_rect = my_rect;
        for child in self.container_mut().children.values_mut() {
            // TODO: Find a way of not having these
            if let Some(child) = child.downcast_mut::<Label>() {
                child.render(ctx);
            }
            if let Some(child) = child.downcast_mut::<Button>() {
                child.render(ctx);
            }
            if let Some(child) = child.downcast_mut::<Dialog>() {
                child.render(ctx);
            }
        }
        ctx.layout_rect = layout_rect;
    }

}