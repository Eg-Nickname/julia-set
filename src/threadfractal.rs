use colorgrad;

use std::thread;
use std::sync::Arc;
use std::cell::SyncUnsafeCell;

use crate::fractals::{Fractal, BaseFractal};
use crate::{WIDTH, HEIGHT, MAX_ITERATION, R, SAMPLES_PER_LINE, SAMPLES_PER_PIXEL};


pub struct ThreadFractal {
    pub base: BaseFractal,
    display_frame: Arc<SyncUnsafeCell<Vec<Vec<u16>>>>,
}

impl ThreadFractal {
    pub fn new() -> Self {
        // Vec realocates when len == capacity, so vector capacity needs to be 1 bigger
        let mut display_vec = Vec::with_capacity(HEIGHT as usize + 1);

        for _w in 0..HEIGHT{
            let mut width_line: Vec<u16> = Vec::with_capacity(WIDTH as usize + 1);
            for _h in 0..WIDTH{
                width_line.push(0)
            }
            display_vec.push(width_line)
        }

        Self {
            base: BaseFractal::new(),
            display_frame: Arc::new(display_vec.into()),
        }
    }
}

impl Fractal for ThreadFractal{
    fn update_fractal(&mut self) {
        let zoom = self.base.zoom;
        let offset_x = self.base.offset_x;
        let offset_y = self.base.offset_y;

        let cx = self.base.cx;
        let cy = self.base.cy;

        let mut handles = vec![];

        for y in 0..HEIGHT{
            let safe = Arc::clone(&self.display_frame);
            let handle = thread::spawn(move|| {
                for x in 0..WIDTH{
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
                    unsafe {
                        let diplay =  &mut *safe.get();
                        diplay[y as usize][x as usize] = iterations_per_pixel / SAMPLES_PER_PIXEL as u16;
                    }
                    // *safe.get_mut(y as usize).unwrap().get_mut(x as usize).unwrap() = iterations_per_pixel / SAMPLES_PER_PIXEL as u16;
                    // *safe = iterations_per_pixel / SAMPLES_PER_PIXEL as u16;
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    fn draw(&self, frame: &mut [u8]) {
        let grad = colorgrad::cubehelix_default();
        let safe =  Arc::clone(&self.display_frame);

        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = i % WIDTH as usize;
            let y = i / WIDTH as usize;
            
            unsafe {
                let diplay =  &*safe.get();
                let rgba: [u8; 4] = grad.at(diplay[y as usize][x as usize] as f64 / MAX_ITERATION as f64).to_rgba8();
                pixel.copy_from_slice(&rgba);
            }
            

        }
    }
}