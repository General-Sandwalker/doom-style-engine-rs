mod map;
mod player;
mod raycaster;
mod renderer;

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use winit::{
    application::ApplicationHandler,
    event::{WindowEvent, KeyEvent, ElementState},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use map::Map;
use player::Player;
use renderer::Renderer;

struct App {
    window: Option<Arc<Window>>,
    renderer: Arc<Mutex<Option<Renderer>>>,
    map: Map,
    player: Player,
    keys: HashSet<KeyCode>,
}

impl App {
    fn new() -> Self {
        let map = Map::load();
        let (sx, sy, sa) = map.player_start;
        let player = Player::new(sx, sy, sa);
        Self {
            window: None,
            renderer: Arc::new(Mutex::new(None)),
            map,
            player,
            keys: HashSet::new(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let win_attrs = Window::default_attributes()
            .with_title("Wolf3D-RS")
            .with_inner_size(winit::dpi::LogicalSize::new(
                raycaster::SCREEN_W as u32,
                raycaster::SCREEN_H as u32,
            ));

        #[cfg(target_arch = "wasm32")]
        let win_attrs = {
            use winit::platform::web::WindowAttributesExtWebSys;
            let canvas = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.get_element_by_id("canvas"))
                .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok());
            if let Some(c) = canvas { win_attrs.with_canvas(Some(c)) } else { win_attrs }
        };

        let window = Arc::new(event_loop.create_window(win_attrs).unwrap());
        self.window = Some(window.clone());

        #[cfg(not(target_arch = "wasm32"))]
        {
            *self.renderer.lock().unwrap() = Some(pollster::block_on(Renderer::new(window)));
        }

        #[cfg(target_arch = "wasm32")]
        {
            let renderer_slot = self.renderer.clone();
            wasm_bindgen_futures::spawn_local(async move {
                *renderer_slot.lock().unwrap() = Some(Renderer::new(window).await);
            });
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(key),
                    state,
                    ..
                },
                ..
            } => {
                match state {
                    ElementState::Pressed  => { self.keys.insert(key); }
                    ElementState::Released => { self.keys.remove(&key); }
                }
                if key == KeyCode::Escape {
                    event_loop.exit();
                }
            }

            WindowEvent::Resized(size) => {
                if let Ok(mut r) = self.renderer.lock() {
                    if let Some(r) = r.as_mut() {
                        r.resize(size);
                    }
                }
            }

            WindowEvent::RedrawRequested => {
                self.player.update(&self.keys, &self.map);

                if let Ok(mut guard) = self.renderer.lock() {
                    if let Some(r) = guard.as_mut() {
                        match r.render(&self.player, &self.map) {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost) => {
                                let size = winit::dpi::PhysicalSize::new(r.config.width, r.config.height);
                                r.resize(size);
                            }
                            Err(e) => eprintln!("Render error: {e:?}"),
                        }
                    }
                }

                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }

            _ => {}
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn wasm_main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Warn).expect("logger init failed");
    run();
}

pub fn run() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
