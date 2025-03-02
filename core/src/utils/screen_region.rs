use opencv::core::Rect;
extern crate serde;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
pub struct ScreenRegion {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

impl ScreenRegion {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        ScreenRegion {
            x,
            y,
            width,
            height,
        }
    }

    pub fn from_rect(rect: Rect) -> Self {
        ScreenRegion {
            x: rect.x,
            y: rect.y,
            width: rect.width as u32,
            height: rect.height as u32,
        }
    }

    pub fn values(&self) -> (i32, i32, u32, u32) {
        return (self.x, self.y, self.width, self.height);
    }
}
