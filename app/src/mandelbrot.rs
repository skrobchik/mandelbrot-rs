use crate::color_functions::ColorFunction;
use num_complex::Complex64;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use std::sync::Arc;


pub struct MandelbrotSet {
    width: u32,
    height: u32,
    output_buffer: Vec<u8>,

    pub translation: [f64; 2],
    pub pixel_size: f64,

    pub max_iterations: u32,
    pub color_function: ColorFunction,
}

// #[cutile::module]
// mod my_module {
//     use cutile::core::*;
//
//     #[cutile::entry()]
//     fn mandelbrot<const S: [i32; 2]>(
//         s: &mut Tensor<u32, S>,  // Output: number of iterations
//         c_re: &Tensor<f32, {[-1, -1]}>,
//         c_im: &Tensor<f32, {[-1, -1]}>,
//         max_iterations: u32,
//     ) {
//         let mut n = 0.broadcast(s.shape());
//
//         let c_re = c_re.load_like(s);
//         let c_im = c_im.load_like(s);
//         let mut z_re = c_re;
//         let mut z_im = c_im;
//
//         for _ in 0..max_iterations {
//             let norm_sqr = z_re * z_re + z_im * z_im;
//             let reached = norm_sqr.lt_tile((4.0).broadcast(norm_sqr.shape()));
//
//             let new_z_re = z_re * z_re - z_im * z_im + c_re;
//             let new_z_im = (2.0).broadcast(z_im.shape()) * z_re * z_im + c_im;
//             z_re = select(reached, z_re, new_z_re);
//             z_im = select(reached, z_im, new_z_im);
//             n = select(reached, n, 1.broadcast(n.shape()) + n);
//         }
//         s.store(n);
//     }
// }

impl MandelbrotSet {
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.output_buffer.resize((width as usize) * (height as usize), 0);
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

    // pub fn calculate_gpu(&mut self) {
    //     let width = self.width;
    //     let height = self.height;
    //     let re_range = (width as f64) * self.pixel_size;
    //     let im_range = (height as f64) * self.pixel_size;
    //     let m_re = re_range / (width as f64);
    //     let m_im = im_range / (height as f64);
    //     let re0 = self.translation[0] - re_range / 2.0;
    //     let im0 = self.translation[1] - im_range / 2.0;
    //     let max_iterations = self.max_iterations;
    //
    //     let out = zeros::<u32>(&[width as usize, height as usize]).partition([4, 4]);
    //     let c_re = linspace(re0 as f32, re0 as f32 + re_range as f32, width as usize);
    //     let c_im = linspace(im0 as f32, im0 as f32 + im_range as f32, height as usize);
    //     my_module::mandelbrot(out, c_re, c_im, max_iterations).sync().unwrap();
    // }

    #[allow(dead_code)]
    pub fn calculate_cpu(&mut self) {
        let width = self.width;
        let height = self.height;
        let re_range = (width as f64) * self.pixel_size;
        let im_range = (height as f64) * self.pixel_size;
        let m_re = re_range / (width as f64);
        let m_im = im_range / (height as f64);
        let re0 = self.translation[0] - re_range / 2.0;
        let im0 = self.translation[1] - im_range / 2.0;
        let max_iterations = self.max_iterations;

        self.output_buffer
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, c)| {
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
            output_buffer: vec![0; (width as usize) * (height as usize)],
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
