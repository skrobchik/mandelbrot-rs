#![forbid(unsafe_code)]

use std::sync::Arc;
use log::error;
use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{DeviceEvent, DeviceId, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit_input_helper::WinitInputHelper;
use num_complex::Complex64;
use rayon::prelude::*;
use winit::application::ApplicationHandler;
use winit::keyboard::KeyCode;
use winit::window::{Window, WindowAttributes, WindowId};

const RESOLUTION: u32 = 800;


struct MandelbrotSet {
    set: [u8; (RESOLUTION*RESOLUTION) as usize],
    re_limits: [f64; 2],
    im_limits: [f64; 2],
    max_iterations: u32,
    color_function: fn(u8) -> [u8; 3]
}

fn hsv_to_rgb(hsv: [u8; 3]) -> [u8; 3] {
    let h: f32 = (hsv[0] as f32)/255.0 * 360.0;
    let s: f32 = (hsv[1] as f32)/255.0;
    let v: f32 = (hsv[2] as f32)/255.0;
    let f = |n: f32|{
        let u = n + h/60.0;
        let k = u - (u / 6.0).floor() * 6.0;
        v - v * s * k.min(4.0-k).min(1.0).max(0.0)
    };
    let r = 255.0 * f(5.0);
    let g = 255.0 * f(3.0);
    let b = 255.0 * f(1.0);
    [r as u8, g as u8, b as u8]
}

impl MandelbrotSet {
    pub fn normalize(iterations: u32, max_iterations: u32) -> u8 {
        ((iterations as f32) / (max_iterations as f32) * 255.0).round() as u8
    }
    pub fn mandelbrot(re: f64, im: f64, max_iterations: u32) -> u32 {
        let mut n = 0;
        let c = Complex64::new(re, im);
        let mut z = Complex64::new(0.0, 0.0);
        while n <= max_iterations && z.norm_sqr() < 4.0 {
            z = z.powu(2) + c;
            n += 1;
        }
        n
    }
    pub fn calculate(self: &mut Self) {
        let re_range = self.re_limits[1] - self.re_limits[0];
        let im_range = self.im_limits[1] - self.im_limits[0];
        let m_re = re_range / (RESOLUTION as f64);
        let m_im = im_range / (RESOLUTION as f64);
        let re0 = self.re_limits[0];
        let im0 = self.im_limits[0];
        let max_iterations = self.max_iterations;
        self.set.par_iter_mut().enumerate().for_each(|(i, c)|{
            let x = i % RESOLUTION as usize;
            let y = i / RESOLUTION as usize;
            let re = re0 + m_re * (x as f64);
            let im = im0 + m_im * (y as f64);
            *c = MandelbrotSet::normalize(MandelbrotSet::mandelbrot(re, im, max_iterations), max_iterations);
        });
    }
    pub fn new() -> Self {
        Self {
            set: [0; ((RESOLUTION*RESOLUTION) as usize)],
            re_limits: [-2.0, 2.0],
            im_limits: [-2.0, 2.0],
            max_iterations: 255,
            color_function: |c: u8| { [c, c, c] }
        }
    }
    /// Asumes 4*RESOLUTION*RESOLUTION size
    pub fn draw(self: &MandelbrotSet, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let c = self.set[i];
            let rgb = (self.color_function)(c);
            pixel.copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
    }
}

struct InitializedApp {
    resize_count: i32,
    window: Arc<Window>,
    pixels: Pixels<'static>,
    mandelbrot: MandelbrotSet,
    input: WinitInputHelper,
}

struct App(Option<InitializedApp>);

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let size = LogicalSize::new(RESOLUTION as f64, RESOLUTION as f64);
        let window = Arc::new(event_loop.create_window(WindowAttributes::default().with_title("Mandelbrot")
            .with_inner_size(size)
            .with_min_inner_size(size)).unwrap());

        let mut mandelbrot = MandelbrotSet::new();
        mandelbrot.calculate();
        mandelbrot.color_function = |c| {
            hsv_to_rgb([c, 255, 255])
        };

        let pixels: Pixels<'static> = {
            let window_size = window.inner_size();
            let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, window.clone());
            Pixels::new(RESOLUTION as u32, RESOLUTION as u32, surface_texture).unwrap()
        };

        let input = WinitInputHelper::new();

        std::mem::swap(self, &mut App(Some(InitializedApp {
            resize_count: 0,
            window,
            pixels,
            mandelbrot,
            input,
        })))
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        let app = self.0.as_mut().unwrap();

        // Draw the current frame
        if let WindowEvent::RedrawRequested = event {
            if app.resize_count > 0 {
                let size = app.window.inner_size();
                app.pixels.resize_surface(size.width, size.height).unwrap();
                app.resize_count -= 1;
            }

            app.mandelbrot.draw(app.pixels.frame_mut());

            if app.pixels
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
            pixels.resize_surface(app.window.inner_size().width, app.window.inner_size().height).unwrap();

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
                    if down { im_shift -= shift };
                    if up { im_shift += shift };
                    if left { re_shift -= shift };
                    if right { re_shift += shift };
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
                        if zoom_in { 1.0 }
                        else { -1.0 }
                    };
                    mandelbrot.re_limits[0] += zoom_dir * shift * re_range;
                    mandelbrot.re_limits[1] -= zoom_dir * shift * re_range;
                    mandelbrot.im_limits[0] += zoom_dir * shift * im_range;
                    mandelbrot.im_limits[1] -= zoom_dir * shift * im_range;
                }
                if reset {
                    std::mem::swap(mandelbrot, &mut MandelbrotSet::new());
                }
                let iterations_delta = 10;
                if more_iterations {
                    mandelbrot.max_iterations += iterations_delta;
                }
                if less_iterations && mandelbrot.max_iterations > iterations_delta {
                    mandelbrot.max_iterations -= 10;
                }
                if up || down || left || right || zoom_in || zoom_out || reset || less_iterations || more_iterations {
                    mandelbrot.calculate();
                }
            }

            app.window.request_redraw();
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: DeviceId, event: DeviceEvent) {
        let app = self.0.as_mut().unwrap();
        app.input.process_device_event(&event);
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: StartCause) {
        if let Some(app) = self.0.as_mut() {
            app.input.step();
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let app = self.0.as_mut().unwrap();
        app.input.end_step();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let event_loop = EventLoop::new()?;
    let mut app = App(None);
    event_loop.run_app(&mut app)?;
    Ok(())
}