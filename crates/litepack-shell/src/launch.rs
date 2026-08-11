//! 从 IShellItemArray 提取文件路径，并启动 LitePack.exe。

use std::ffi::c_void;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::guid::*;

// ─── 全局 HMODULE（DllMain 中设置）────────────────────────────────────────────

static G_MODULE: AtomicUsize = AtomicUsize::new(0);

pub fn set_module_handle(h: usize) {
    G_MODULE.store(h, Ordering::SeqCst);
}

// ─── IShellItemArray / IShellItem vtable（仅定义需要调用的方法）─────────────────

#[repr(C)]
pub struct IShellItemArrayVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut c_void) -> u32,
    pub Release: unsafe extern "system" fn(*mut c_void) -> u32,
    pub BindToHandler: unsafe extern "system" fn(
        *mut c_void,
        *mut c_void,
        *const GUID,
        *const GUID,
        *mut *mut c_void,
    ) -> HRESULT,
    pub GetPropertyStore:
        unsafe extern "system" fn(*mut c_void, i32, *const GUID, *mut *mut c_void) -> HRESULT,
    pub GetPropertyDescriptionList: unsafe extern "system" fn(
        *mut c_void,
        *const c_void,
        *const GUID,
        *mut *mut c_void,
    ) -> HRESULT,
    pub GetAttributes: unsafe extern "system" fn(*mut c_void, u32, u32, *mut u32) -> HRESULT,
    pub GetCount: unsafe extern "system" fn(*mut c_void, *mut u32) -> HRESULT,
    pub GetAt: unsafe extern "system" fn(*mut c_void, u32, *mut *mut c_void) -> HRESULT,
    pub EnumItems: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HRESULT,
}

#[repr(C)]
pub struct IShellItemArray {
    pub lpVtbl: *const IShellItemArrayVtbl,
}

#[repr(C)]
pub struct IShellItemVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut c_void) -> u32,
    pub Release: unsafe extern "system" fn(*mut c_void) -> u32,
    pub BindToHandler: unsafe extern "system" fn(
        *mut c_void,
        *mut c_void,
        *const GUID,
        *const GUID,
        *mut *mut c_void,
    ) -> HRESULT,
    pub GetParent: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HRESULT,
    pub GetDisplayName: unsafe extern "system" fn(*mut c_void, u32, *mut *mut u16) -> HRESULT,
    pub GetAttributes: unsafe extern "system" fn(*mut c_void, u32, *mut u32) -> HRESULT,
    pub Compare: unsafe extern "system" fn(*mut c_void, *mut c_void, u32, *mut i32) -> HRESULT,
}

#[repr(C)]
pub struct IShellItem {
    pub lpVtbl: *const IShellItemVtbl,
}

// ─── 路径提取 ─────────────────────────────────────────────────────────────────

/// 从 IShellItemArray 中取第一个文件的文件系统路径。
pub fn get_first_file_path(psi_item_array: *mut c_void) -> Option<String> {
    if psi_item_array.is_null() {
        return None;
    }

    unsafe {
        let array = &*(psi_item_array as *const IShellItemArray);

        let mut count: u32 = 0;
        let hr = ((*array.lpVtbl).GetCount)(psi_item_array, &mut count);
        if hr != S_OK || count == 0 {
            return None;
        }

        let mut p_item: *mut c_void = std::ptr::null_mut();
        let hr = ((*array.lpVtbl).GetAt)(psi_item_array, 0, &mut p_item);
        if hr != S_OK || p_item.is_null() {
            return None;
        }

        let item = &*(p_item as *const IShellItem);
        let mut p_path: *mut u16 = std::ptr::null_mut();
        let hr = ((*item.lpVtbl).GetDisplayName)(p_item, SIGDN_FILESYSPATH, &mut p_path);

        // 释放 IShellItem
        ((*item.lpVtbl).Release)(p_item);

        if hr != S_OK || p_path.is_null() {
            return None;
        }

        let path = wide_to_string(p_path);
        windows_sys::Win32::System::Com::CoTaskMemFree(p_path as *const c_void);

        if path.is_empty() {
            None
        } else {
            Some(path)
        }
    }
}

// ─── 定位 LitePack.exe ────────────────────────────────────────────────────────

/// 获取当前 DLL 所在目录。
fn get_dll_dir() -> Option<PathBuf> {
    let hmodule = G_MODULE.load(Ordering::SeqCst);
    if hmodule == 0 {
        return None;
    }

    let mut buf = vec![0u16; 32768];
    let len = unsafe {
        windows_sys::Win32::System::LibraryLoader::GetModuleFileNameW(
            hmodule as *mut c_void,
            buf.as_mut_ptr(),
            buf.len() as u32,
        )
    };
    if len == 0 {
        return None;
    }

    let path = String::from_utf16_lossy(&buf[..len as usize]);
    PathBuf::from(path).parent().map(|p| p.to_path_buf())
}

/// 在 DLL 同目录下查找 LitePack 主程序。
pub fn get_exe_path() -> Option<PathBuf> {
    let dir = get_dll_dir()?;

    // MSIX 包内统一命名为 LitePack.exe
    let candidates = ["LitePack.exe", "litepack-gui.exe"];
    for name in candidates {
        let p = dir.join(name);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

/// 获取图标路径（exe 路径 + ",0" 表示第一个图标资源）。
pub fn get_icon_path() -> Option<String> {
    let exe = get_exe_path()?;
    Some(format!("{},0", exe.display()))
}

// ─── 启动 LitePack ────────────────────────────────────────────────────────────

/// 启动 LitePack.exe 执行指定操作。
///
/// action: "--open" | "--extract-here" | "--extract-to" | "--extract-named"
/// file_path: 目标归档文件路径
pub fn launch(action: &str, file_path: &str) -> bool {
    let exe = match get_exe_path() {
        Some(p) => p,
        None => return false,
    };

    std::process::Command::new(&exe)
        .arg(action)
        .arg(file_path)
        .spawn()
        .is_ok()
}
