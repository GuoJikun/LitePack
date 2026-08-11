//! IExplorerCommand 实现（父命令 + 子命令）及 IEnumExplorerCommand 枚举器。

use std::ffi::c_void;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::guid::*;
use crate::launch;

// ─── 全局引用计数（DllCanUnloadNow 用）────────────────────────────────────────

static G_TOTAL_REFCOUNT: AtomicU32 = AtomicU32::new(0);

pub fn global_refcount() -> u32 {
    G_TOTAL_REFCOUNT.load(Ordering::SeqCst)
}

fn add_global_ref() {
    G_TOTAL_REFCOUNT.fetch_add(1, Ordering::SeqCst);
}

fn sub_global_ref() {
    G_TOTAL_REFCOUNT.fetch_sub(1, Ordering::SeqCst);
}

// ─── 子命令动作 ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub enum Action {
    Open,
    ExtractHere,
    ExtractTo,
    ExtractNamed,
}

impl Action {
    pub fn title(&self) -> &'static str {
        match self {
            Action::Open => "用 LitePack 打开",
            Action::ExtractHere => "解压到当前文件夹",
            Action::ExtractTo => "解压到...",
            Action::ExtractNamed => "解压到同名目录",
        }
    }

    pub fn cli_flag(&self) -> &'static str {
        match self {
            Action::Open => "--open",
            Action::ExtractHere => "--extract-here",
            Action::ExtractTo => "--extract-to",
            Action::ExtractNamed => "--extract-named",
        }
    }

    pub fn canonical_guid(&self) -> &'static GUID {
        match self {
            Action::Open => &GUID_CMD_OPEN,
            Action::ExtractHere => &GUID_CMD_EXTRACT_HERE,
            Action::ExtractTo => &GUID_CMD_EXTRACT_TO,
            Action::ExtractNamed => &GUID_CMD_EXTRACT_NAMED,
        }
    }
}

const ALL_ACTIONS: [Action; 4] = [
    Action::Open,
    Action::ExtractHere,
    Action::ExtractTo,
    Action::ExtractNamed,
];

// ─── IExplorerCommand vtable ─────────────────────────────────────────────────

#[repr(C)]
pub struct IExplorerCommandVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut c_void) -> u32,
    pub Release: unsafe extern "system" fn(*mut c_void) -> u32,
    pub GetTitle: unsafe extern "system" fn(*mut c_void, *mut c_void, *mut *mut u16) -> HRESULT,
    pub GetIcon: unsafe extern "system" fn(*mut c_void, *mut c_void, *mut *mut u16) -> HRESULT,
    pub GetToolTip: unsafe extern "system" fn(*mut c_void, *mut c_void, *mut *mut u16) -> HRESULT,
    pub GetCanonicalName: unsafe extern "system" fn(*mut c_void, *mut GUID) -> HRESULT,
    pub GetState: unsafe extern "system" fn(*mut c_void, *mut c_void, i32, *mut u32) -> HRESULT,
    pub Invoke: unsafe extern "system" fn(*mut c_void, *mut c_void, *mut c_void) -> HRESULT,
    pub GetFlags: unsafe extern "system" fn(*mut c_void, *mut u32) -> HRESULT,
    pub EnumSubCommands: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HRESULT,
}

// ─── ExplorerCommand 对象 ─────────────────────────────────────────────────────

#[repr(C)]
pub struct ExplorerCommand {
    pub lpVtbl: *const IExplorerCommandVtbl,
    pub ref_count: AtomicU32,
    pub kind: CommandKind,
}

pub enum CommandKind {
    /// 顶层 "LitePack" 菜单项，拥有子命令
    Parent,
    /// 叶子命令，Invoke 时启动 LitePack.exe
    Child(Action),
}

// ─── 静态 vtable ──────────────────────────────────────────────────────────────

static EXPLORER_COMMAND_VTBL: IExplorerCommandVtbl = IExplorerCommandVtbl {
    QueryInterface: explorer_command_query_interface,
    AddRef: explorer_command_add_ref,
    Release: explorer_command_release,
    GetTitle: explorer_command_get_title,
    GetIcon: explorer_command_get_icon,
    GetToolTip: explorer_command_get_tooltip,
    GetCanonicalName: explorer_command_get_canonical_name,
    GetState: explorer_command_get_state,
    Invoke: explorer_command_invoke,
    GetFlags: explorer_command_get_flags,
    EnumSubCommands: explorer_command_enum_sub_commands,
};

