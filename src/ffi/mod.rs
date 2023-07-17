use crate::run_screenshot;

mod windows;
mod macos;
mod linux;


#[no_mangle]
pub extern "C" fn c_run_screenshot() {
    println!("run_screenshot...");
    run_screenshot();
}

#[no_mangle]
pub extern "C" fn c_screenshot_2_pngs() -> {
    println!("run_screenshot...");
    run_screenshot();
}

#[no_mangle]
pub extern "C" fn c_run_screenshot() {
    println!("run_screenshot...");
    run_screenshot();
}