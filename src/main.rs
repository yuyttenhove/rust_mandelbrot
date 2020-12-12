use num::complex::Complex;
use image::{RgbImage, Rgb, imageops};
use std::time::{Instant};
use num::Integer;
use rayon::prelude::*;

fn escape_time(c: Complex<f64>, max_iter: u32) -> u32 {
    let mut p = Complex::new(0.0, 0.0);
    let mut norm_sqr = p.norm_sqr();
    let mut ctr = 0 as u32;

    while ctr < max_iter && norm_sqr < 4. {
        ctr += 1;
        p = p * p + c;
        norm_sqr = p.norm_sqr();
    }
    return ctr;
}

fn escape_time_region(corner: Complex<f64>, npix_x: u32, npix_y: u32, scale: f64) -> RgbImage{
    let mut im = RgbImage::new(npix_x, npix_y);
    for (x, y, pixel) in im.enumerate_pixels_mut(){
        let c = corner + Complex::new((x as f64) * scale, (y as f64) * scale);
        let escape_time = escape_time(c, 4096) as u8;
        *pixel = Rgb([escape_time, escape_time, escape_time]);
    }
    return im;
}

fn get_chunk_corners(npix_x: u32, npix_y: u32, chunk_w: u32, chunk_h: u32) -> Vec<Complex<f64>>{
    let n_chunks = ((npix_x.div_ceil(&chunk_w)) * (npix_y.div_ceil(&chunk_h))) as usize;
    let mut chunk_corners = vec![Complex::new(0., 0.); n_chunks];

    let mut cx = 0 as u32;
    let mut cy = 0 as u32;
    let mut i = 0;
    while cx < npix_x {
        while cy < npix_y {
            chunk_corners[i] = Complex::new(cx as f64, cy as f64);
            cy += chunk_h;
            i += 1;
        }
        cx += chunk_w;
        cy = 0;
    }
    return chunk_corners
}

fn escape_time_image(center: Complex<f64>, npix_x: u32, npix_y: u32, width_x: f64,
                     chunk_w: u32, chunk_h: u32) -> RgbImage{
    let scale = width_x / (npix_x as f64);
    let center_pix_x = npix_x as f64 / 2.;
    let center_pix_y = npix_y as f64 / 2.;
    let img_corner = center - Complex::new(center_pix_x * scale, center_pix_y * scale);

    let chunk_corners = get_chunk_corners(npix_x, npix_y, chunk_w, chunk_h);
    let mut chunk_imgs = vec![RgbImage::new(chunk_w, chunk_h); chunk_corners.len()];

    chunk_corners.par_iter().zip(chunk_imgs.par_iter_mut()).for_each(|(chunk_corner, chunk_img)|{
        *chunk_img = escape_time_region(img_corner + *chunk_corner*scale, chunk_w, chunk_h, scale);
    });

    let mut canvas: RgbImage = RgbImage::new(npix_x, npix_y);
    for (chunk_corner, chunk_img) in chunk_corners.iter().zip(chunk_imgs.iter()){
        imageops::replace(&mut canvas, chunk_img, chunk_corner.re as u32, chunk_corner.im as u32);
    }
    return canvas;
}

fn main() {
    let start = Instant::now();
    let im = escape_time_image(Complex::new(-0.75, 0.05), 1000, 1000, 0.01, 32, 32);
    let duration = start.elapsed();
    println!("Image construction took: {:?}", duration);
    im.save("test.png").unwrap();

}