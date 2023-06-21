mod support;
mod util;
mod ffi;
mod core;

fn run_screenshot() {
    support::run();
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_run_screenshot() {
        crate::core::core::run();
    }
}