use crate::support::init_event_loop;

#[cfg(windows)]
#[no_mangle]
pub extern "system" fn DllMain(
    _hinst_dll: *mut u8,
    fdw_reason: u32,
    _lpv_reserved: *mut u8,
) -> u32 {
    match fdw_reason {
        1 /* DLL_PROCESS_ATTACH */ => {
            // DLL 被加载时执行的操作
            println!("DLL loaded");
        }
        0 /* DLL_PROCESS_DETACH */ => {
            // DLL 被卸载时执行的操作
            println!("DLL unloaded");
        }
        _ => {}
    }

    1 // 返回值为非零表示执行成功
}