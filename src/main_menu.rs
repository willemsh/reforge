use std::rc::Rc;
use std::cell::RefCell;
use std::path::Path;

use glutin_window::GlutinWindow;
use piston::event_loop::Events;
use graphics::{Context, ImageSize};
use piston::input::*;
use opengl_graphics::{GlGraphics, Texture};

#[derive(PartialEq)]
pub enum MainMenuSelection {
    Multiplayer,
    Exit,
}

pub struct MainMenu {
    selected: u8,
    done: bool,

    mouse_x: f64,
    mouse_y: f64,

    // Textures
    bg_texture: Texture,
    multiplayer_texture: Texture,
    exit_texture: Texture,
}

impl MainMenu {
    pub fn new() -> MainMenu {
        MainMenu {
            selected: 0,
            done: false,
            mouse_x: 0.0,
            mouse_y: 0.0,
            bg_texture: Texture::from_path(&Path::new("content/textures/gui/main_menu.png")).unwrap(),
            multiplayer_texture: Texture::from_path(&Path::new("content/textures/gui/multiplayer.png")).unwrap(),
            exit_texture: Texture::from_path(&Path::new("content/textures/gui/exit.png")).unwrap(),
        }
    }

    pub fn run<F>(mut self, window: &Rc<RefCell<GlutinWindow>>, gl: &mut GlGraphics, mut f: F)
        where
            F: FnMut(&Rc<RefCell<GlutinWindow>>, &mut GlGraphics, &Texture, MainMenuSelection) -> bool
    {
        // Main loop
        for e in Events::events(window.clone()) {
            use piston::event_loop as event;
            use piston::input;
            use piston::event_loop::*;

            let e: input::Event<input::Input> = e;

            self.event(&e);

            // Render GUI
            e.render(|args| {
                gl.draw(args.viewport(), |c, gl| {
                    self.draw(&c, gl);
                });
            });

            if self.done {
                let menu_selection =
                    match self.selected {
                        0 => MainMenuSelection::Multiplayer,
                        1 => MainMenuSelection::Exit,
                        _ => panic!("Invalid main menu selection"),
                    };
                if !f(window, gl, &self.bg_texture, menu_selection) {
                    break;
                }
                self.done = false;
            }
        }
    }

    pub fn event<E: GenericEvent>(&mut self, e: &E) {
        use piston::event_loop::*;
        
        e.mouse_cursor(|x, y| {
            self.on_mouse_moved(x, y);
        });
        e.press(|button| {
            match button {
                Button::Keyboard(key) => self.on_key_pressed(key), 
                Button::Mouse(button) => {
                    self.on_mouse_pressed(button);
                },
                _ => { },
            }
        });
    }

    fn on_key_pressed(&mut self, key: keyboard::Key) {
        use piston::input::keyboard::Key;
        match key {
            Key::Up if self.selected > 0 => { self.selected -= 1; },
            Key::Up if self.selected == 0 => { self.selected = 1; },
            Key::Down if self.selected < 1 => { self.selected += 1; },
            Key::Down if self.selected == 1 => { self.selected = 0; },
            Key::Return => { self.done = true; },
            _ => {},
        }
    }

    fn on_mouse_pressed(&mut self, button: mouse::MouseButton) {
        match button {
            mouse::MouseButton::Left => {
                if self.is_mouse_over_button() == 0 {
                    self.done = true;
                } else if self.is_mouse_over_button() == 1 {
                    self.done = true;
                } else {}
            },
            mouse::MouseButton::Right => {},
            _ => {},
        }
    }

    fn on_mouse_moved(&mut self, x: f64, y: f64) {
        self.mouse_x = x;
        self.mouse_y = y;

        self.selected = self.is_mouse_over_button();
    }

    fn is_mouse_over_button(&mut self) -> u8 {
        let (m_width, m_height) = self.multiplayer_texture.get_size();
        let (e_width, e_height) = self.exit_texture.get_size();

        let mut selected: u8; // is the "button" selected
        selected = self.selected;

        if self.mouse_x > 550.0 && self.mouse_x < (550.0 + (m_width as f64)) && 
            self.mouse_y > 300.0 && self.mouse_y < (300.0 + (m_height as f64)) {
            selected = 0;
        } else if self.mouse_x > 550.0 && self.mouse_x < (550.0 + (e_width as f64)) && 
            self.mouse_y > 400.0 && self.mouse_y < (400.0 + (e_height as f64)) {
            selected = 1;
        }

        selected
    }

    fn draw(&mut self, context: &Context, gl: &mut GlGraphics) {
        use graphics::*;
        
        // Clear the screen
        clear([0.0; 4], gl);

        image(&self.bg_texture, context.transform, gl);
        image(&self.multiplayer_texture, context.trans(550.0, 300.0).transform, gl);
        image(&self.exit_texture, context.trans(550.0, 400.0).transform, gl);

        if self.selected == 0 {
            let context = context.trans(550.0, 300.0);
            Image::new()
                .color([1.0, 0.0, 0.0, 1.0])
                .draw(&self.multiplayer_texture, &context.draw_state, context.transform, gl);
        }
        if self.selected == 1 {
            let context = context.trans(550.0, 400.0);
            Image::new()
                .color([1.0, 0.0, 0.0, 1.0])
                .draw(&self.exit_texture, &context.draw_state, context.transform, gl);
        }
    }
}
