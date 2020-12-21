mod mandelbrot;

use num::complex::Complex;
use glium::{glutin, Surface};

fn get_mandelbrot_opengl_texture(display: &glium::Display, center: Complex<f64>, image_dimensions: (u32, u32), width_x: f64)
    -> glium::Texture2d {
    let max_iter = 1024 as u16;
    let chunk_w = 32;
    let chunk_h = 32;
    let im = mandelbrot::construct_mandelbrot_image(center, image_dimensions.0 as usize, image_dimensions.1 as usize, width_x, chunk_w, chunk_h, max_iter);
    glium::Texture2d::new(display,
                          glium::texture::RawImage2d::from_raw_rgb(im.into_raw(),
                                                                   image_dimensions))
        .unwrap()
}

fn print_help_message(){
    println!("Press I to zoom in, O to move out and use the arrow keys to move around.");
    println!("Press S to save a high resolution image of the current view.");
}

fn save_high_res_mandelbrot_image(center: Complex<f64>, width_x: f64) {
    let max_iter = 4096 as u16;
    let chunk_w = 32;
    let chunk_h = 32;
    println!("Generating high res image");
    let im = mandelbrot::construct_mandelbrot_image(center, 5760, 3240, width_x, chunk_w, chunk_h, max_iter);
    im.save(format!("mandelbrot_({:.3e}, {:.3e})_{:.3e}.png", center.re, center.im, width_x)).unwrap();
    println!("High res image saved!");
}

fn main() {
    print_help_message();
    let mut width_x = 5.;
    let mut center = Complex::new(-0.75, 0.);
    let movement_scale = 0.01;
    let image_dimensions = (1600, 1000);

    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Mandelbrot explorer")
        .with_inner_size(glutin::dpi::PhysicalSize{width: image_dimensions.0, height: image_dimensions.1});
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let mut opengl_texture = get_mandelbrot_opengl_texture(&display, center, image_dimensions, width_x);

    let dest_texture = glium::Texture2d::empty_with_format(&display,
                                                           glium::texture::UncompressedFloatFormat::U8U8U8U8,
                                                           glium::texture::MipmapsOption::NoMipmap,
                                                           image_dimensions.0, image_dimensions.1).unwrap();
    dest_texture.as_surface().clear_color(0.0, 0.0, 0.0, 1.0);
    let (left, bottom, dimensions): (f32, f32, f32) = (0., 1., 1.);
    let dest_rect = glium::BlitTarget {
        left: (left * dest_texture.get_width() as f32) as u32,
        bottom: (bottom * dest_texture.get_height().unwrap() as f32) as u32,
        width: (dimensions * dest_texture.get_width() as f32) as i32,
        height: (-1. * dimensions * dest_texture.get_height().unwrap() as f32) as i32,
    };

    let mut saved_location = false;
    event_loop.run(move |event, _, control_flow| {
        opengl_texture.as_surface().blit_whole_color_to(&dest_texture.as_surface(), &dest_rect,
                                                        glium::uniforms::MagnifySamplerFilter::Linear);

        let target = display.draw();
        dest_texture.as_surface().fill(&target, glium::uniforms::MagnifySamplerFilter::Linear);
        target.finish().unwrap();

        let next_frame_time = std::time::Instant::now() +
            std::time::Duration::from_nanos(33_333_333);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                },
                glutin::event::WindowEvent::KeyboardInput {input, ..} => {
                    if input.state == glutin::event::ElementState::Pressed {
                        match input.virtual_keycode.unwrap() {
                            glutin::event::VirtualKeyCode::Up => {
                                center -= Complex::new(0., movement_scale * width_x);
                                opengl_texture = get_mandelbrot_opengl_texture(&display, center, image_dimensions, width_x);
                                saved_location = false;
                            },
                            glutin::event::VirtualKeyCode::Down => {
                                center += Complex::new(0., movement_scale * width_x);
                                opengl_texture = get_mandelbrot_opengl_texture(&display, center, image_dimensions, width_x);
                                saved_location = false;
                            },
                            glutin::event::VirtualKeyCode::Left => {
                                center -= Complex::new(movement_scale * width_x, 0.);
                                opengl_texture = get_mandelbrot_opengl_texture(&display, center, image_dimensions, width_x);
                                saved_location = false;
                            },
                            glutin::event::VirtualKeyCode::Right => {
                                center += Complex::new(movement_scale * width_x, 0.);
                                opengl_texture = get_mandelbrot_opengl_texture(&display, center, image_dimensions, width_x);
                                saved_location = false;
                            },
                            glutin::event::VirtualKeyCode::I => {
                                width_x *= 0.95;
                                opengl_texture = get_mandelbrot_opengl_texture(&display, center, image_dimensions, width_x);
                                saved_location = false;

                            },
                            glutin::event::VirtualKeyCode::O => {
                                width_x /= 0.95;
                                opengl_texture = get_mandelbrot_opengl_texture(&display, center, image_dimensions, width_x);
                                saved_location = false;
                            },
                            glutin::event::VirtualKeyCode::S => {
                                if !saved_location {
                                    saved_location = true;
                                    save_high_res_mandelbrot_image(center, width_x);
                                }
                            },
                            _ => {
                                print_help_message();
                            }
                        }
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