use window::Window;


pub trait Layout {
    fn layout(&self, width: i32, height: i32, stack: &mut [Window]) -> Result<(), String>;
}


pub struct TiledLayout;

impl Layout for TiledLayout {
    fn layout(&self, width: i32, height: i32, stack: &mut [Window]) -> Result<(), String> {
        if stack.len() == 0 {
            return Ok(());
        }

        let tile_height = height / stack.len() as i32;

        for (i, window) in stack.iter_mut().enumerate() {
            window.position(0, i as i32 * tile_height, width, tile_height)?;
        }

        Ok(())
    }
}
