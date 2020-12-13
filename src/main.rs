use num::complex::Complex;
use image::{RgbImage};
use std::time::{Instant};
use num::Integer;
use rayon::prelude::*;
use ndarray::prelude::*;
use std::cmp;

fn escape_time(c: Complex<f64>, max_iter: u16) -> u16 {
    let mut p = Complex::new(0.0, 0.0);
    let mut norm_sqr = p.norm_sqr();
    let mut ctr = 0 as u16;

    while ctr < max_iter && norm_sqr < 4. {
        ctr += 1;
        p = p * p + c;
        norm_sqr = p.norm_sqr();
    }
    return ctr;
}

fn escape_time_chunk(corner: Complex<f64>, npix_x: u32, npix_y: u32, scale: f64, max_iter: u16) -> Array2<u16>{
    let escape_times: Array2::<u16> = Array::from_shape_fn(
        (npix_y as usize, npix_x as usize),
        |(i, j)| {
            return escape_time(corner + Complex::new(j as f64, i as f64) * scale, max_iter);
        });
    return escape_times;
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

fn escape_time_array(center: Complex<f64>, npix_x: u32, npix_y: u32, width_x: f64,
                     chunk_w: u32, chunk_h: u32, max_iter: u16) -> Array2<u16>{
    let scale = width_x / (npix_x as f64);
    let center_pix_x = npix_x as f64 / 2.;
    let center_pix_y = npix_y as f64 / 2.;
    let img_corner = center - Complex::new(center_pix_x * scale, center_pix_y * scale);

    let chunk_corners = get_chunk_corners(npix_x, npix_y, chunk_w, chunk_h);
    let chunk_escape_times: Vec<Array2<u16>> = chunk_corners.par_iter()
        .map(|chunk_corner|{
            return escape_time_chunk(img_corner + *chunk_corner * scale, chunk_w, chunk_h, scale, max_iter)
        }).collect();

    let mut escape_times = Array2::<u16>::zeros((npix_y as usize, npix_x as usize));

    for (chunk_corner, chunk_escape_times) in chunk_corners.iter().zip(chunk_escape_times.iter()){
        let x = chunk_corner.re as usize;
        let x_end = cmp::min(npix_x as usize, x + chunk_w as usize);
        let y = chunk_corner.im as usize;
        let y_end = cmp::min(npix_y as usize, y + chunk_h as usize);
        escape_times.slice_mut(s![y..y_end, x..x_end])
            .assign(&chunk_escape_times.slice(s![..y_end - y, ..x_end - x]));
    }
    //return escape_times.mapv(|v| v as u32);
    return escape_times;
}

fn palette(hue: f64, channel: usize) -> u8 {
    if hue == 0. {
        return 0;
    }
    let n_colors = 16.;
    let color_map: Array2<u8> = arr2(&[
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
    ]);
    return color_map[[(hue % (n_colors - 1.)) as usize, channel]];
}

fn arr_to_image(arr: Array3<u8>) -> RgbImage {
    assert!(arr.is_standard_layout());

    let (height, width, _) = arr.dim();
    let raw = arr.into_raw_vec();

    return RgbImage::from_raw(width as u32, height as u32, raw)
        .expect("container should have the right size for the image dimensions")
}

fn escape_time_to_image(escape_times: Array2<u16>, max_iter: u16) -> RgbImage {
    let (n_rows, n_cols) = escape_times.dim();
    let mut n_pixels_per_n_iterations = Array1::<u32>::zeros((max_iter) as usize);
    escape_times.iter().for_each(|v| {
        if *v < max_iter {
            n_pixels_per_n_iterations[*v as usize] += 1;
        }
    });
    let total = n_pixels_per_n_iterations.sum() as f64;
    n_pixels_per_n_iterations.accumulate_axis_inplace(Axis(0), |&prev, curr| *curr += prev);
    let hue_array = Array2::<f64>::from_shape_fn(
        escape_times.dim(),
        |(i, j)| {
            let v = escape_times[[i, j]];
            let hue;
            if v == max_iter {
                hue = 0.;
            } else {
                let helper = n_pixels_per_n_iterations[v as usize];
                hue = v as f64;
            }
            return hue;
        }
    );

    let arr = Array3::<u8>::from_shape_fn(
        (n_rows, n_cols, 3),
        |(i, j, k)| { return palette(hue_array[[i, j]], k); }
    );
    return arr_to_image(arr);
}

fn main() {
    let mut start = Instant::now();
    let max_iter = 1024 as u16;
    let arr = escape_time_array(Complex::new(-0.75, 0.05), 1000, 1000, 3., 32, 32, max_iter);
    let duration = start.elapsed();
    println!("Array construction took: {:?}", duration);
    start = Instant::now();
    let im = escape_time_to_image(arr, max_iter);
    let duration = start.elapsed();
    println!("Image construction took: {:?}", duration);
    im.save("test.png").unwrap();

}