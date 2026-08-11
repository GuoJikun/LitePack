//! LitePack Shell Extension — Windows 11 右键菜单 COM DLL
//!
//! 实现 IExplorerCommand 接口，通过 MSIX `windows.fileExplorerContextMenus`
//! 扩展注册到 Win11 一级右键菜单。
//!
//! 导出函数：
//! - DllGetClassObject: COM 运行时调用，获取类工厂
//! - DllCanUnloadNow: COM 运行时调用，判断是否可卸载

#![cfg(windows)]
#![allow(non_snake_case)] // COM vtable 字段使用 PascalCase，与 Windows API 一致

mod command;
mod factory;
mod guid;
mod launch;

use std::ffi::c_void;

use guid::*;

// ─── DllMain ──────────────────────────────────────────────────────────────────

const DLL_PROCESS_ATTACH: u32 = 1;

#[no_mangle]
unsafe extern "system" fn DllMain(
    hinst: *mut c_void,
    reason: u32,
    _reserved: *mut c_void,
) -> i32 {
    if reason == DLL_PROCESS_ATTACH {
        launch::set_module_handle(hinst as usize);
    }
    1 // TRUE
}

// ─── COM DLL 导出 ─────────────────────────────────────────────────────────────

/// COM 运行时入口：根据 CLSID 返回对应的 IClassFactory。
///
/// Explorer（或 dllhost.exe）调用此函数获取类工厂，
/// 再通过 IClassFactory::CreateInstance 创建 IExplorerCommand。
#[no_mangle]
unsafe extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    *ppv = std::ptr::null_mut();

    // 只响应我们自己的 CLSID
    if rclsid.is_null() || !(*rclsid).eq(&CLSID_LITEPACKSHELL) {
        return E_FAIL; // CLASS_E_CLASSNOTAVAILABLE
    }

    // 只支持 IClassFactory / IUnknown
    if !iid_matches(riid, &IID_ICLASSFACTORY) && !iid_matches(riid, &IID_IUNKNOWN) {
        return E_NOINTERFACE;
    }

    let factory = factory::create_class_factory();
    if factory.is_null() {
        return E_OUTOFMEMORY;
    }

    *ppv = factory;
    S_OK
}

/// COM 运行时查询：DLL 中是否还有未释放的对象。
/// 返回 S_OK 表示可以卸载，S_FALSE 表示仍有活跃对象。
#[no_mangle]
unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    if command::global_refcount() == 0 {
        S_OK
    } else {
        S_FALSE
    }
}