// ─── 构造函数 ─────────────────────────────────────────────────────────────────

pub fn create_parent_command() -> *mut c_void {
    let cmd = Box::new(ExplorerCommand {
        lpVtbl: &EXPLORER_COMMAND_VTBL,
        ref_count: AtomicU32::new(1),
        kind: CommandKind::Parent,
    });
    add_global_ref();
    Box::into_raw(cmd) as *mut c_void
}

pub fn create_child_command(action: Action) -> *mut c_void {
    let cmd = Box::new(ExplorerCommand {
        lpVtbl: &EXPLORER_COMMAND_VTBL,
        ref_count: AtomicU32::new(1),
        kind: CommandKind::Child(action),
    });
    add_global_ref();
    Box::into_raw(cmd) as *mut c_void
}

// ─── IUnknown 实现 ────────────────────────────────────────────────────────────

unsafe extern "system" fn explorer_command_query_interface(
    this: *mut c_void,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }

    if iid_matches(riid, &IID_IUNKNOWN) || iid_matches(riid, &IID_IEXPLORERCOMMAND) {
        *ppv = this;
        explorer_command_add_ref(this);
        S_OK
    } else {
        *ppv = std::ptr::null_mut();
        E_NOINTERFACE
    }
}

unsafe extern "system" fn explorer_command_add_ref(this: *mut c_void) -> u32 {
    let cmd = &*(this as *const ExplorerCommand);
    add_global_ref();
    cmd.ref_count.fetch_add(1, Ordering::SeqCst) + 1
}

unsafe extern "system" fn explorer_command_release(this: *mut c_void) -> u32 {
    let cmd = &*(this as *const ExplorerCommand);
    let new_count = cmd.ref_count.fetch_sub(1, Ordering::SeqCst) - 1;
    sub_global_ref();
    if new_count == 0 {
        drop(Box::from_raw(this as *mut ExplorerCommand));
    }
    new_count
}

// ─── IExplorerCommand 方法实现 ────────────────────────────────────────────────

unsafe extern "system" fn explorer_command_get_title(
    this: *mut c_void,
    _psi_item_array: *mut c_void,
    ppsz_name: *mut *mut u16,
) -> HRESULT {
    if ppsz_name.is_null() {
        return E_POINTER;
    }

    let cmd = &*(this as *const ExplorerCommand);
    let title = match &cmd.kind {
        CommandKind::Parent => "LitePack",
        CommandKind::Child(action) => action.title(),
    };

    let ptr = alloc_wide(title);
    if ptr.is_null() {
        return E_OUTOFMEMORY;
    }
    *ppsz_name = ptr;
    S_OK
}

unsafe extern "system" fn explorer_command_get_icon(
    _this: *mut c_void,
    _psi_item_array: *mut c_void,
    ppsz_icon: *mut *mut u16,
) -> HRESULT {
    if ppsz_icon.is_null() {
        return E_POINTER;
    }

    match launch::get_icon_path() {
        Some(icon) => {
            let ptr = alloc_wide(&icon);
            if ptr.is_null() {
                return E_OUTOFMEMORY;
            }
            *ppsz_icon = ptr;
            S_OK
        }
        None => {
            *ppsz_icon = std::ptr::null_mut();
            S_FALSE // 无图标但不报错
        }
    }
}

unsafe extern "system" fn explorer_command_get_tooltip(
    this: *mut c_void,
    _psi_item_array: *mut c_void,
    ppsz_infotip: *mut *mut u16,
) -> HRESULT {
    if ppsz_infotip.is_null() {
        return E_POINTER;
    }

    let cmd = &*(this as *const ExplorerCommand);
    let tooltip = match &cmd.kind {
        CommandKind::Parent => "LitePack 压缩/解压工具",
        CommandKind::Child(action) => action.title(),
    };

    let ptr = alloc_wide(tooltip);
    if ptr.is_null() {
        return E_OUTOFMEMORY;
    }
    *ppsz_infotip = ptr;
    S_OK
}

