use crate::run_screenshot;

mod windows;
mod macos;
mod linux;


#[no_mangle]
pub extern "C" fn hello_from_rust() {
    println!("run_screenshot...");
    run_screenshot();
}