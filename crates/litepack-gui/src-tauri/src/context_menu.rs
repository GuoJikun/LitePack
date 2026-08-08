//! Windows 资源管理器右键菜单「直接解压」注册（仅 Windows，其余平台为 no-op）。

#[cfg(windows)]
mod imp {
    use std::io;

    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    const SHELL_KEY: &str = r"Software\Classes";
    const VERB_NAME: &str = "LitePackExtractHere";
    const VERB_DISPLAY: &str = "直接解压(&X)";
    const EXTS: [&str; 2] = [".zip", ".7z"];

    fn verb_path(ext: &str) -> String {
        format!(r"{SHELL_KEY}\{ext}\shell\{VERB_NAME}")
    }

    fn command_line(exe: &std::path::Path) -> String {
        format!("\"{}\" --extract-here \"%1\"", exe.display())
    }

    /// 注册 .zip/.7z 右键菜单「直接解压」。
    pub fn register() -> Result<(), String> {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let cmd = command_line(&exe);
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        for ext in EXTS {
            let (verb, _disp) = hkcu
                .create_subkey(verb_path(ext))
                .map_err(|e| e.to_string())?;
            verb.set_value("", &VERB_DISPLAY).map_err(|e| e.to_string())?;
            verb.set_value("Icon", &format!("{},0", exe.display()))
                .map_err(|e| e.to_string())?;
            let (cmd_key, _) = verb.create_subkey("command").map_err(|e| e.to_string())?;
            cmd_key.set_value("", &cmd).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// 移除 .zip/.7z 右键菜单「直接解压」。
    pub fn unregister() -> Result<(), String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        for ext in EXTS {
            match hkcu.delete_subkey_all(verb_path(ext)) {
                Ok(()) => {}
                Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.to_string()),
            }
        }
        Ok(())
    }

    /// 是否已注册（以 .zip 键是否存在为准）。
    pub fn is_registered() -> bool {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        hkcu.open_subkey(verb_path(".zip")).is_ok()
    }
}

#[cfg(not(windows))]
mod imp {
    /// 非 Windows 平台返回错误。
    pub fn register() -> Result<(), String> {
        Err("当前平台不支持右键菜单注册".to_string())
    }

    /// 非 Windows 平台无需处理。
    pub fn unregister() -> Result<(), String> {
        Ok(())
    }

    /// 非 Windows 平台恒为未注册。
    pub fn is_registered() -> bool {
        false
    }
}

pub use imp::{is_registered, register, unregister};

#[cfg(all(windows, test))]
mod tests {
    use super::imp;

    #[test]
    fn register_unregister_roundtrip() {
        let _ = imp::unregister();
        assert!(!imp::is_registered(), "残留注册应被清理");

        imp::register().expect("注册应成功");

        let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let cmd: String = hkcu
            .open_subkey(r"Software\Classes\.zip\shell\LitePackExtractHere\command")
            .expect(".zip command 键应存在")
            .get_value("")
            .expect("command 默认值应存在");
        assert!(cmd.contains("--extract-here"), "命令应包含 --extract-here: {cmd}");

        imp::unregister().expect("注销应成功");
        assert!(!imp::is_registered(), "注销后不应仍注册");
    }
}