unsafe extern "system" fn explorer_command_get_canonical_name(
    this: *mut c_void,
    pguid: *mut GUID,
) -> HRESULT {
    if pguid.is_null() {
        return E_POINTER;
    }

    let cmd = &*(this as *const ExplorerCommand);
    let guid = match &cmd.kind {
        CommandKind::Parent => &CLSID_LITEPACKSHELL,
        CommandKind::Child(action) => action.canonical_guid(),
    };

    *pguid = *guid;
    S_OK
}

unsafe extern "system" fn explorer_command_get_state(
    _this: *mut c_void,
    _psi_item_array: *mut c_void,
    _f_ok_to_be_slow: i32,
    p_cmd_state: *mut u32,
) -> HRESULT {
    if p_cmd_state.is_null() {
        return E_POINTER;
    }
    *p_cmd_state = ECS_ENABLED;
    S_OK
}

unsafe extern "system" fn explorer_command_invoke(
    this: *mut c_void,
    psi_item_array: *mut c_void,
    _pbc: *mut c_void,
) -> HRESULT {
    let cmd = &*(this as *const ExplorerCommand);

    let action = match &cmd.kind {
        CommandKind::Parent => return E_NOTIMPL, // 父命令不直接执行
        CommandKind::Child(a) => *a,
    };

    // 从 IShellItemArray 获取文件路径
    let file_path = launch::get_first_file_path(psi_item_array);
    match file_path {
        Some(path) => {
            if launch::launch(action.cli_flag(), &path) {
                S_OK
            } else {
                E_FAIL
            }
        }
        None => E_FAIL,
    }
}

unsafe extern "system" fn explorer_command_get_flags(
    this: *mut c_void,
    p_flags: *mut u32,
) -> HRESULT {
    if p_flags.is_null() {
        return E_POINTER;
    }

    let cmd = &*(this as *const ExplorerCommand);
    *p_flags = match &cmd.kind {
        CommandKind::Parent => ECF_HASSUBCOMMANDS,
        CommandKind::Child(_) => ECF_DEFAULT,
    };
    S_OK
}

unsafe extern "system" fn explorer_command_enum_sub_commands(
    this: *mut c_void,
    pp_enum: *mut *mut c_void,
) -> HRESULT {
    if pp_enum.is_null() {
        return E_POINTER;
    }

    let cmd = &*(this as *const ExplorerCommand);
    match &cmd.kind {
        CommandKind::Parent => {
            let enumerator = create_enum_commands();
            if enumerator.is_null() {
                return E_OUTOFMEMORY;
            }
            *pp_enum = enumerator;
            S_OK
        }
        CommandKind::Child(_) => {
            *pp_enum = std::ptr::null_mut();
            E_NOTIMPL
        }
    }
}

// ─── IEnumExplorerCommand ─────────────────────────────────────────────────────

#[repr(C)]
pub struct IEnumExplorerCommandVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut c_void) -> u32,
    pub Release: unsafe extern "system" fn(*mut c_void) -> u32,
    pub Next: unsafe extern "system" fn(*mut c_void, u32, *mut *mut c_void, *mut u32) -> HRESULT,
    pub Skip: unsafe extern "system" fn(*mut c_void, u32) -> HRESULT,
    pub Reset: unsafe extern "system" fn(*mut c_void) -> HRESULT,
    pub Clone: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HRESULT,
}

#[repr(C)]
pub struct EnumCommands {
    pub lpVtbl: *const IEnumExplorerCommandVtbl,
    pub ref_count: AtomicU32,
    pub commands: [*mut c_void; 4],
    pub index: AtomicU32,
}

static ENUM_COMMANDS_VTBL: IEnumExplorerCommandVtbl = IEnumExplorerCommandVtbl {
    QueryInterface: enum_commands_query_interface,
    AddRef: enum_commands_add_ref,
    Release: enum_commands_release,
    Next: enum_commands_next,
    Skip: enum_commands_skip,
    Reset: enum_commands_reset,
    Clone: enum_commands_clone,
};

