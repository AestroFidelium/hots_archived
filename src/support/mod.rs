use glutin::prelude::*;

use glium::Display;
use glutin::display::GetGlDisplay;
use glutin::surface::WindowSurface;
use raw_window_handle::HasWindowHandle;
use std::num::NonZeroU32;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

pub use app::*;
pub use atlas::*;
pub use draw::*;
pub use fpscounter::*;
pub use serverutils::*;

pub mod app;
pub mod atlas;
pub mod draw;
pub mod fpscounter;
pub mod gltfmodel;
pub mod image_renderer;
pub mod serverutils;
pub mod utils;

pub trait ApplicationContext {
    fn draw_frame(&mut self, _delta_time: f32, _display: &Display<WindowSurface>) {}
    fn new(display: &Display<WindowSurface>) -> Self;
    fn update(&mut self) {}
    fn handle_window_event(
        &mut self,
        _event: &glium::winit::event::WindowEvent,
        _window: &glium::winit::window::Window,
    ) {
    }
    const WINDOW_TITLE: &'static str;
}

pub struct State<T> {
    pub display: glium::Display<WindowSurface>,
    pub window: glium::winit::window::Window,
    pub context: T,
}

impl<T: ApplicationContext + 'static> State<T> {
    pub fn new(event_loop: &glium::winit::event_loop::ActiveEventLoop, visible: bool) -> Self {
        let window_attributes = winit::window::Window::default_attributes()
            .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
            .with_position(LogicalPosition::new(1920, 1080))
            .with_inner_size(LogicalSize::new(1920.0, 1080.0))
            .with_maximized(true)
            .with_title(T::WINDOW_TITLE)
            .with_decorations(false)
            .with_visible(visible);

        let config_template_builder = glutin::config::ConfigTemplateBuilder::new();
        let display_builder =
            glutin_winit::DisplayBuilder::new().with_window_attributes(Some(window_attributes));

        let (window, gl_config) = display_builder
            .build(event_loop, config_template_builder, |mut configs| {
                configs.next().unwrap()
            })
            .unwrap();
        let window = window.unwrap();

        let window_handle = window
            .window_handle()
            .expect("couldn't obtain window handle");
        let context_attributes =
            glutin::context::ContextAttributesBuilder::new().build(Some(window_handle.into()));
        let fallback_context_attributes = glutin::context::ContextAttributesBuilder::new()
            .with_context_api(glutin::context::ContextApi::Gles(None))
            .build(Some(window_handle.into()));

        let not_current_gl_context = unsafe {
            gl_config
                .display()
                .create_context(&gl_config, &context_attributes)
                .unwrap_or_else(|_| {
                    gl_config
                        .display()
                        .create_context(&gl_config, &fallback_context_attributes)
                        .expect("failed to create context")
                })
        };

        let (width, height): (u32, u32) = if visible {
            window.inner_size().into()
        } else {
            (1920, 1080)
        };
        let attrs = glutin::surface::SurfaceAttributesBuilder::<WindowSurface>::new().build(
            window_handle.into(),
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        );
        let surface = unsafe {
            gl_config
                .display()
                .create_window_surface(&gl_config, &attrs)
                .unwrap()
        };
        let current_context = not_current_gl_context.make_current(&surface).unwrap();
        let display = glium::Display::from_context_surface(current_context, surface).unwrap();

        Self::from_display_window(display, window)
    }

    pub fn from_display_window(
        display: glium::Display<WindowSurface>,
        window: glium::winit::window::Window,
    ) -> Self {
        let context = T::new(&display);
        Self {
            display,
            window,
            context,
        }
    }

    pub fn run_loop() {
        let event_loop = glium::winit::event_loop::EventLoop::builder()
            .build()
            .expect("event loop building");
        let mut app = App::<T> {
            state: None,
            visible: true,
            close_promptly: false,
            now: Instant::now(),
        };

        let result = event_loop.run_app(&mut app);
        result.unwrap();
    }
}

struct App<T> {
    state: Option<State<T>>,
    visible: bool,
    close_promptly: bool,
    now: Instant,
}

impl<T: ApplicationContext + 'static> ApplicationHandler<()> for App<T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.state = Some(State::new(event_loop, self.visible));

        if !self.visible && self.close_promptly {
            event_loop.exit();
        }
    }
    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.state = None;
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            glium::winit::event::WindowEvent::Resized(new_size) => {
                if let Some(state) = &self.state {
                    state.display.resize(new_size.into());
                }
            }
            glium::winit::event::WindowEvent::RedrawRequested => {
                if let Some(state) = &mut self.state {
                    let now = Instant::now();
                    let delta_time = now.duration_since(self.now).as_secs_f32();
                    state.context.draw_frame(delta_time, &state.display);
                    self.now = now;
                    if self.close_promptly {
                        event_loop.exit();
                    }
                }
            }
            glium::winit::event::WindowEvent::CloseRequested
            | glium::winit::event::WindowEvent::KeyboardInput {
                event:
                    glium::winit::event::KeyEvent {
                        state: glium::winit::event::ElementState::Pressed,
                        logical_key:
                            glium::winit::keyboard::Key::Named(glium::winit::keyboard::NamedKey::Escape),
                        ..
                    },
                ..
            } => event_loop.exit(),
            ev => {
                if let Some(state) = &mut self.state {
                    state.context.handle_window_event(&ev, &state.window);
                }
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = &mut self.state {
            state.context.update();
            state.window.request_redraw();
        }
    }
}
