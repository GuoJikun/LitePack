// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if litepack_cli::is_cli_subcommand() {
        // GUI 二进制设了 windows_subsystem = "windows"，进程不会继承父控制台。
        // 从 cmd / PowerShell 调用时需要 AttachConsole 重新附着，否则 stdout/stderr 无处输出。
        #[cfg(windows)]
        unsafe {
            use windows_sys::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
            let _ = AttachConsole(ATTACH_PARENT_PROCESS);
        }
        let code = litepack_cli::run_cli();
        std::process::exit(code);
    }

    litepack_gui_lib::run();
}
