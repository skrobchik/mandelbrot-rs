use crate::color_functions::ColorFunction;
use krnl::buffer::Buffer;
use krnl::device::Device;
use krnl::macros::module;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

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
    use core::ops::Add;
    #[cfg(not(target_arch = "spirv"))]
    use krnl::krnl_core;
    use krnl_core::macros::kernel;
    #[cfg(target_arch = "spirv")]
    use krnl_core::num_traits::Float;

    #[derive(Clone, Copy)]
    struct Complex {
        re: f64,
        im: f64,
    }
    impl Complex {
        fn new(re: f64, im: f64) -> Complex {
            Complex { re, im }
        }
        fn norm_sqr(&self) -> f64 {
            self.re * self.re + self.im * self.im
        }
        fn pow2(&self) -> Complex {
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

    fn normalize(iterations: u32, max_iterations: u32) -> u8 {
        ((iterations as f32) / (max_iterations as f32) * 255.0).round() as u8
    }

    fn mandelbrot_iterations(max_iterations: u32, re: f64, im: f64) -> u32 {
        let mut n = 0;
        let c = Complex::new(re, im);
        let mut z = Complex::new(0.0, 0.0);
        while n <= max_iterations && z.norm_sqr() < 4.0 {
            z = z.pow2() + c;
            n += 1;
        }
        n
    }

    pub fn mandelbrot_impl(
        width: u32,
        height: u32,
        max_iterations: u32,
        re0: f64,
        m_re: f64,
        im0: f64,
        m_im: f64,
        i: usize,
    ) -> u8 {
        let x = i % width as usize;
        let y = i / height as usize;
        let re = re0 + m_re * (x as f64);
        let im = im0 + m_im * (y as f64);
        normalize(
            mandelbrot_iterations(max_iterations, re, im),
            max_iterations,
        )
    }

    #[kernel]
    pub fn mandelbrot(
        width: u32,
        height: u32,
        max_iterations: u32,
        re0: f64,
        m_re: f64,
        im0: f64,
        m_im: f64,
        #[item] normalized_iterations: &mut u8,
    ) {
        *normalized_iterations = mandelbrot_impl(
            width,
            height,
            max_iterations,
            re0,
            m_re,
            im0,
            m_im,
            kernel.item_id(),
        );
    }
}

impl MandelbrotSet {
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.output_buffer
            .resize((width as usize) * (height as usize), 0);
    }

    pub fn calculate(&mut self, gpu: Option<Device>) {
        let width = self.width;
        let height = self.height;
        let re_range = (width as f64) * self.pixel_size;
        let im_range = (height as f64) * self.pixel_size;
        let m_re = re_range / (width as f64);
        let m_im = im_range / (height as f64);
        let re0 = self.translation[0] - re_range / 2.0;
        let im0 = self.translation[1] - im_range / 2.0;
        let max_iterations = self.max_iterations;

        if let Some(gpu) = gpu.clone() {
            let mut output_buffer =
                Buffer::from_elem(gpu.clone(), width as usize * height as usize, 0_u8).unwrap();
            kernels::mandelbrot::builder()
                .unwrap()
                .build(gpu)
                .unwrap()
                .dispatch(
                    width,
                    height,
                    max_iterations,
                    re0,
                    m_re,
                    im0,
                    m_im,
                    output_buffer.as_slice_mut(),
                )
                .unwrap();
            std::mem::swap(&mut self.output_buffer, &mut output_buffer.into_vec().unwrap());
        } else {
            self.output_buffer
                .par_iter_mut()
                .enumerate()
                .for_each(|(i, c)| {
                    *c = kernels::mandelbrot_impl(
                        width,
                        height,
                        max_iterations,
                        re0,
                        m_re,
                        im0,
                        m_im,
                        i,
                    );
                });
        }
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
