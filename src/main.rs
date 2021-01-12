#![forbid(unsafe_code)]

use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use num_complex::Complex64;
use rayon::prelude::*;

const RESOLUTION: u32 = 800;


struct MandelbrotSet {
    set: [u8; ((RESOLUTION*RESOLUTION) as usize)],
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

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(RESOLUTION as f64, RESOLUTION as f64);
        WindowBuilder::new()
            .with_title("Mandelbrot")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(RESOLUTION as u32, RESOLUTION as u32, surface_texture)?
    };

    let mut mandelbrot = MandelbrotSet::new();
    mandelbrot.calculate();
    mandelbrot.color_function = |c| {
        hsv_to_rgb([c, 255, 255])
    };
    
    let mut resize_count = 0;
    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            if resize_count > 0 {
                let size = window.inner_size();
                pixels.resize(size.width, size.height);
                resize_count -= 1;
            }

            mandelbrot.draw(pixels.get_frame());
            
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            //if let Some(size) = input.window_resized() {
            //    pixels.resize(size.width, size.height);
            //}
            // https://github.com/parasyte/pixels/issues/121
            pixels.resize(window.inner_size().width, window.inner_size().height);

            // Mandelbrot movement
            {
            let invert_vertical = true;
            let up = input.key_pressed(VirtualKeyCode::Up);
            let down = input.key_pressed(VirtualKeyCode::Down);
            let left = input.key_pressed(VirtualKeyCode::Left);
            let right = input.key_pressed(VirtualKeyCode::Right);
            let zoom_in = input.key_pressed(VirtualKeyCode::X);
            let zoom_out = input.key_pressed(VirtualKeyCode::Z);
            let reset = input.key_pressed(VirtualKeyCode::C);
            let less_iterations = input.key_pressed(VirtualKeyCode::Subtract);
            let more_iterations = input.key_pressed(VirtualKeyCode::Add) || input.key_pressed(VirtualKeyCode::Equals);
            
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
                mandelbrot = MandelbrotSet::new();
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

            window.request_redraw();
        }
    });
}