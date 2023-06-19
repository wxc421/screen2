mod support;

#[no_mangle]
pub extern "C" fn hello_from_rust() {
    support::run();
}