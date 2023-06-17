use colorgrad;

use std::thread;
use std::sync::{Mutex, Arc};

use crate::{WIDTH, HEIGHT};

const MAX_ITERATION: u32 = 500;
const R: f64 = 2.0;

const SAMPLES_PER_LINE: usize = 2;
const SAMPLES_PER_PIXEL: usize = SAMPLES_PER_LINE * SAMPLES_PER_LINE; 


pub struct Fractal {
    display_frame: Vec<Vec<u16>>,
    cx: f64,
    cy: f64,
    zoom: f64,
    offset_x: i32,
    offset_y: i32,
    iterations: u64,
}

impl Fractal {
    pub fn new() -> Self {
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

    pub fn get_iterations(&self) -> u64{
        self.iterations
    }
    pub fn zoom(&mut self){
        self.zoom = self.zoom * 0.85;
    }

    pub fn move_fractal(&mut self){
        self.cx += 0.01;
        self.cy += -0.02;

        if self.cx > 1.0 {
            self.cx = -0.999;
        }else if self.cx < -1.0 {
            self.cx = 0.999;
        }

        if self.cy > 1.0 {
            self.cy = -0.998;
        }else if self.cy < -1.0 {
            self.cy = 0.998;
        }
    }

    pub fn change_offset(&mut self, off_x: i32, off_y: i32){
        self.offset_x += off_x;
        self.offset_y += off_y;
    }

    pub fn update_fractal(&mut self) {
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

    pub fn draw(&self, frame: &mut [u8]) {
        let grad = colorgrad::cubehelix_default();

        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = i % WIDTH as usize;
            let y = i / WIDTH as usize;

            let rgba: [u8; 4] = grad.at(self.display_frame[x][y] as f64 / MAX_ITERATION as f64).to_rgba8();

            pixel.copy_from_slice(&rgba);
        }
    }
}