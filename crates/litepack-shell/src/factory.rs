//! IClassFactory 实现 — COM 类工厂，创建 IExplorerCommand 实例。

use std::ffi::c_void;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::command::create_parent_command;
use crate::guid::*;

// ─── IClassFactory vtable ─────────────────────────────────────────────────────

#[repr(C)]
pub struct IClassFactoryVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut c_void) -> u32,
    pub Release: unsafe extern "system" fn(*mut c_void) -> u32,
    pub CreateInstance: unsafe extern "system" fn(
        *mut c_void,
        *mut c_void,
        *const GUID,
        *mut *mut c_void,
    ) -> HRESULT,
    pub LockServer: unsafe extern "system" fn(*mut c_void, i32) -> HRESULT,
}

#[repr(C)]
pub struct ClassFactory {
    pub lpVtbl: *const IClassFactoryVtbl,
    pub ref_count: AtomicU32,
}

static CLASS_FACTORY_VTBL: IClassFactoryVtbl = IClassFactoryVtbl {
    QueryInterface: class_factory_query_interface,
    AddRef: class_factory_add_ref,
    Release: class_factory_release,
    CreateInstance: class_factory_create_instance,
    LockServer: class_factory_lock_server,
};

static FACTORY_REFCOUNT: AtomicU32 = AtomicU32::new(0);

// ─── 构造 ─────────────────────────────────────────────────────────────────────

pub fn create_class_factory() -> *mut c_void {
    let factory = Box::new(ClassFactory {
        lpVtbl: &CLASS_FACTORY_VTBL,
        ref_count: AtomicU32::new(1),
    });
    FACTORY_REFCOUNT.fetch_add(1, Ordering::SeqCst);
    Box::into_raw(factory) as *mut c_void
}

// ─── IUnknown ─────────────────────────────────────────────────────────────────

unsafe extern "system" fn class_factory_query_interface(
    this: *mut c_void,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }

    if iid_matches(riid, &IID_IUNKNOWN) || iid_matches(riid, &IID_ICLASSFACTORY) {
        *ppv = this;
        class_factory_add_ref(this);
        S_OK
    } else {
        *ppv = std::ptr::null_mut();
        E_NOINTERFACE
    }
}

unsafe extern "system" fn class_factory_add_ref(this: *mut c_void) -> u32 {
    let f = &*(this as *const ClassFactory);
    f.ref_count.fetch_add(1, Ordering::SeqCst) + 1
}

unsafe extern "system" fn class_factory_release(this: *mut c_void) -> u32 {
    let f = &*(this as *const ClassFactory);
    let new_count = f.ref_count.fetch_sub(1, Ordering::SeqCst) - 1;
    if new_count == 0 {
        FACTORY_REFCOUNT.fetch_sub(1, Ordering::SeqCst);
        drop(Box::from_raw(this as *mut ClassFactory));
    }
    new_count
}

// ─── IClassFactory 方法 ───────────────────────────────────────────────────────

unsafe extern "system" fn class_factory_create_instance(
    _this: *mut c_void,
    p_unk_outer: *mut c_void,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    *ppv = std::ptr::null_mut();

    // 不支持聚合
    if !p_unk_outer.is_null() {
        return 0x8004_0110u32 as i32; // CLASS_E_NOAGGREGATION
    }

    // 只支持创建 IExplorerCommand
    if !iid_matches(riid, &IID_IEXPLORERCOMMAND) && !iid_matches(riid, &IID_IUNKNOWN) {
        *ppv = std::ptr::null_mut();
        return E_NOINTERFACE;
    }

    let cmd = create_parent_command();
    if cmd.is_null() {
        return E_OUTOFMEMORY;
    }

    *ppv = cmd;
    S_OK
}

unsafe extern "system" fn class_factory_lock_server(_this: *mut c_void, f_lock: i32) -> HRESULT {
    if f_lock != 0 {
        FACTORY_REFCOUNT.fetch_add(1, Ordering::SeqCst);
    } else {
        FACTORY_REFCOUNT.fetch_sub(1, Ordering::SeqCst);
    }
    S_OK
}
