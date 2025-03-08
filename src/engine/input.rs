use std::collections::HashSet;

use piston::{Button, ButtonArgs, ButtonState, MouseButton};

use crate::DisplayContext;

pub enum InputEvent {
    None,
    Drag { offset: [f64; 2], button: MouseButton }
}

impl InputEvent {
    pub fn from_button_args(args: &ButtonArgs, state:  &mut InputState) -> InputEvent {
        if args.state == ButtonState::Press {
            state.pressed.insert(args.button);
        }
        if args.state == ButtonState::Release {
            state.pressed.remove(&args.button);
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
                return InputEvent::Drag { offset, button: MouseButton::Left }
            }
        }
        return InputEvent::None
    }
}

pub struct InputState {
    pub last_mouse: [f64; 2],
    pub pressed: HashSet<Button>
}