pub fn create_enum_commands() -> *mut c_void {
    let commands = [
        create_child_command(ALL_ACTIONS[0]),
        create_child_command(ALL_ACTIONS[1]),
        create_child_command(ALL_ACTIONS[2]),
        create_child_command(ALL_ACTIONS[3]),
    ];

    let enumerator = Box::new(EnumCommands {
        lpVtbl: &ENUM_COMMANDS_VTBL,
        ref_count: AtomicU32::new(1),
        commands,
        index: AtomicU32::new(0),
    });
    add_global_ref();
    Box::into_raw(enumerator) as *mut c_void
}

unsafe extern "system" fn enum_commands_query_interface(
    this: *mut c_void,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }

    if iid_matches(riid, &IID_IUNKNOWN) || iid_matches(riid, &IID_IENUMEXPLORERCOMMAND) {
        *ppv = this;
        enum_commands_add_ref(this);
        S_OK
    } else {
        *ppv = std::ptr::null_mut();
        E_NOINTERFACE
    }
}

unsafe extern "system" fn enum_commands_add_ref(this: *mut c_void) -> u32 {
    let e = &*(this as *const EnumCommands);
    add_global_ref();
    e.ref_count.fetch_add(1, Ordering::SeqCst) + 1
}

unsafe extern "system" fn enum_commands_release(this: *mut c_void) -> u32 {
    let e = &*(this as *const EnumCommands);
    let new_count = e.ref_count.fetch_sub(1, Ordering::SeqCst) - 1;
    sub_global_ref();
    if new_count == 0 {
        // 释放所有子命令的引用
        let enumerator = Box::from_raw(this as *mut EnumCommands);
        for &cmd in &enumerator.commands {
            if !cmd.is_null() {
                explorer_command_release(cmd);
            }
        }
    }
    new_count
}

unsafe extern "system" fn enum_commands_next(
    this: *mut c_void,
    celt: u32,
    p_commands: *mut *mut c_void,
    p_fetched: *mut u32,
) -> HRESULT {
    if p_commands.is_null() {
        return E_POINTER;
    }

    let e = &mut *(this as *mut EnumCommands);
    let mut fetched = 0u32;

    for i in 0..celt as usize {
        let idx = e.index.load(Ordering::SeqCst) as usize;
        if idx >= e.commands.len() {
            break;
        }
        let cmd = e.commands[idx];
        // AddRef for the caller
        explorer_command_add_ref(cmd);
        *p_commands.add(i) = cmd;
        e.index.fetch_add(1, Ordering::SeqCst);
        fetched += 1;
    }

    if !p_fetched.is_null() {
        *p_fetched = fetched;
    }

    if fetched == celt {
        S_OK
    } else {
        S_FALSE
    }
}

unsafe extern "system" fn enum_commands_skip(this: *mut c_void, celt: u32) -> HRESULT {
    let e = &mut *(this as *mut EnumCommands);
    let current = e.index.load(Ordering::SeqCst);
    let new_idx = current + celt;
    if new_idx > e.commands.len() as u32 {
        e.index.store(e.commands.len() as u32, Ordering::SeqCst);
        S_FALSE
    } else {
        e.index.store(new_idx, Ordering::SeqCst);
        S_OK
    }
}

unsafe extern "system" fn enum_commands_reset(this: *mut c_void) -> HRESULT {
    let e = &mut *(this as *mut EnumCommands);
    e.index.store(0, Ordering::SeqCst);
    S_OK
}

unsafe extern "system" fn enum_commands_clone(
    this: *mut c_void,
    pp_enum: *mut *mut c_void,
) -> HRESULT {
    if pp_enum.is_null() {
        return E_POINTER;
    }

    let e = &*(this as *const EnumCommands);

    // AddRef all commands for the clone
    for &cmd in &e.commands {
        explorer_command_add_ref(cmd);
    }

    let clone = Box::new(EnumCommands {
        lpVtbl: &ENUM_COMMANDS_VTBL,
        ref_count: AtomicU32::new(1),
        commands: e.commands,
        index: AtomicU32::new(e.index.load(Ordering::SeqCst)),
    });
    add_global_ref();
    *pp_enum = Box::into_raw(clone) as *mut c_void;
    S_OK
}
