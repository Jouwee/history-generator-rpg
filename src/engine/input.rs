use std::collections::HashSet;

use piston::{Button, ButtonArgs, ButtonState, MouseButton};

use crate::DisplayContext;

pub enum InputEvent {
    None,
    Click { button: MouseButton, pos: [f64; 2] },
    Drag { button: MouseButton, offset: [f64; 2] }
}

impl InputEvent {
    pub fn from_button_args(args: &ButtonArgs, state:  &mut InputState) -> InputEvent {
        if args.state == ButtonState::Press {
            state.pressed.insert(args.button);
        }
        if args.state == ButtonState::Release {
            let was_dragging = state.dragging.contains(&args.button);
            state.pressed.remove(&args.button);
            state.dragging.remove(&args.button);
            if !was_dragging {
                match args.button {
                    Button::Mouse(btn) => return InputEvent::Click { pos: state.last_mouse, button: btn.clone() },
                    _ => ()
                }
            }
        }
        return InputEvent::None
    }

    pub fn from_mouse_move(mouse_pos: [f64; 2], display_ctx: &DisplayContext, state:  &mut InputState) -> InputEvent {
        let mouse_pos = [mouse_pos[0] / display_ctx.scale, mouse_pos[1] / display_ctx.scale];
        if mouse_pos != state.last_mouse {
            let last_pos = state.last_mouse;
            state.last_mouse = mouse_pos;
            if state.pressed.contains(&Button::Mouse(MouseButton::Left)) {
                let offset = [mouse_pos[0] - last_pos[0], mouse_pos[1] - last_pos[1]];
                state.dragging.insert(Button::Mouse(MouseButton::Left));
                return InputEvent::Drag { offset, button: MouseButton::Left }
            }
        }
        return InputEvent::None
    }
}

pub struct InputState {
    last_mouse: [f64; 2],
    pressed: HashSet<Button>,
    dragging: HashSet<Button>
}

impl InputState {

    pub fn new() -> InputState {
        InputState {
            last_mouse: [0.; 2],
            pressed: HashSet::new(),
            dragging: HashSet::new()
        }
    }
}