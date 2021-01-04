#![forbid(unsafe_code)]

use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use num_complex::Complex64;

const RESOLUTION: usize = 400;

struct MandelbrotSet {
    set: [u8; RESOLUTION*RESOLUTION],
    re_limits: [f64; 2],
    im_limits: [f64; 2],
}

impl MandelbrotSet {
    pub fn mandelbrot(re: f64, im: f64) -> u8 {
        const MAX_ITERATIONS: u8 = 255;
        let mut n: u8 = 0;
        let c = Complex64::new(re, im);
        let mut z = Complex64::new(0.0, 0.0);
        while n < MAX_ITERATIONS && z.norm_sqr() < 4.0 {
            z = z.powu(2) + c;
            n += 1;
        }
        n
    }
    pub fn calculate(self: &mut MandelbrotSet) {
        for (i, c) in self.set.iter_mut().enumerate() {
            let x = i % RESOLUTION;
            let y = i / RESOLUTION;
            let re_range = self.re_limits[1] - self.re_limits[0];
            let im_range = self.im_limits[1] - self.im_limits[0];
            let re = self.re_limits[0] + re_range/(RESOLUTION as f64) * (x as f64);
            let im = self.im_limits[0] + im_range/(RESOLUTION as f64) * (y as f64);
            *c = MandelbrotSet::mandelbrot(re, im);
        }
    }
    pub fn new() -> MandelbrotSet {
        MandelbrotSet {
            set: [0; RESOLUTION*RESOLUTION],
            re_limits: [-2.0, 2.0],
            im_limits: [-2.0, 2.0],
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

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            //world.draw(pixels.get_frame());
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
            if let Some(size) = input.window_resized() {
                pixels.resize(size.width, size.height);
            }

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

            window.request_redraw();
        }
    });
}