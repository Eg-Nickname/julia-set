pub trait Fractal{
    fn zoom(&mut self);

    fn move_fractal(&mut self);

    fn change_offset(&mut self, off_x: i32, off_y: i32);

    fn update_fractal(&mut self);

    fn draw(&self, frame: &mut [u8]);
}