use crate::color_functions::ColorFunction;
use num_complex::Complex64;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use krnl::macros::module;

pub struct MandelbrotSet {
    width: u32,
    height: u32,
    output_buffer: Vec<u8>,

    pub translation: [f64; 2],
    pub pixel_size: f64,

    pub max_iterations: u32,
    pub color_function: ColorFunction,
}

#[module]
mod kernels {
    #[cfg(not(target_arch = "spirv"))]
    use krnl::krnl_core;
    use krnl_core::macros::kernel;
    use core::ops::Add;

    #[derive(Clone, Copy)]
    pub struct Complex {
        re: f64,
        im: f64,
    }
    impl Complex {
        pub fn new(re: f64, im: f64) -> Complex {
            Complex {
                re,
                im,
            }
        }
        pub fn norm_sqr(&self) -> f64 {
            self.re * self.re + self.im * self.im
        }
        pub fn pow2(&self) -> Complex {
            Complex {
                re: self.re * self.re - self.im * self.im,
                im: 2.0 * self.re * self.im,
            }
        }
    }

    impl Add for Complex {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            Complex {
                re: self.re + rhs.re,
                im: self.im + rhs.im,
            }
        }
    }

    pub fn mandelbrot_impl(max_iterations: u32, re: f64, im: f64, iterations: &mut u32) {
        let mut n = 0;
        let c = Complex::new(re, im);
        let mut z = Complex::new(0.0, 0.0);
        while n <= max_iterations && z.norm_sqr() < 4.0 {
            z = z.pow2() + c;
            n += 1;
        }
        *iterations = n
    }

    #[kernel]
    pub fn mandelbrot(max_iterations: u32, #[item] re: f64, #[item] im: f64, #[item] iterations: &mut u32) {
        mandelbrot_impl(max_iterations, re, im, iterations);
    }
}

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
                let mut iterations = 0;
                kernels::mandelbrot_impl(max_iterations, re, im, &mut iterations);
                *c = MandelbrotSet::normalize(
                    iterations,
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
