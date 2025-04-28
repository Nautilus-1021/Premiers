use std::sync::atomic::{AtomicUsize, Ordering};

use winit::{
    application::ApplicationHandler, error::EventLoopError, event::WindowEvent, event_loop::{ActiveEventLoop, EventLoop}, window::{Window, WindowId}
};
use glutin_winit::{DisplayBuilder, GlWindow};
use glutin::{
    config::{ConfigTemplateBuilder, GlConfig},
    context::{ContextApi, ContextAttributesBuilder, GlProfile, Version},
    display::{GetGlDisplay, GlDisplay}
};
use winit::raw_window_handle::HasWindowHandle;

mod renderer;
use renderer::{ThreadMessage, ThreadedRenderer};
pub mod util;

fn main() -> Result<(), EventLoopError> {
    // Création de la fenêtre
    let event_loop = EventLoop::new().unwrap();

    let mut app = App { renderer: None, window: None };

    event_loop.run_app(&mut app)
}

struct App {
    renderer: Option<ThreadedRenderer>,
    window: Option<Window>
}

static RESUME_COUNTER: AtomicUsize = AtomicUsize::new(1);

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("Resume #{}", RESUME_COUNTER.fetch_add(1, Ordering::Relaxed));

        let config_template = ConfigTemplateBuilder::new().with_transparency(true);
        
        let wnd_attributes = Window::default_attributes().with_title("OpenGL Rust").with_transparent(true);

        let display_builder = DisplayBuilder::new().with_window_attributes(Some(wnd_attributes));
        
        let (wnd, gl_config) = display_builder.build(event_loop, config_template, |configs| {
            configs.reduce(|c1, c2| {
                if !c2.supports_transparency().unwrap_or(false) {
                    return c1
                } else if !c1.supports_transparency().unwrap_or(false) {
                    return c2
                }

                if c2.num_samples() > c1.num_samples() {
                    c2
                } else {
                    c1
                }
            }).unwrap()
        }).unwrap();

        println!("[Config] Samples: {}, Transparency: {}, Alpha depth: {}", gl_config.num_samples(), gl_config.supports_transparency().map(|tr| tr.to_string()).unwrap_or("<Unknown>".to_owned()), gl_config.alpha_size());

        let wnd = wnd.unwrap();

        let rwh = wnd.window_handle().ok().map(|handle| { println!("Raw handle created"); handle.as_raw() });

        let context_attributes = ContextAttributesBuilder::new().with_profile(GlProfile::Core).with_context_api(ContextApi::OpenGl(Some(Version::new(4, 3)))).build(rwh);

        let gl_display = gl_config.display();

        let not_current_context = unsafe { gl_display.create_context(&gl_config, &context_attributes).unwrap() };
        
        let attrs = wnd.build_surface_attributes(<_>::default()).expect("Unable to build attributes");
        // Création de la surface de rendu OpenGL, je ne peux que déssiner dedans
        let gl_surface = unsafe { gl_display.create_window_surface(&gl_config, &attrs).unwrap() };

        let renderer = renderer::ThreadedRenderer::new(gl_config, not_current_context, gl_surface);

        self.renderer = Some(renderer);
        self.window = Some(wnd)
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::Resized(size) => {
                if size.width != 0 && size.height != 0 {
                    // J'envoie un "message" au thread de rendu dédié

                    if let Some(ref renderer) = self.renderer {
                        renderer.send_message(ThreadMessage::Resize(size.width as i32, size.height as i32)).unwrap();
                    }
                }
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => ()
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(renderer) = self.renderer.take() {
            renderer.stop();
        }
    }
}