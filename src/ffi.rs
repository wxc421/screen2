use crate::run_screenshot;

#[no_mangle]
pub extern "C" fn hello_from_rust() {
    println!("run_screenshot...");
    run_screenshot();
}