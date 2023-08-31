use draw::drawing::Drawing;

mod calendar;

pub trait Drawer {
    type Subject: ?Sized;

    fn draw(&self, subject: &Self::Subject, bounds: &DrawingBounds) -> Vec<Drawing>;
}

#[derive(Debug, Hash, Clone)]
pub struct DrawingBounds {
    pub left: u32,
    pub top: u32,
    pub width: u32,
    pub height: u32,
}

impl DrawingBounds {
    pub fn bottom(&self) -> u32 {
        return self.height + self.top;
    }

    pub fn right(&self) -> u32 {
        return self.left + self.width;
    }

    pub fn cropped_subshape(&self, relative_offset: DrawingBounds) -> DrawingBounds {
        DrawingBounds {
            left: self.left + relative_offset.left,
            top: self.top + relative_offset.top,
            width: self.width + relative_offset.width,
            height: self.height + relative_offset.height,
        }
    }

    pub fn crop(self, relative_offset: DrawingBounds) -> DrawingBounds {
        self.cropped_subshape(relative_offset)
    }
}
