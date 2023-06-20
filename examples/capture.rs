use screenshots::Screen;
use std::{fs, time::Instant};

fn main() {
    let start = Instant::now();
    let screens = Screen::all().unwrap();

    for screen in screens {
        // println!("capturer {screen:?}");
        let mut image = screen.capture().unwrap();
        let aaa: Vec<u8> = image.into();
        // println!("aaa:{:?}", &aaa[0..80]);
        // let mut buffer = image.to_png().unwrap();
        // fs::write(format!("target/777777-{}.png", screen.display_info.id), buffer).unwrap();

        // image = screen.capture_area(300, 300, 300, 300).unwrap();
        // buffer = image.to_png().unwrap();
        // fs::write(format!("target/{}-2.png", screen.display_info.id), buffer).unwrap();
    }

    // let screen = Screen::from_point(100, 100).unwrap();
    // println!("capturer {screen:?}");
    //
    // let image = screen.capture_area(300, 300, 300, 300).unwrap();
    // let buffer = image.to_png().unwrap();
    // fs::write("target/capture_display_with_point.png", buffer).unwrap();

    println!("运行耗时: {:?}", start.elapsed());
}