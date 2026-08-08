//! Windows 资源管理器右键菜单「直接解压」注册（仅 Windows，其余平台为 no-op）。

#[cfg(windows)]
mod imp {
    use std::io;
    use std::ptr;

    use windows_sys::Win32::UI::Shell::{
        SHChangeNotify, SHCNE_ASSOCCHANGED, SHCNF_IDLIST,
    };
    use winreg::enums::{HKEY_CLASSES_ROOT, HKEY_CURRENT_USER};
    use winreg::RegKey;

    const SHELL_KEY: &str = r"Software\Classes";
    const VERB_NAME: &str = "LitePackExtractHere";
    const VERB_DISPLAY: &str = "直接解压(&X)";
    const EXTS: [&str; 2] = [".zip", ".7z"];

    fn verb_path(progid: &str) -> String {
        format!(r"{SHELL_KEY}\{progid}\shell\{VERB_NAME}")
    }

    /// 解析扩展名的实际 ProgID（通过 HKCR 合并视图读取，HKCU 优先于 HKLM）。
    /// 扩展名键默认值为空或不存在时，直接以扩展名自身作为 ProgID。
    fn resolve_progid(ext: &str) -> String {
        let key = match RegKey::predef(HKEY_CLASSES_ROOT).open_subkey(format!(r"{SHELL_KEY}\{ext}")) {
            Ok(k) => k,
            Err(_) => return ext.to_string(),
        };
        match key.get_value::<String, _>("") {
            Ok(p) if !p.is_empty() => p,
            _ => ext.to_string(),
        }
    }

    /// SystemFileAssociations 路径：与默认程序无关，始终随扩展名生效，
    /// 也是 Win11 下最可靠的注册位置。
    fn sfa_path(ext: &str) -> String {
        format!(r"{SHELL_KEY}\SystemFileAssociations\{ext}\shell\{VERB_NAME}")
    }

    /// 注册动词的目标路径（去重）：
    /// - SystemFileAssociations\.ext（首选，稳定）
    /// - 扩展名自身
    /// - 解析出的 ProgID
    pub fn target_paths() -> Vec<String> {
        let mut paths: Vec<String> = Vec::new();
        for ext in EXTS {
            for p in [sfa_path(ext), verb_path(ext), verb_path(&resolve_progid(ext))] {
                if !paths.contains(&p) {
                    paths.push(p);
                }
            }
        }
        paths
    }

    fn command_line(exe: &std::path::Path) -> String {
        format!("\"{}\" --extract-here \"%1\"", exe.display())
    }

    /// 通知 Explorer 文件关联已变化，刷新右键菜单缓存。
    fn notify_shell() {
        unsafe {
            SHChangeNotify(SHCNE_ASSOCCHANGED as i32, SHCNF_IDLIST, ptr::null(), ptr::null());
        }
    }

    /// 注册 .zip/.7z 右键菜单「直接解压」。
    pub fn register() -> Result<(), String> {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let cmd = command_line(&exe);
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        for path in target_paths() {
            let (verb, _disp) = hkcu.create_subkey(&path).map_err(|e| e.to_string())?;
            verb.set_value("", &VERB_DISPLAY).map_err(|e| e.to_string())?;
            verb.set_value("Icon", &format!("{},0", exe.display()))
                .map_err(|e| e.to_string())?;
            let (cmd_key, _) = verb.create_subkey("command").map_err(|e| e.to_string())?;
            cmd_key.set_value("", &cmd).map_err(|e| e.to_string())?;
        }
        notify_shell();
        Ok(())
    }

    /// 移除 .zip/.7z 右键菜单「直接解压」。
    pub fn unregister() -> Result<(), String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        for path in target_paths() {
            match hkcu.delete_subkey_all(&path) {
                Ok(()) => {}
                Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.to_string()),
            }
        }
        notify_shell();
        Ok(())
    }

    /// 是否已注册（以目标路径是否存在为准）。
    pub fn is_registered() -> bool {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        target_paths().iter().any(|p| hkcu.open_subkey(p).is_ok())
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
        for path in imp::target_paths() {
            let cmd: String = hkcu
                .open_subkey(format!(r"{path}\command"))
                .expect("command 键应存在")
                .get_value("")
                .expect("command 默认值应存在");
            assert!(
                cmd.contains("--extract-here"),
                "命令应包含 --extract-here: {cmd}"
            );
        }

        imp::unregister().expect("注销应成功");
        assert!(!imp::is_registered(), "注销后不应仍注册");
    }
}
