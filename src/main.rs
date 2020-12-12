use std::f64::consts::PI;
use num::complex::Complex;
use image::DynamicImage;

fn main() {
    let x = Complex::new(0.0, 2.0*PI);

    let n = x.norm_sqr();

    let im = DynamicImage::new_rgb8(200, 150);
    im.save("test.png");

    println!("e^(2i * pi) = {}", x.exp()); // =~1
    println!("norm squared is {}", n);
    println!("norm is: {}", n.sqrt());
}