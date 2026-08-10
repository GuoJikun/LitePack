//! Windows 资源管理器右键菜单注册（仅 Windows，其余平台为 no-op）。
//!
//! 注册四种动词：
//! - LitePackOpen → 打开/浏览归档
//! - LitePackExtractHere → 解压到当前文件夹
//! - LitePackExtractTo → 解压到...（弹出对话框）
//! - LitePackExtractNamed → 解压到同名目录

#[cfg(windows)]
mod imp {
    use std::io;
    use std::ptr;

    use windows_sys::Win32::UI::Shell::{
        SHChangeNotify, SHCNE_ASSOCCHANGED, SHCNF_IDLIST,
    };
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    const SHELL_KEY: &str = r"Software\Classes";
    const EXTS: [&str; 2] = [".zip", ".7z"];

    /// 右键菜单动词定义：(名称, 显示文本, 启动参数标志)
    const VERBS: [(&str, &str, &str); 4] = [
        ("LitePackOpen", "用 LitePack 打开(&O)", "--open"),
        ("LitePackExtractHere", "解压到当前文件夹(&X)", "--extract-here"),
        ("LitePackExtractTo", "解压到...(&E)...", "--extract-to"),
        ("LitePackExtractNamed", "解压到同名目录(&N)", "--extract-named"),
    ];

    fn verb_path(progid: &str, verb_name: &str) -> String {
        format!("{SHELL_KEY}\\{progid}\\shell\\{verb_name}")
    }

    /// SystemFileAssociations 路径：与默认程序无关，始终随扩展名生效，
    /// 也是 Win11 下最可靠的注册位置。
    fn sfa_path(ext: &str, verb_name: &str) -> String {
        format!("{SHELL_KEY}\\SystemFileAssociations\\{ext}\\shell\\{verb_name}")
    }

    /// 某个动词需要注册的路径（去重）。
    ///
    /// 仅使用 SystemFileAssociations 与扩展名自身两处：
    /// - SFA 覆盖已关联默认程序（WinRAR/7-Zip 等）的情况；
    /// - 扩展名自身覆盖未关联、以扩展名作为 ProgID 的情况。
    /// 不再写入其他程序的 ProgID 键，避免残留/覆盖风险。
    fn verb_target_paths(verb_name: &str) -> Vec<String> {
        let mut paths: Vec<String> = Vec::new();
        for ext in EXTS {
            for p in [sfa_path(ext, verb_name), verb_path(ext, verb_name)] {
                if !paths.contains(&p) {
                    paths.push(p);
                }
            }
        }
        paths
    }

    /// 所有需要注册的路径（去重）。
    pub fn all_target_paths() -> Vec<String> {
        let mut paths: Vec<String> = Vec::new();
        for (verb_name, _, _) in VERBS {
            for p in verb_target_paths(verb_name) {
                if !paths.contains(&p) {
                    paths.push(p);
                }
            }
        }
        paths
    }

    fn command_line(exe: &std::path::Path, flag: &str) -> String {
        format!("\"{}\" {flag} \"%1\"", exe.display())
    }

    /// 通知 Explorer 文件关联已变化，刷新右键菜单缓存。
    fn notify_shell() {
        unsafe {
            SHChangeNotify(SHCNE_ASSOCCHANGED as i32, SHCNF_IDLIST, ptr::null(), ptr::null());
        }
    }

    /// 注册 .zip/.7z 右键菜单全部动词。
    pub fn register() -> Result<(), String> {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        for (verb_name, display, flag) in VERBS {
            let cmd = command_line(&exe, flag);
            let display_str = display.to_string();
            for path in verb_target_paths(verb_name) {
                let (verb, _) = hkcu.create_subkey(&path).map_err(|e| e.to_string())?;
                verb.set_value("", &display_str).map_err(|e| e.to_string())?;
                verb.set_value("Icon", &format!("{},0", exe.display()))
                    .map_err(|e| e.to_string())?;
                let (cmd_key, _) = verb.create_subkey("command").map_err(|e| e.to_string())?;
                cmd_key.set_value("", &cmd).map_err(|e| e.to_string())?;
            }
        }
        notify_shell();
        Ok(())
    }

    /// 移除 .zip/.7z 右键菜单全部动词。
    pub fn unregister() -> Result<(), String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        for path in all_target_paths() {
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
        all_target_paths().iter().any(|p| hkcu.open_subkey(p).is_ok())
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
        let verbs = ["--open", "--extract-here", "--extract-to", "--extract-named"];
        for path in imp::all_target_paths() {
            let cmd: String = hkcu
                .open_subkey(format!("{path}\\command"))
                .expect("command 键应存在")
                .get_value("")
                .expect("command 默认值应存在");
            assert!(
                verbs.iter().any(|v| cmd.contains(v)),
                "命令应包含有效参数: {cmd}"
            );
        }

        imp::unregister().expect("注销应成功");
        assert!(!imp::is_registered(), "注销后不应仍注册");
    }
}
