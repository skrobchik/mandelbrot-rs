use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event_loop::ActiveEventLoop;
use winit::event::{DeviceEvent, DeviceId, StartCause, WindowEvent};
use winit::dpi::LogicalSize;
use winit::window::{Window, WindowAttributes, WindowId};
use pixels::{Pixels, SurfaceTexture};
use winit_input_helper::WinitInputHelper;
use log::error;
use winit::keyboard::KeyCode;
use crate::color_functions::rainbow;
use crate::mandelbrot::{MandelbrotSet, RESOLUTION};

pub struct InitializedApp {
    resize_count: i32,
    window: Arc<Window>,
    pixels: Pixels<'static>,
    mandelbrot: MandelbrotSet,
    input: WinitInputHelper,
}

pub struct App(pub Option<InitializedApp>);

impl ApplicationHandler for App {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: StartCause) {
        if let Some(app) = self.0.as_mut() {
            app.input.step();
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let size = LogicalSize::new(RESOLUTION as f64, RESOLUTION as f64);
        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title("Mandelbrot")
                        .with_inner_size(size)
                        .with_min_inner_size(size),
                )
                .unwrap(),
        );

        let mut mandelbrot = MandelbrotSet::new(rainbow);
        mandelbrot.calculate();

        let pixels: Pixels<'static> = {
            let window_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, window.clone());
            Pixels::new(RESOLUTION, RESOLUTION, surface_texture).unwrap()
        };

        let input = WinitInputHelper::new();

        *self = App(Some(InitializedApp {
            resize_count: 0,
            window,
            pixels,
            mandelbrot,
            input,
        }));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let app = self.0.as_mut().unwrap();

        // Draw the current frame
        if let WindowEvent::RedrawRequested = event {
            if app.resize_count > 0 {
                let size = app.window.inner_size();
                app.pixels.resize_surface(size.width, size.height).unwrap();
                app.resize_count -= 1;
            }

            app.mandelbrot.draw(app.pixels.frame_mut());

            if app
                .pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                event_loop.exit();
                return;
            }
        }

        let input = &mut app.input;
        let pixels = &mut app.pixels;
        let mandelbrot = &mut app.mandelbrot;
        // Handle input events
        if input.process_window_event(&event) {
            // Close events
            if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                event_loop.exit();
                return;
            }

            // Resize the window
            //if let Some(size) = input.window_resized() {
            //    pixels.resize(size.width, size.height);
            //}
            // https://github.com/parasyte/pixels/issues/121
            pixels
                .resize_surface(
                    app.window.inner_size().width,
                    app.window.inner_size().height,
                )
                .unwrap();

            // Mandelbrot movement
            {
                let invert_vertical = true;
                let up = input.key_pressed(KeyCode::ArrowUp);
                let down = input.key_pressed(KeyCode::ArrowDown);
                let left = input.key_pressed(KeyCode::ArrowLeft);
                let right = input.key_pressed(KeyCode::ArrowRight);
                let zoom_in = input.key_pressed(KeyCode::KeyX);
                let zoom_out = input.key_pressed(KeyCode::KeyZ);
                let reset = input.key_pressed(KeyCode::KeyC);
                let less_iterations = input.key_pressed(KeyCode::Minus);
                let more_iterations = input.key_pressed(KeyCode::Equal);

                let im_range = mandelbrot.im_limits[1] - mandelbrot.im_limits[0];
                let re_range = mandelbrot.re_limits[1] - mandelbrot.re_limits[0];
                let shift = 0.1;

                if up || down || left || right {
                    let mut im_shift = 0.0;
                    let mut re_shift = 0.0;
                    if down {
                        im_shift -= shift
                    };
                    if up {
                        im_shift += shift
                    };
                    if left {
                        re_shift -= shift
                    };
                    if right {
                        re_shift += shift
                    };
                    if invert_vertical {
                        im_shift *= -1.0;
                    }
                    mandelbrot.im_limits[0] += im_shift * im_range;
                    mandelbrot.im_limits[1] += im_shift * im_range;
                    mandelbrot.re_limits[0] += re_shift * re_range;
                    mandelbrot.re_limits[1] += re_shift * re_range;
                }
                if zoom_in || zoom_out {
                    let zoom_dir = {
                        if zoom_in {
                            1.0
                        } else {
                            -1.0
                        }
                    };
                    mandelbrot.re_limits[0] += zoom_dir * shift * re_range;
                    mandelbrot.re_limits[1] -= zoom_dir * shift * re_range;
                    mandelbrot.im_limits[0] += zoom_dir * shift * im_range;
                    mandelbrot.im_limits[1] -= zoom_dir * shift * im_range;
                }
                if reset {
                    *mandelbrot = MandelbrotSet::new(rainbow);
                }
                let iterations_delta = 10;
                if more_iterations {
                    mandelbrot.max_iterations += iterations_delta;
                }
                if less_iterations && mandelbrot.max_iterations > iterations_delta {
                    mandelbrot.max_iterations -= 10;
                }
                if up
                    || down
                    || left
                    || right
                    || zoom_in
                    || zoom_out
                    || reset
                    || less_iterations
                    || more_iterations
                {
                    mandelbrot.calculate();
                }
            }

            app.window.request_redraw();
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        let app = self.0.as_mut().unwrap();
        app.input.process_device_event(&event);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let app = self.0.as_mut().unwrap();
        app.input.end_step();
    }
}