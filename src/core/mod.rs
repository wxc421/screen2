pub mod core;
mod image;

use crate::core::image::Image;


#[cfg(target_os = "macos")]
mod darwin;

use display_info::DisplayInfo;
#[cfg(target_os = "macos")]
use darwin::*;

#[cfg(target_os = "windows")]
mod win32;


#[cfg(target_os = "windows")]
use win32::*;


#[derive(Debug, Clone, Copy)]
pub struct Screen {
    pub display_info: DisplayInfo,
}

impl Screen {
    pub fn new(display_info: &DisplayInfo) -> Self {
        Screen {
            display_info: *display_info,
        }
    }

    pub fn all() -> Result<Vec<Screen>, E> {
        let screens = DisplayInfo::all()?.iter().map(Screen::new).collect();
        Ok(screens)
    }

    pub fn from_point(x: i32, y: i32) -> Result<Screen, E> {
        let display_info = DisplayInfo::from_point(x, y)?;
        Ok(Screen::new(&display_info))
    }

    pub fn capture(&self) -> Result<Image, E> {
        capture_screen(&self.display_info)
    }
}
