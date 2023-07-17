mod util;
mod ffi;
mod core;

fn run_screenshot() {
    core::run();
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_run_screenshot() {
        crate::core::core::run();
    }
}