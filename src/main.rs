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

const RESOLUTION: u32 = 400;

struct MandelbrotSet {
    set: [u8; ((RESOLUTION*RESOLUTION) as usize)],
    re_limits: [f64; 2],
    im_limits: [f64; 2],
    max_iterations: u32,
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
    pub fn calculate(self: &mut MandelbrotSet) {
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
    pub fn new() -> MandelbrotSet {
        MandelbrotSet {
            set: [0; ((RESOLUTION*RESOLUTION) as usize)],
            re_limits: [-2.0, 2.0],
            im_limits: [-2.0, 2.0],
            max_iterations: 255
        }
    }
    /// Asumes 4*RESOLUTION*RESOLUTION size
    pub fn draw(self: &MandelbrotSet, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let c = self.set[i];
            pixel.copy_from_slice(&[c, c, c, 255]);
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
            pixels.resize(window.inner_size().width, window.inner_size().height);

            if input.key_pressed(VirtualKeyCode::Down) {
                let im_range = mandelbrot.im_limits[1] - mandelbrot.im_limits[0];
                mandelbrot.im_limits[0] += 0.1 * im_range;
                mandelbrot.im_limits[1] += 0.1 * im_range;
                mandelbrot.calculate();
            }
            if input.key_pressed(VirtualKeyCode::Up) {
                let im_range = mandelbrot.im_limits[1] - mandelbrot.im_limits[0];
                mandelbrot.im_limits[0] -= 0.1 * im_range;
                mandelbrot.im_limits[1] -= 0.1 * im_range;
                mandelbrot.calculate();
            }
            if input.key_pressed(VirtualKeyCode::Left) {
                let re_range = mandelbrot.re_limits[1] - mandelbrot.re_limits[0];
                mandelbrot.re_limits[0] -= 0.1 * re_range;
                mandelbrot.re_limits[1] -= 0.1 * re_range;
                mandelbrot.calculate();
            }
            if input.key_pressed(VirtualKeyCode::Right) {
                let re_range = mandelbrot.re_limits[1] - mandelbrot.re_limits[0];
                mandelbrot.re_limits[0] += 0.1 * re_range;
                mandelbrot.re_limits[1] += 0.1 * re_range;
                mandelbrot.calculate();
            }
            if input.key_pressed(VirtualKeyCode::X) {
                let re_range = mandelbrot.re_limits[1] - mandelbrot.re_limits[0];
                mandelbrot.re_limits[0] += 0.1 * re_range;
                mandelbrot.re_limits[1] -= 0.1 * re_range;
                let im_range = mandelbrot.im_limits[1] - mandelbrot.im_limits[0];
                mandelbrot.im_limits[0] += 0.1 * im_range;
                mandelbrot.im_limits[1] -= 0.1 * im_range;
                mandelbrot.calculate();
            }
            if input.key_pressed(VirtualKeyCode::Z) {
                let re_range = mandelbrot.re_limits[1] - mandelbrot.re_limits[0];
                mandelbrot.re_limits[0] -= 0.1 * re_range;
                mandelbrot.re_limits[1] += 0.1 * re_range;
                let im_range = mandelbrot.im_limits[1] - mandelbrot.im_limits[0];
                mandelbrot.im_limits[0] -= 0.1 * im_range;
                mandelbrot.im_limits[1] += 0.1 * im_range;
                mandelbrot.calculate();
            }
            if input.key_pressed(VirtualKeyCode::C) {
                mandelbrot = MandelbrotSet::new();
                mandelbrot.calculate();
            }
            if input.key_pressed(VirtualKeyCode::Add) {
                mandelbrot.max_iterations += 10;
                mandelbrot.calculate();
            }
            if input.key_pressed(VirtualKeyCode::Subtract) {
                if mandelbrot.max_iterations > 10 {
                    mandelbrot.max_iterations -= 10;
                }
                mandelbrot.calculate();
            }

            window.request_redraw();
        }
    });
}