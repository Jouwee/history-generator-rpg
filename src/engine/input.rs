use std::collections::HashSet;

use piston::{Button, ButtonArgs, ButtonState, Key, MouseButton};

use crate::DisplayContext;

pub(crate) enum InputEvent {
    None,
    Click { button: MouseButton, pos: [f64; 2] },
    MouseMove { pos: [f64; 2] },
    Scroll { pos: [f64; 2], offset: f64 },
    Drag { button: MouseButton, offset: [f64; 2] },
    Key { key: Key }
}

impl InputEvent {
    pub(crate) fn from_button_args(args: &ButtonArgs, state:  &mut InputState) -> InputEvent {
        if args.state == ButtonState::Press {
            state.pressed.insert(args.button);
        }
        if args.state == ButtonState::Release {
            state.drag_candidate = None;
            let was_dragging = state.dragging.contains(&args.button);
            state.pressed.remove(&args.button);
            state.dragging.remove(&args.button);
            if !was_dragging {
                match args.button {
                    Button::Mouse(btn) => return InputEvent::Click { pos: state.last_mouse, button: btn.clone() },
                    Button::Keyboard(key) => return InputEvent::Key { key },
                    _ => ()
                }
            }
        }
        return InputEvent::None
    }

    pub(crate) fn from_mouse_move(mouse_pos: [f64; 2], display_ctx: &DisplayContext, state:  &mut InputState) -> InputEvent {
        let mouse_pos = [mouse_pos[0] / display_ctx.scale, mouse_pos[1] / display_ctx.scale];
        if mouse_pos != state.last_mouse {
            let last_pos = state.last_mouse;
            state.last_mouse = mouse_pos;
            if state.pressed.contains(&Button::Mouse(MouseButton::Left)) {
                let offset = [mouse_pos[0] - last_pos[0], mouse_pos[1] - last_pos[1]];
                // TODO(xYMCADko): Min-delta logic doesn't seem to have worked
                if !state.dragging.contains(&Button::Mouse(MouseButton::Left)) {
                    let pos = state.drag_candidate.get_or_insert(last_pos);
                    let dst_sqrd = (pos[0] - mouse_pos[0]).powf(2.) + (pos[1] - mouse_pos[1]).powf(2.); 
                    if dst_sqrd > 2. {
                        state.dragging.insert(Button::Mouse(MouseButton::Left));
                        state.drag_candidate = None;
                    }
                }
                if state.dragging.contains(&Button::Mouse(MouseButton::Left)) {
                    return InputEvent::Drag { offset, button: MouseButton::Left }
                } 
            }
            return InputEvent::MouseMove { pos: mouse_pos }
        }
        return InputEvent::None
    }

    pub(crate) fn from_mouse_scroll(mouse_scroll: [f64; 2], state:  &mut InputState) -> InputEvent {
        return InputEvent::Scroll { pos: state.last_mouse, offset: mouse_scroll[1] };
    }
}

pub(crate) struct InputState {
    last_mouse: [f64; 2],
    pressed: HashSet<Button>,
    drag_candidate: Option<[f64; 2]>,
    dragging: HashSet<Button>
}

impl InputState {

    pub(crate) fn new() -> InputState {
        InputState {
            last_mouse: [0.; 2],
            drag_candidate: None,
            pressed: HashSet::new(),
            dragging: HashSet::new()
        }
    }
}