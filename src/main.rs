mod mandelbrot;

use num::complex::Complex;
use std::time::{Instant};
#[macro_use]
extern crate glium;

use std::io::Cursor;
#[allow(unused_imports)]
use glium::{glutin, Surface};

fn main() {
    let start = Instant::now();
    let max_iter = 1024 as u16;
    let mut width_x = 5.;
    let mut center = Complex::new(-0.75, 0.);
    let mut im = mandelbrot::construct_mandelbrot_image(center, 1000, 1000, width_x, 32, 32, max_iter);
    let duration = start.elapsed();
    println!("Image construction took: {:?}", duration);
    im.save("test.png").unwrap();
    let image_dimensions = im.dimensions();

    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new().with_inner_size(glutin::dpi::PhysicalSize{width: image_dimensions.0, height: image_dimensions.1});
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let image = glium::texture::RawImage2d::from_raw_rgb(im.into_raw(), image_dimensions);
    let opengl_texture = glium::Texture2d::new(&display, image).unwrap();

    let dest_texture = glium::Texture2d::empty_with_format(&display,
                                                           glium::texture::UncompressedFloatFormat::U8U8U8U8,
                                                           glium::texture::MipmapsOption::NoMipmap,
                                                           image_dimensions.0, image_dimensions.1).unwrap();
    dest_texture.as_surface().clear_color(0.0, 0.0, 0.0, 1.0);


    event_loop.run(move |event, _, control_flow| {
        let (left, bottom, dimensions): (f32, f32, f32) = (0., 0., 1.);
        let dest_rect = glium::BlitTarget {
            left: (left * dest_texture.get_width() as f32) as u32,
            bottom: (bottom * dest_texture.get_height().unwrap() as f32) as u32,
            width: (dimensions * dest_texture.get_width() as f32) as i32,
            height: (dimensions * dest_texture.get_height().unwrap() as f32) as i32,
        };

        opengl_texture.as_surface().blit_whole_color_to(&dest_texture.as_surface(), &dest_rect,
                                                        glium::uniforms::MagnifySamplerFilter::Linear);

        let target = display.draw();
        dest_texture.as_surface().fill(&target, glium::uniforms::MagnifySamplerFilter::Linear);
        target.finish().unwrap();

        let next_frame_time = std::time::Instant::now() +
            std::time::Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                },
                glutin::event::WindowEvent::KeyboardInput {input, ..} => {
                    //println!("Keyboardevent! {:?}", input);
                    match input.virtual_keycode.unwrap() {
                        glutin::event::VirtualKeyCode::S => {
                            println!("S pressed!")
                        },
                        _ => {}
                    }
                },
                _ => return,
            },
            glutin::event::Event::NewEvents(cause) => match cause {
                glutin::event::StartCause::ResumeTimeReached { .. } => (),
                glutin::event::StartCause::Init => (),
                _ => return,
            },
            _ => return,
        }
    });
}