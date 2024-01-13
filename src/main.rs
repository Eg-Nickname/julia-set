#![deny(clippy::all)]
#![feature(sync_unsafe_cell)]


use log::error;
use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;



const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

const MAX_ITERATION: u32 = 500;
const R: f64 = 2.0;

const SAMPLES_PER_LINE: usize = 2;
const SAMPLES_PER_PIXEL: usize = SAMPLES_PER_LINE * SAMPLES_PER_LINE; 

use std::time::Instant;


pub mod fractals;
use crate::fractals::Fractal;

// Diffrent julia set fractal implementation
mod threadmutexfractal;
mod threadfractal;
mod rayonfractal;


fn main() -> Result<(), pixels::Error> {
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

    // let mut fractal = threadmutexfractal::ThreadMutexFractal::new();
    let mut fractal = rayonfractal::RayonFractal::new();

    let mut zoom: bool = false;
    let mut move_fractal: bool = false;

    event_loop.run(move |event, _, control_flow| {
        let time = Instant::now();

        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            fractal.draw(pixels.frame_mut());
            if let Err(err) = pixels.render() {
                error!("pixels.render() failed: {err}");
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.close_requested() {
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
            if input.key_pressed(VirtualKeyCode::Up) {
                fractal.base.change_offset(0, 10)
            }
            if input.key_pressed(VirtualKeyCode::Down) {
                fractal.base.change_offset(0, -10)
            }

            if input.key_pressed(VirtualKeyCode::Left) {
                fractal.base.change_offset(-10, 0)
            }
            if input.key_pressed(VirtualKeyCode::Right) {
                fractal.base.change_offset(10, 0)
            }

            // Pausing zooming
            if input.key_pressed(VirtualKeyCode::Z) {
                zoom = !zoom;
            }
            if input.key_pressed(VirtualKeyCode::M) {
                move_fractal = !move_fractal;
            }
            

            if zoom{
                fractal.base.zoom();
            }
            if move_fractal{
                fractal.base.move_fractal();
            }

            fractal.update_fractal();

            // Log time took to draw this fractal
            println!("Time per loop: {:?}", time.elapsed());

            window.request_redraw();
        }
    });
}