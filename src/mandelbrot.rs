use num_complex::Complex64;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use crate::color_functions::ColorFunction;

pub struct MandelbrotSet {
    width: u32,
    height: u32,
    output_buffer: Vec<u8>,

    pub translation: [f64; 2],
    pub pixel_size: f64,

    pub max_iterations: u32,
    pub color_function: ColorFunction,
}

impl MandelbrotSet {
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.output_buffer.resize((width * height) as usize, 0);
    }

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
        let width = self.width;
        let height = self.height;
        let re_range = (width as f64) * self.pixel_size;
        let im_range = (height as f64) * self.pixel_size;
        let m_re = re_range / (width as f64);
        let m_im = im_range / (height as f64);
        let re0 = self.translation[0] - re_range / 2.0;
        let im0 = self.translation[1] - im_range / 2.0;
        let max_iterations = self.max_iterations;
        self.output_buffer.par_iter_mut().enumerate().for_each(|(i, c)| {
            let x = i % width as usize;
            let y = i / height as usize;
            let re = re0 + m_re * (x as f64);
            let im = im0 + m_im * (y as f64);
            *c = MandelbrotSet::normalize(
                MandelbrotSet::mandelbrot(re, im, max_iterations),
                max_iterations,
            );
        });
    }
    pub fn new(color_function: fn(u8) -> [u8; 3], width: u32, height: u32) -> Self {
        Self {
            output_buffer: vec![0; (width * height) as usize],
            translation: [0.0, 0.0],
            width,
            height,
            max_iterations: 255,
            color_function,
            pixel_size: 4.0 / (height as f64),
        }
    }

    pub fn draw(self: &MandelbrotSet, frame: &mut [u8]) {
        for (i, pixel) in frame.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            let c = self.output_buffer[i];
            let rgb = (self.color_function)(c);
            pixel.copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
    }
}