use num_complex::Complex64;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use crate::color_functions::ColorFunction;

pub const RESOLUTION: u32 = 800;

pub struct MandelbrotSet {
    set: [u8; (RESOLUTION * RESOLUTION) as usize],
    pub(crate) re_limits: [f64; 2],
    pub(crate) im_limits: [f64; 2],
    pub(crate) max_iterations: u32,
    pub(crate) color_function: ColorFunction,
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
    pub fn calculate(&mut self) {
        let re_range = self.re_limits[1] - self.re_limits[0];
        let im_range = self.im_limits[1] - self.im_limits[0];
        let m_re = re_range / (RESOLUTION as f64);
        let m_im = im_range / (RESOLUTION as f64);
        let re0 = self.re_limits[0];
        let im0 = self.im_limits[0];
        let max_iterations = self.max_iterations;
        self.set.par_iter_mut().enumerate().for_each(|(i, c)| {
            let x = i % RESOLUTION as usize;
            let y = i / RESOLUTION as usize;
            let re = re0 + m_re * (x as f64);
            let im = im0 + m_im * (y as f64);
            *c = MandelbrotSet::normalize(
                MandelbrotSet::mandelbrot(re, im, max_iterations),
                max_iterations,
            );
        });
    }
    pub fn new(color_function: fn(u8) -> [u8; 3]) -> Self {
        Self {
            set: [0; (RESOLUTION * RESOLUTION) as usize],
            re_limits: [-2.0, 2.0],
            im_limits: [-2.0, 2.0],
            max_iterations: 255,
            color_function,
        }
    }

    /// Asumes 4*RESOLUTION*RESOLUTION size
    pub fn draw(self: &MandelbrotSet, frame: &mut [u8]) {
        for (i, pixel) in frame.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            let c = self.set[i];
            let rgb = (self.color_function)(c);
            pixel.copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
    }
}