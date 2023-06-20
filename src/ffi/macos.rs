#[cfg(target_os = "macos")]
#[ctor]
fn init() {
    // DLL 被加载时执行的操作
}

#[cfg(target_os = "macos")]
#[dtor]
fn cleanup() {
    // DLL 被卸载时执行的操作
}