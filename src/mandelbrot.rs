use num::complex::Complex;
use image::{RgbImage};
use num::Integer;
use rayon::prelude::*;
use ndarray::prelude::*;
use std::cmp;

fn escape_time_optimized(c: Complex<f64>, max_iter: u16) -> u16 {
    let x0 = c.re;
    let y0 = c.im;

    // Test whether c lies in cardoid:
    let mut x_prime = x0 - 0.25;
    let y_prime = y0 * y0;
    let q = x_prime * x_prime +y_prime;
    if q * (q + x_prime) <= 0.25 * y_prime { return max_iter; }

    // Test whether c lies in period-2 bulb
    x_prime = x0 + 1.;
    if x_prime * x_prime + y_prime <= 0.0625 { return max_iter; }

    let mut x = 0.;
    let mut y = 0.;
    let mut x2 = 0.;
    let mut y2 = 0.;
    let mut ctr = 0 as u16;

    while ctr < max_iter && x2 + y2 < 4. {
        // TODO add periodicity checking
        ctr += 1;
        y = (x + x) * y + y0;
        x = x2 - y2 + x0;
        x2 = x * x;
        y2 = y * y
    }
    ctr
}

fn escape_time_chunk(corner: Complex<f64>, npix_x: usize, npix_y: usize, scale: f64, max_iter: u16) -> Array2<u16>{
    let escape_times: Array2::<u16> = Array::from_shape_fn(
        (npix_y, npix_x),
        |(i, j)| {
            escape_time_optimized(corner + Complex::new(j as f64, i as f64) * scale, max_iter)
        });
    escape_times
}

fn color_escape_time_array(arr: Array2<u16>, max_iter: u16) -> Array3<u8>{
    let (h, w) = arr.dim();
    Array3::<u8>::from_shape_fn((h, w, 3), |(i, j, k)| {
        palette(arr[[i, j]], k, max_iter)
    })
}

fn get_chunk_corners(npix_x: usize, npix_y: usize, chunk_w: usize, chunk_h: usize) -> Vec<Complex<f64>>{
    let n_chunks = (npix_x.div_ceil(&chunk_w)) * (npix_y.div_ceil(&chunk_h));
    let mut chunk_corners = vec![Complex::new(0., 0.); n_chunks];

    let mut cx = 0 as usize;
    let mut cy = 0 as usize;
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
    chunk_corners
}

fn construct_mandelbrot_array(center: Complex<f64>, npix_x: usize, npix_y: usize, width_x: f64,
                              chunk_w: usize, chunk_h: usize, max_iter: u16) -> Array3<u8> {
    let scale = width_x / (npix_x as f64);
    let center_pix_x = npix_x as f64 / 2.;
    let center_pix_y = npix_y as f64 / 2.;
    let img_corner = center - Complex::new(center_pix_x * scale, center_pix_y * scale);

    let chunk_corners = get_chunk_corners(npix_x, npix_y, chunk_w, chunk_h);
    let chunk_escape_times: Vec<Array3<u8>> = chunk_corners.par_iter()
        .map(|chunk_corner|{
            let escape_time_chunk = escape_time_chunk(img_corner + *chunk_corner * scale, chunk_w, chunk_h, scale, max_iter);
            color_escape_time_array(escape_time_chunk, max_iter)
        }).collect();

    let mut pixels = Array3::<u8>::zeros((npix_y, npix_x, 3));

    for (chunk_corner, chunk_escape_times) in chunk_corners.iter().zip(chunk_escape_times.iter()){
        let x = chunk_corner.re as usize;
        let x_end = cmp::min(npix_x, x + chunk_w);
        let y = chunk_corner.im as usize;
        let y_end = cmp::min(npix_y, y + chunk_h);
        pixels.slice_mut(s![y..y_end, x..x_end, ..])
            .assign(&chunk_escape_times.slice(s![..y_end - y, ..x_end - x, ..]));
    }
    pixels
}

pub fn construct_mandelbrot_image(center: Complex<f64>, npix_x: usize, npix_y: usize, width_x: f64,
                              chunk_w: usize, chunk_h: usize, max_iter: u16) -> RgbImage{
    let pixels = construct_mandelbrot_array(center, npix_x, npix_y, width_x, chunk_w, chunk_h, max_iter);
    arr_to_image(pixels)
}

fn palette(iteration: u16, channel: usize, max_iter: u16) -> u8 {
    if iteration == 0 || iteration == max_iter {
        return 0;
    }
    let n_colors = 16;
    let color_map: [[u8; 3]; 16] = [
        [66, 30, 15],
        [25, 7, 26],
        [9, 1, 47],
        [4, 4, 73],
        [0, 7, 100],
        [12, 44, 138],
        [24, 82, 177],
        [57, 125, 209],
        [134, 181, 229],
        [211, 236, 248],
        [241, 233, 191],
        [248, 201, 95],
        [255, 170, 0],
        [204, 128, 0],
        [153, 87, 0],
        [106, 52, 3],
    ];
    return color_map[(iteration % n_colors) as usize][channel];
}

fn arr_to_image(arr: Array3<u8>) -> RgbImage {
    assert!(arr.is_standard_layout());

    let (height, width, _) = arr.dim();
    let raw = arr.into_raw_vec();

    return RgbImage::from_raw(width as u32, height as u32, raw)
        .expect("container should have the right size for the image dimensions")
}
