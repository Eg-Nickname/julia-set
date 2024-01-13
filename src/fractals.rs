pub struct BaseFractal{
    pub cx: f64,
    pub cy: f64,
    pub zoom: f64,
    pub offset_x: i32,
    pub offset_y: i32
}

impl BaseFractal {
    pub fn new() -> Self {
        Self {
            cx: -0.8,
            cy: 0.156,
            zoom: 2.0,
            offset_x:0,
            offset_y:0,
        }
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
}

pub trait Fractal{
    fn update_fractal(&mut self);

    fn draw(&self, frame: &mut [u8]);
}