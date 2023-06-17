#![deny(clippy::all)]


use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;



const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

use std::time::Instant;



pub mod fractal;

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Julia set visualisation")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    let mut fractal = fractal::Fractal::new();

    let mut zoom: bool = false;
    let mut move_fractal: bool = false;
    let mut loops: u64 = 0;

    event_loop.run(move |event, _, control_flow| {
        let time = Instant::now();

        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            fractal.draw(pixels.get_frame_mut());
            if let Err(err) = pixels.render() {
                error!("pixels.render() failed: {err}");
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    error!("pixels.resize_surface() failed: {err}");
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }
            // key inputs
            if input.key_pressed(VirtualKeyCode::Up) || input.quit() {
                fractal.change_offset(0, 10)
            }
            if input.key_pressed(VirtualKeyCode::Down) || input.quit() {
                fractal.change_offset(0, -10)
            }

            if input.key_pressed(VirtualKeyCode::Left) || input.quit() {
                fractal.change_offset(-10, 0)
            }
            if input.key_pressed(VirtualKeyCode::Right) || input.quit() {
                fractal.change_offset(10, 0)
            }

            // Pausing zooming
            if input.key_pressed(VirtualKeyCode::Z) || input.quit() {
                zoom = !zoom;
            }
            if input.key_pressed(VirtualKeyCode::M) || input.quit() {
                move_fractal = !move_fractal;
            }
            

            if zoom{
                fractal.zoom();
            }
            if move_fractal{
                fractal.move_fractal();
            }

            fractal.update_fractal();

            loops += 1;
            println!("Time per loop: {:?}, Avg iterations per loop: {}, Fractal iterations: {},", time.elapsed(), fractal.get_iterations()/loops, fractal.get_iterations());


            window.request_redraw();
        }
    });
}