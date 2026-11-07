mod font;
mod game;
mod lighting;
mod player;
mod render;
mod rng;
mod texture;
mod world;
mod sprite;

use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::{Duration, Instant};

use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, DeviceId, ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::platform::run_on_demand::EventLoopExtRunOnDemand;
use winit::window::{CursorGrabMode, Window, WindowId};

use game::Game;
use player::Input;
use render::{RENDER_HEIGHT, RENDER_WIDTH, Renderer, blit_scaled, internal_size};
use rng::Rng;

use crate::settings::Settings;
use color_eyre::Result;
use winit::window::Fullscreen;

const TITLE: &str = "Dungeon Pre-Alpha 0.0.2 (Rust)";
const SCALE: u32 = 3;
const WIN_W: u32 = RENDER_WIDTH as u32 * SCALE;
const WIN_H: u32 = RENDER_HEIGHT as u32 * SCALE;

const TICK_NANOS: u64 = 1_000_000_000 / 60;
const TICK: Duration = Duration::from_nanos(TICK_NANOS);
const MAX_LAG: Duration = Duration::from_nanos(TICK_NANOS * 5);

const CYAN: u32 = 0x00_FF_FF;

struct Gfx {
    window: Rc<Window>,
    _context: Context<Rc<Window>>,
    surface: Surface<Rc<Window>, Rc<Window>>,
}

struct App {
    gfx: Option<Gfx>,
    game: Game,
    settings: Settings,
    renderer: Renderer,
    input: Input,

    mouse_dx: f64,
    mouse_dy: f64,
    grabbed: bool,
    quit: bool,

    debug: bool,
    block_debug: String,
    ticks: u64,
    fps: u32,
    frames: u32,
    fps_timer: Instant,

    previous: Instant,
    lag: Duration,
    frame_interval: Option<Duration>,
    next_frame: Instant,
    alpha: f32,
    closing: u8,
}

impl App {
    fn new(game: Game, settings: Settings) -> Self {
        let now = Instant::now();
        let mut renderer = Renderer::new(RENDER_WIDTH, RENDER_HEIGHT);
        renderer.fov = settings.fov as f32;
        renderer.set_color_mask(settings.color_mask(), settings.fog_shift());

        let frame_interval = if settings.fps_limit == 0 {
            None
        } else {
            Some(Duration::from_secs_f64(1.0 / settings.fps_limit as f64))
        };

        App {
            gfx: None,
            game,
            settings,
            renderer,
            input: Input::default(),
            mouse_dx: 0.0,
            mouse_dy: 0.0,
            grabbed: false,
            debug: false,
            block_debug: String::new(),
            ticks: 0,
            fps: 0,
            frames: 0,
            fps_timer: now,
            previous: now,
            lag: Duration::ZERO,
            frame_interval,
            next_frame: now,
            alpha: 1.0,
            quit: false,
            closing: 0,
        }
    }

    fn grab_cursor(&mut self) {
        if self.grabbed {
            return;
        }
        let Some(gfx) = &self.gfx else { return };
        let w = &gfx.window;
        let ok = w
            .set_cursor_grab(CursorGrabMode::Locked)
            .or_else(|_| w.set_cursor_grab(CursorGrabMode::Confined))
            .is_ok();
        if ok {
            w.set_cursor_visible(false);
            self.grabbed = true;
        }
    }

    fn release_cursor(&mut self) {
        if !self.grabbed {
            return;
        }
        self.grabbed = false;
        if let Some(gfx) = &self.gfx {
            let _ = gfx.window.set_cursor_grab(CursorGrabMode::None);
            gfx.window.set_cursor_visible(true);
        }
        self.mouse_dx = 0.0;
        self.mouse_dy = 0.0;
    }

    fn on_key(&mut self, key: PhysicalKey, state: ElementState, repeat: bool) {
        let down = state == ElementState::Pressed;
        let PhysicalKey::Code(code) = key else { return };
        match code {
            KeyCode::KeyW => self.input.forward = down,
            KeyCode::KeyS => self.input.back = down,
            KeyCode::KeyA => self.input.left = down,
            KeyCode::KeyD => self.input.right = down,
            KeyCode::Space => self.input.jump = down,
            KeyCode::ShiftLeft | KeyCode::ShiftRight => self.input.crouch = down,
            KeyCode::ControlLeft | KeyCode::ControlRight => self.input.sprint = down,
            KeyCode::ArrowLeft => self.input.turn_left = down,
            KeyCode::ArrowRight => self.input.turn_right = down,
            KeyCode::Escape if down => {
                if self.grabbed {
                    self.release_cursor();
                } else {
                    self.quit = true;
                }
            }
            KeyCode::F3 if down && !repeat => self.debug = !self.debug,
            _ => {}
        }
    }

    fn sim_tick(&mut self) {
        if self.grabbed {
            self.game.player.look(self.mouse_dx, self.mouse_dy);
        }
        self.mouse_dx = 0.0;
        self.mouse_dy = 0.0;

        self.game.tick(&self.input);
        self.ticks += 1;

        if self.debug && self.ticks % 3 == 0 {
            let p = &self.game.player;
            self.block_debug = format!(
                "BLOCK: {} {} {}",
                (p.x as i32) / 16,
                (p.y as i32) / 16,
                (p.z as i32) / 16
            );
        }
    }

