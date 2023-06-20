mod support;
mod util;
mod ffi;
mod core;

fn run_screenshot() {
    support::run();
}


#[cfg(test)]
mod tests {
    use crate::run_screenshot;

    #[test]
    fn test_run_screenshot() {
        run_screenshot();
    }
}