use crate::color_functions::rainbow;
use crate::mandelbrot::MandelbrotSet;
use log::error;
use pixels::{Pixels, SurfaceTexture};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, StartCause, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::{Window, WindowAttributes, WindowId};
use winit_input_helper::WinitInputHelper;

pub struct InitializedApp {
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
        let window = Arc::new(
            event_loop
                .create_window(WindowAttributes::default().with_title("Mandelbrot"))
                .unwrap(),
        );

        let size = window.inner_size();
        let app = InitializedApp {
            window: window.clone(),
            pixels: Pixels::new(
                size.width,
                size.height,
                SurfaceTexture::new(size.width, size.height, window),
            )
            .unwrap(),
            mandelbrot: MandelbrotSet::new(rainbow, size.width, size.height),
            input: WinitInputHelper::new(),
        };
        *self = App(Some(app));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let app = self.0.as_mut().unwrap();
        let input = &mut app.input;
        let pixels = &mut app.pixels;
        let mandelbrot = &mut app.mandelbrot;

        if let WindowEvent::Resized(size) = event {
            pixels.resize_surface(size.width, size.height).unwrap();
            pixels.resize_buffer(size.width, size.height).unwrap();
            mandelbrot.resize(size.width, size.height);
        }

        if let WindowEvent::RedrawRequested = event {
            mandelbrot.calculate();
            mandelbrot.draw(pixels.frame_mut());
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                event_loop.exit();
                return;
            }
        }

        // Handle input events
        if input.process_window_event(&event) {
            // Close events
            if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                event_loop.exit();
                return;
            }

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

                if up || down || left || right {
                    let speed_pixels =
                        (0.1 * (app.window.inner_size().height as f32)).ceil() as i32;
                    let mut translation_pixels: [i32; 2] = [0, 0];
                    if down {
                        translation_pixels[1] -= speed_pixels;
                    };
                    if up {
                        translation_pixels[1] += speed_pixels;
                    };
                    if left {
                        translation_pixels[0] -= speed_pixels;
                    };
                    if right {
                        translation_pixels[0] += speed_pixels;
                    };
                    if invert_vertical {
                        translation_pixels[1] *= -1;
                    }
                    mandelbrot.translation[0] +=
                        (translation_pixels[0] as f64) * mandelbrot.pixel_size;
                    mandelbrot.translation[1] +=
                        (translation_pixels[1] as f64) * mandelbrot.pixel_size;
                }
                let zoom_speed = 0.1;
                if zoom_in {
                    mandelbrot.pixel_size *= 1.0 - zoom_speed;
                }
                if zoom_out {
                    mandelbrot.pixel_size *= 1.0 + zoom_speed;
                }
                if reset {
                    let size = app.window.inner_size();
                    *mandelbrot = MandelbrotSet::new(rainbow, size.width, size.height);
                }
                let iterations_delta = 10;
                if more_iterations {
                    mandelbrot.max_iterations += iterations_delta;
                }
                if less_iterations && mandelbrot.max_iterations > iterations_delta {
                    mandelbrot.max_iterations -= 10;
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
