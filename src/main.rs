#![forbid(unsafe_code)]

mod app;
mod mandelbrot;
mod color_functions;

use std::error::Error;
use winit::event_loop::EventLoop;
use app::App;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let event_loop = EventLoop::new()?;
    let mut app = App(None);
    event_loop.run_app(&mut app)?;
    Ok(())
}
