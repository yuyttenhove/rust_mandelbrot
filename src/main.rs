mod mandelbrot;

use num::complex::Complex;
use std::time::{Instant};



fn main() {
    let start = Instant::now();
    let max_iter = 1024 as u16;
    let im = mandelbrot::construct_mandelbrot_image(Complex::new(-0.75, 0.), 1000, 1000, 5., 32, 32, max_iter);
    let duration = start.elapsed();
    println!("Image construction took: {:?}", duration);
    im.save("test.png").unwrap();
}