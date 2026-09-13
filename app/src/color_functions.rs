/// A ColorFunction converts a normalized iterations value to an RGB color
pub type ColorFunction = fn(u8) -> [u8; 3];

fn hsv_to_rgb(hsv: [u8; 3]) -> [u8; 3] {
    let h: f32 = (hsv[0] as f32) / 255.0 * 360.0;
    let s: f32 = (hsv[1] as f32) / 255.0;
    let v: f32 = (hsv[2] as f32) / 255.0;
    let f = |n: f32| {
        let u = n + h / 60.0;
        let k = u - (u / 6.0).floor() * 6.0;
        v - v * s * k.min(4.0 - k).clamp(0.0, 1.0)
    };
    let r = 255.0 * f(5.0);
    let g = 255.0 * f(3.0);
    let b = 255.0 * f(1.0);
    [r as u8, g as u8, b as u8]
}

pub fn rainbow(normalized_iterations: u8) -> [u8; 3] {
    hsv_to_rgb([normalized_iterations, 255, 255])
}

#[allow(dead_code)]
pub fn grayscale(normalized_iterations: u8) -> [u8; 3] {
    [
        normalized_iterations,
        normalized_iterations,
        normalized_iterations,
    ]
}
