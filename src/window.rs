//! Primitives for creating and controlling [`Window`](struct.Window.html).

use glutin;
use glutin::{GlContext, GlWindow};
use mint;
use render;

use factory::Factory;
use input::Input;
use render::Renderer;
use std::path::PathBuf;
use std::os::raw::c_void;

/// `Window` is the core entity of every `three-rs` application.
///
/// It provides [user input](struct.Window.html#method.update),
/// [`Factory`](struct.Factory.html) and [`Renderer`](struct.Renderer.html).
pub struct Window {
    window: glutin::GlWindow,
}

impl ::Context for Window {
    fn query_proc_address(&self, symbol: &str) -> *const c_void {
        self.window.get_proc_address(symbol) as *const c_void
    }
}

/// Builder for creating new [`Window`](struct.Window.html) with desired parameters.
#[derive(Debug, Clone)]
pub struct Builder {
    dimensions: (u32, u32),
    fullscreen: bool,
    multisampling: u16,
    shader_directory: Option<PathBuf>,
    title: String,
    vsync: bool,
}

impl Builder {
    /// Set the size of the viewport (the resolution) in pixels. Defaults to 1024x768.
    pub fn dimensions(
        &mut self,
        width: u32,
        height: u32,
    ) -> &mut Self {
        self.dimensions = (width, height);
        self
    }

    /// Whether enable fullscreen mode or not. Defauls to `false`.
    pub fn fullscreen(
        &mut self,
        option: bool,
    ) -> &mut Self {
        self.fullscreen = option;
        self
    }

    /// Sets the multisampling level to request. A value of `0` indicates that multisampling must
    /// not be enabled. Must be the power of 2. Defaults to `0`.
    pub fn multisampling(
        &mut self,
        option: u16,
    ) -> &mut Self {
        self.multisampling = option;
        self
    }

    /// Specifies the user shader directory.
    pub fn shader_directory<P: Into<PathBuf>>(
        &mut self,
        option: P,
    ) -> &mut Self {
        self.shader_directory = Some(option.into());
        self
    }

    /// Whether to enable vertical synchronization or not. Defaults to `true`.
    pub fn vsync(
        &mut self,
        option: bool,
    ) -> &mut Self {
        self.vsync = option;
        self
    }

    /// Create new `Window` with desired parameters.
    pub fn build(&mut self) -> (Window, Input, Renderer, Factory) {
        let events_loop = glutin::EventsLoop::new();
        let builder = if self.fullscreen {
            let monitor_id = events_loop.get_primary_monitor();
            glutin::WindowBuilder::new().with_fullscreen(Some(monitor_id))
        } else {
            glutin::WindowBuilder::new()
        };

        let builder = builder
            .with_dimensions(self.dimensions.0, self.dimensions.1)
            .with_title(self.title.clone());

        let context = glutin::ContextBuilder::new()
            .with_vsync(self.vsync)
            .with_multisampling(self.multisampling);

        let mut source_set = render::source::Set::default();
        if let Some(path) = self.shader_directory.as_ref() {
            let path = path.to_str().unwrap();
            macro_rules! try_override {
                ($name:ident) => {
                    match render::Source::user(path, stringify!($name), "vs") {
                        Ok(src) => {
                            info!("Overriding {}_vs.glsl", stringify!($name));
                            source_set.$name.vs = src;
                        }
                        Err(err) => {
                            error!("{:#?}", err);
                            info!("Using default {}_vs.glsl", stringify!($name));
                        }
                    }
                    match render::Source::user(path, stringify!($name), "ps") {
                        Ok(src) => {
                            info!("Overriding {}_ps.glsl", stringify!($name));
                            source_set.$name.ps = src;
                        }
                        Err(err) => {
                            error!("{:#?}", err);
                            info!("Using default {}_ps.glsl", stringify!($name));
                        }
                    }
                };
                ( $($name:ident,)* ) => {
                    $( try_override!($name); )*
                };
            }
            try_override!(basic, gouraud, pbr, phong, quad, shadow, skybox, sprite,);
        }

        let glx = GlWindow::new(builder, context, &events_loop).unwrap();
        unsafe { glx.make_current().expect("GL context bind failed") };
        let window = Window {  window: glx };
        let factory = Factory::new(&window);
        let renderer = Renderer::new(factory.clone());
        let input = Input::new(events_loop);
        (window, input, renderer, factory)
    }
}

impl Window {
    /// Create a new window with default parameters.
    pub fn new<T: Into<String>>(title: T) -> (Self, Input, Renderer, Factory) {
        Self::builder(title).build()
    }

    /// Create new `Builder` with standard parameters.
    pub fn builder<T: Into<String>>(title: T) -> Builder {
        Builder {
            dimensions: (1024, 768),
            fullscreen: false,
            multisampling: 0,
            shader_directory: None,
            title: title.into(),
            vsync: true,
        }
    }

    /// Presents the front buffer.
    pub fn swap_buffers(&mut self) {
        self.window.swap_buffers().unwrap();
    }

    /// Get current window size in pixels.
    pub fn size(&self) -> mint::Vector2<f32> {
        let size = self.window
            .get_inner_size()
            .expect("Can't get window size");
        [size.0 as f32, size.1 as f32].into()
    }

    /// Set cursor visibility
    pub fn show_cursor(
        &self,
        enable: bool,
    ) {
        let _ = if enable {
            self.window.set_cursor_state(glutin::CursorState::Normal)
        } else {
            self.window.set_cursor_state(glutin::CursorState::Hide)
        };
    }

    /// Returns underlaying `glutin::GlWindow`.
    #[cfg(feature = "opengl")]
    pub fn glutin_window(&self) -> &glutin::GlWindow {
        &self.window
    }
}
