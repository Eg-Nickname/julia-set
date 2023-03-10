#![deny(clippy::all)]


use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use colorgrad;
use std::time::Instant;

use std::thread;
use std::sync::{Mutex, Arc};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

const MAX_ITERATION: u32 = 500;
const R: f64 = 2.0;

const SAMPLES_PER_LINE: usize = 4;
const SAMPLES_PER_PIXEL: usize = SAMPLES_PER_LINE * SAMPLES_PER_LINE; 


struct Fractal {
    display_frame: Vec<Vec<u16>>,
    cx: f64,
    cy: f64,
    zoom: f64,
    offset_x: i32,
    offset_y: i32,
    iterations: u64,
}

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

    let mut fractal = Fractal::new();
    let mut paused: bool = false;
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
            if input.key_pressed(VirtualKeyCode::Space) || input.quit() {
                paused = !paused;
            }
            

            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    error!("pixels.resize_surface() failed: {err}");
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }

            // Update internal state and request a redraw
            if !paused{
                fractal.zoom();
            }
            fractal.update_fractal();

            loops += 1;

            println!("Time per loop: {:?}, Avg iterations per loop: {}, Fractal iterations: {},", time.elapsed(), fractal.iterations/loops, fractal.iterations);


            window.request_redraw();
        }
    });
}


impl Fractal {

    fn new() -> Self {

        let mut empty_set:  Vec<Vec<u16>> = [].to_vec();

        for _w in 0..WIDTH{
            let mut width_line: Vec<u16> = [].to_vec();
            for _h in 0..HEIGHT{
                width_line.push(0)
            }
            empty_set.push(width_line)
        }
        Self {
            display_frame: empty_set,
            cx: -0.8,
            cy: 0.156,
            zoom: 2.0,
            offset_x:0,
            offset_y:0,
            iterations: 0
        }
    }
    fn zoom(&mut self){
        self.zoom = self.zoom * 0.85;
    }
    fn change_offset(&mut self, off_x: i32, off_y: i32){
        self.offset_x += off_x;
        self.offset_y += off_y;
    }

    fn update_fractal(&mut self) {
        let mut return_vec: Vec<Vec<u16>> = [].to_vec();

        for _w in 0..WIDTH{
            let mut width_line: Vec<u16> = [].to_vec();
            for _h in 0..HEIGHT{
                width_line.push(0)
            }
            return_vec.push(width_line)
        }

        let zoom = self.zoom;
        let offset_x = self.offset_x;
        let offset_y = self.offset_y;

        let cx = self.cx;
        let cy = self.cy;

        let safe = Arc::new(Mutex::new(return_vec));
        let mut handles = vec![];

        for x in 0..WIDTH{
            let safe = Arc::clone(&safe);
            let handle = thread::spawn(move|| {
                for y in 0..HEIGHT{
                    let mut iterations_per_pixel: u16 = 0;
                    for i in 0..SAMPLES_PER_PIXEL{

                        let mut zx: f64 = ((x as i32 + offset_x) as f64 + (i % SAMPLES_PER_LINE) as f64 / SAMPLES_PER_LINE as f64)/(WIDTH as f64/(2.0*zoom)) - zoom;
                        let mut zy: f64 = ((y as i32 + offset_y) as f64 + (i / SAMPLES_PER_LINE) as f64 / SAMPLES_PER_LINE as f64)/(WIDTH as f64/(2.0*zoom)) - zoom/2.0;

                        let mut iteration: u16 = 0;
                        while zx * zx + zy * zy < R*R && iteration < MAX_ITERATION as u16 {
                            let xtemp = zx * zx - zy * zy;
                            zy = 2.0 * zx * zy  + cy;
                            zx = xtemp + cx;
        
                            iteration += 1;
                        }
                        iterations_per_pixel += iteration;
                    }

                    let mut temp_return_vec = safe.lock().unwrap();
                    *temp_return_vec.get_mut(x as usize).unwrap().get_mut(y as usize).unwrap() = iterations_per_pixel / SAMPLES_PER_PIXEL as u16;
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let messages = safe.lock().unwrap();
        for x in 0..WIDTH{
            for y in 0..HEIGHT{
                self.display_frame[x as usize][y as usize] = *messages.get(x as usize).unwrap().get(y as usize).unwrap() as u16;
                self.iterations += (self.display_frame[x as usize][y as usize] * 16) as u64;
            }
        }
    }

    fn draw(&self, frame: &mut [u8]) {
        let grad = colorgrad::cubehelix_default();

        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = i % WIDTH as usize;
            let y = i / WIDTH as usize;

            let rgba: [u8; 4] = grad.at(self.display_frame[x][y] as f64 / MAX_ITERATION as f64).to_rgba8();

            pixel.copy_from_slice(&rgba);
        }
    }
}