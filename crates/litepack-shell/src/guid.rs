//! GUID 定义与 COM 接口 IID/CLSID 常量。

pub type HRESULT = i32;

pub const S_OK: HRESULT = 0;
pub const S_FALSE: HRESULT = 1;
pub const E_NOTIMPL: HRESULT = 0x8000_4001u32 as i32;
pub const E_NOINTERFACE: HRESULT = 0x8000_4002u32 as i32;
pub const E_POINTER: HRESULT = 0x8000_4003u32 as i32;
pub const E_FAIL: HRESULT = 0x8000_4005u32 as i32;
pub const E_OUTOFMEMORY: HRESULT = 0x8007_000Eu32 as i32;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GUID {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

impl GUID {
    pub fn eq(&self, other: &GUID) -> bool {
        self.data1 == other.data1
            && self.data2 == other.data2
            && self.data3 == other.data3
            && self.data4 == other.data4
    }
}

// ─── 标准 IID ────────────────────────────────────────────────────────────────

/// IUnknown {00000000-0000-0000-C000-000000000046}
pub const IID_IUNKNOWN: GUID = GUID {
    data1: 0x0000_0000,
    data2: 0x0000,
    data3: 0x0000,
    data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
};

/// IClassFactory {00000001-0000-0000-C000-000000000046}
pub const IID_ICLASSFACTORY: GUID = GUID {
    data1: 0x0000_0001,
    data2: 0x0000,
    data3: 0x0000,
    data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
};

// ─── Shell IID ───────────────────────────────────────────────────────────────

/// IExplorerCommand {A5C4B8CF-E100-4509-B50D-2A570112495D}
pub const IID_IEXPLORERCOMMAND: GUID = GUID {
    data1: 0xA5C4_B8CF,
    data2: 0xE100,
    data3: 0x4509,
    data4: [0xB5, 0x0D, 0x2A, 0x57, 0x01, 0x12, 0x49, 0x5D],
};

/// IEnumExplorerCommand {A8844EAB-5BFE-4E26-B5C5-5C5E6B3E7C3B}
pub const IID_IENUMEXPLORERCOMMAND: GUID = GUID {
    data1: 0xA884_4EAB,
    data2: 0x5BFE,
    data3: 0x4E26,
    data4: [0xB5, 0xC5, 0x5C, 0x5E, 0x6B, 0x3E, 0x7C, 0x3B],
};

/// IShellItemArray {B63EA76D-1F85-456F-A19C-48156EFA8E5A}
#[allow(dead_code)]
pub const IID_ISHELLITEMARRAY: GUID = GUID {
    data1: 0xB63E_A76D,
    data2: 0x1F85,
    data3: 0x456F,
    data4: [0xA1, 0x9C, 0x48, 0x15, 0x6E, 0xFA, 0x8E, 0x5A],
};

/// IShellItem {43826D1E-E718-42EE-BC55-A1E261C37BFE}
#[allow(dead_code)]
pub const IID_ISHELLITEM: GUID = GUID {
    data1: 0x4382_6D1E,
    data2: 0xE718,
    data3: 0x42EE,
    data4: [0xBC, 0x55, 0xA1, 0xE2, 0x61, 0xC3, 0x7B, 0xFE],
};

// ─── LitePack CLSID ──────────────────────────────────────────────────────────

/// CLSID_LitePackShell {7B2E4A1F-9C3D-4E5F-A6B7-C8D9E0F1A2B3}
/// 此 GUID 必须与 Package.appxmanifest 中 com:Class Id 一致。
pub const CLSID_LITEPACKSHELL: GUID = GUID {
    data1: 0x7B2E_4A1F,
    data2: 0x9C3D,
    data3: 0x4E5F,
    data4: [0xA6, 0xB7, 0xC8, 0xD9, 0xE0, 0xF1, 0xA2, 0xB3],
};

// ─── 子命令 CanonicalName GUID ───────────────────────────────────────────────

pub const GUID_CMD_OPEN: GUID = GUID {
    data1: 0x7B2E_4A1F,
    data2: 0x9C3D,
    data3: 0x4E5F,
    data4: [0xA6, 0xB7, 0xC8, 0xD9, 0xE0, 0xF1, 0xA2, 0xB4],
};

pub const GUID_CMD_EXTRACT_HERE: GUID = GUID {
    data1: 0x7B2E_4A1F,
    data2: 0x9C3D,
    data3: 0x4E5F,
    data4: [0xA6, 0xB7, 0xC8, 0xD9, 0xE0, 0xF1, 0xA2, 0xB5],
};

pub const GUID_CMD_EXTRACT_TO: GUID = GUID {
    data1: 0x7B2E_4A1F,
    data2: 0x9C3D,
    data3: 0x4E5F,
    data4: [0xA6, 0xB7, 0xC8, 0xD9, 0xE0, 0xF1, 0xA2, 0xB6],
};

pub const GUID_CMD_EXTRACT_NAMED: GUID = GUID {
    data1: 0x7B2E_4A1F,
    data2: 0x9C3D,
    data3: 0x4E5F,
    data4: [0xA6, 0xB7, 0xC8, 0xD9, 0xE0, 0xF1, 0xA2, 0xB7],
};

// ─── EXPCMDSTATE / EXPCMDFLAGS ───────────────────────────────────────────────

pub const ECS_ENABLED: u32 = 0;
#[allow(dead_code)]
pub const ECS_DISABLED: u32 = 1;
#[allow(dead_code)]
pub const ECS_HIDDEN: u32 = 2;

pub const ECF_DEFAULT: u32 = 0;
pub const ECF_HASSUBCOMMANDS: u32 = 0x1;

/// SIGDN_FILESYSPATH — 获取文件系统绝对路径
pub const SIGDN_FILESYSPATH: u32 = 0x8005_8000;

// ─── 辅助函数 ────────────────────────────────────────────────────────────────

/// 将 Rust &str 转为 CoTaskMemAlloc 分配的 UTF-16 null-terminated 字符串。
/// 调用者（Explorer）负责 CoTaskMemFree。
pub fn alloc_wide(s: &str) -> *mut u16 {
    let wide: Vec<u16> = s.encode_utf16().collect();
    let byte_len = (wide.len() + 1) * 2;
    let ptr = unsafe { windows_sys::Win32::System::Com::CoTaskMemAlloc(byte_len) as *mut u16 };
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    unsafe {
        std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());
        *ptr.add(wide.len()) = 0;
    }
    ptr
}

/// 将 null-terminated UTF-16 指针转为 Rust String。
pub fn wide_to_string(ptr: *const u16) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let mut len = 0usize;
    unsafe {
        while *ptr.add(len) != 0 {
            len += 1;
        }
        let slice = std::slice::from_raw_parts(ptr, len);
        String::from_utf16_lossy(slice)
    }
}

/// 检查指针是否指向指定 IID（用于 QueryInterface）。
pub fn iid_matches(riid: *const GUID, target: &GUID) -> bool {
    if riid.is_null() {
        return false;
    }
    unsafe { (*riid).eq(target) }
}