    /// Destruye ventana, superficie y contexto con el bucle aún en marcha.
    fn begin_close(&mut self) {
        self.release_cursor();
        self.gfx = None;
        self.closing = 1;
    }

    fn redraw(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else { return };
        let size = gfx.window.inner_size();
        let (Some(w), Some(h)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) else {
            return;
        };

        let (dw, dh) = (size.width as usize, size.height as usize);
        let (iw, ih, scale) = internal_size(dw, dh);
        self.renderer.resize(iw, ih);
        self.renderer.render_interpolated(&self.game, self.alpha);

        if gfx.surface.resize(w, h).is_err() {
            return;
        }
        let Ok(mut buffer) = gfx.surface.buffer_mut() else {
            return;
        };
        blit_scaled(
            &self.renderer.pixels,
            iw,
            ih,
            &mut buffer[..],
            dw,
            dh,
            scale,
        );

        if self.debug {
            let margin = 8 * scale;
            let fps_text = format!("{} FPS", self.fps);
            font::draw_text(
                &mut buffer[..],
                dw,
                dh,
                margin,
                margin,
                scale,
                CYAN,
                &fps_text,
            );
            let tw = font::text_width(&self.block_debug, scale);
            font::draw_text(
                &mut buffer[..],
                dw,
                dh,
                dw.saturating_sub(tw + margin),
                margin,
                scale,
                CYAN,
                &self.block_debug,
            );
        }
        let _ = buffer.present();

        self.frames += 1;
        let now = Instant::now();
        if now - self.fps_timer >= Duration::from_secs(1) {
            self.fps = self.frames;
            self.frames = 0;
            self.fps_timer = now;
        }
    }
    fn request_redraw(&self) {
        if let Some(gfx) = &self.gfx {
            gfx.window.request_redraw();
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gfx.is_some() {
            return;
        }
        let mut attrs = Window::default_attributes()
            .with_title(TITLE)
            .with_inner_size(PhysicalSize::new(WIN_W, WIN_H));
        if self.settings.fullscreen {
            attrs = attrs.with_fullscreen(Some(Fullscreen::Borderless(None)));
        }
        let window = match event_loop.create_window(attrs) {
            Ok(w) => Rc::new(w),
            Err(e) => {
                eprintln!("No se pudo crear la ventana: {e}");
                event_loop.exit();
                return;
            }
        };
        let context =
            Context::new(window.clone()).expect("softbuffer: no se pudo crear el contexto");
        let surface = Surface::new(&context, window.clone())
            .expect("softbuffer: no se pudo crear la superficie");
        window.request_redraw();
        self.gfx = Some(Gfx {
            window,
            _context: context,
            surface,
        });
        self.previous = Instant::now();
        self.lag = Duration::ZERO;
        self.next_frame = Instant::now();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => self.quit = true,
            WindowEvent::Destroyed => event_loop.exit(),
            WindowEvent::Focused(false) => {
                self.input = Input::default();
                self.release_cursor();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.on_key(event.physical_key, event.state, event.repeat);
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                self.grab_cursor();
            }
            WindowEvent::RedrawRequested => self.redraw(),
            _ => {}
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _id: DeviceId, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
            if self.grabbed {
                self.mouse_dx += dx;
                self.mouse_dy += dy;
            }
        }
    }

    /// Timestep fijo a 60 ticks/s (igual que el Java): se renderiza solo si hubo tick.
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.closing > 0 {
            event_loop.set_control_flow(ControlFlow::Poll);
            self.closing = self.closing.saturating_add(1);
            if self.closing > 3 {
                event_loop.exit();
            }
            return;
        }
        // Fase 1: se pidió salir.
        if self.quit {
            self.begin_close();
            event_loop.set_control_flow(ControlFlow::Poll);
            return;
        }
        if self.gfx.is_none() {
            return;
        }
        let now = Instant::now();
        self.lag += now - self.previous;
        self.previous = now;
        if self.lag > MAX_LAG {
            self.lag = MAX_LAG;
        }

        while self.lag >= TICK {
            self.sim_tick();
            self.lag -= TICK;
        }
        self.alpha = (self.lag.as_secs_f32() / TICK.as_secs_f32()).clamp(0.0, 1.0); // <- nuevo
        let next_tick = now + (TICK - self.lag);

        match self.frame_interval {
            None => {
                self.request_redraw();
                event_loop.set_control_flow(ControlFlow::Poll);
            }
            Some(interval) => {
                if now >= self.next_frame {
                    self.request_redraw();
                    self.next_frame += interval;
                    if self.next_frame < now {
                        self.next_frame = now + interval;
                    }
                }
                event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_frame.min(next_tick)));
            }
        }
    }
}



pub fn run_game(settings: &Settings, event_loop: &mut EventLoop<()>) -> Result<()> {
    let seed = Rng::from_time().next_u64();
    let game = Game::new(&mut Rng::new(seed));
    let mut app = App::new(game, settings.clone());
    event_loop.run_app_on_demand(&mut app)?;
    Ok(())
    // aquí `app` se destruye y con él la ventana, antes de que el menú vuelva a abrirse
}
