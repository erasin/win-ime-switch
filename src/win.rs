use std::{env, fs, path::PathBuf};

use windows::{
    Win32::{
        Foundation::{E_FAIL, E_INVALIDARG, LPARAM, WPARAM},
        UI::{
            Input::KeyboardAndMouse::{GetKeyboardLayout, GetKeyboardLayoutList, HKL},
            WindowsAndMessaging::{
                GetForegroundWindow, GetWindowThreadProcessId, PostMessageW,
                WM_INPUTLANGCHANGEREQUEST,
            },
        },
    },
    core::{Error, Result},
};

use crate::lang::LangID;

// 输入法状态管理器
pub struct InputMethodManager {
    config_path: PathBuf,
}

impl InputMethodManager {
    pub fn new() -> Result<Self> {
        // 获取用户配置目录
        let mut config_path = match env::var("APPDATA") {
            Ok(path) => PathBuf::from(path),
            Err(_) => PathBuf::from("."),
        };
        config_path.push("input_switcher_state");

        Ok(Self { config_path })
    }

    // 保存当前输入法状态
    pub fn save_current_layout(&self, hkl: HKL) -> Result<()> {
        let data = format!("{:X}", hkl.0 as u64);
        fs::write(&self.config_path, data)
            .map_err(|e| Error::new(E_FAIL, format!("保存状态失败: {}", e)))
    }

    // 加载保存的输入法状态
    fn load_saved_layout(&self) -> Result<HKL> {
        let data = fs::read_to_string(&self.config_path)
            .map_err(|e| Error::new(E_FAIL, format!("加载状态失败: {}", e)))?;

        let hkl_value = u64::from_str_radix(&data, 16)
            .map_err(|e| Error::new(E_INVALIDARG, format!("无效状态: {}", e)))?;

        Ok(HKL(hkl_value as *mut std::ffi::c_void))
    }

    // 获取当前输入法
    pub fn get_current_layout(&self) -> Result<HKL> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.is_invalid() {
                return Err(Error::from_win32());
            }

            let thread_id = GetWindowThreadProcessId(hwnd, None);
            let hkl = GetKeyboardLayout(thread_id);

            if hkl.is_invalid() {
                Err(Error::from_win32())
            } else {
                Ok(hkl)
            }
        }
    }
}

// 切换输入法核心函数
fn switch_to_layout(hkl: HKL) -> Result<()> {
    unsafe {
        // 获取前景窗口
        let hwnd = GetForegroundWindow();
        if hwnd.is_invalid() {
            return Err(Error::from_win32());
        }

        // 发送切换请求
        PostMessageW(
            Some(hwnd),
            WM_INPUTLANGCHANGEREQUEST,
            WPARAM(0),
            LPARAM(hkl.0 as isize),
        )?;

        Ok(())
    }
}

pub fn switch_input_method(lang_id: LangID) -> Result<()> {
    unsafe {
        // 1. 获取键盘布局数量
        let layout_count = GetKeyboardLayoutList(None);
        if layout_count == 0 {
            return Err(Error::from_win32());
        }

        // 2. 获取所有键盘布局
        let mut layouts = vec![Default::default(); layout_count as usize];
        let actual_count = GetKeyboardLayoutList(Some(&mut layouts));
        if actual_count != layout_count {
            return Err(Error::new(E_INVALIDARG, "获取键盘布局失败"));
        }

        // 3. 寻找英文输入法 (0x0409)
        let english_layout = layouts
            .iter()
            .find(|hkl| {
                let current_lang_id = (hkl.0 as u32) & 0xFFFF;
                lang_id == current_lang_id.into()
            })
            .ok_or_else(|| Error::new(E_FAIL, format!("未找到{lang_id}输入法")))?;

        // 4. 获取前景窗口
        let hwnd = GetForegroundWindow();
        if hwnd.is_invalid() {
            return Err(Error::from_win32());
        }

        // 5. 发送切换请求
        PostMessageW(
            Some(hwnd), // 包装为 Option<HWND>
            WM_INPUTLANGCHANGEREQUEST,
            WPARAM(0),
            LPARAM(english_layout.0 as isize),
        )?; // 直接使用 ? 操作符处理错误

        Ok(())
    }
}

// 切换回上次保存的输入法
pub fn toggle_layout(manager: &InputMethodManager) -> Result<()> {
    // 1. 获取当前输入法
    let current_hkl = manager.get_current_layout()?;

    // 2. 加载保存的输入法
    let saved_hkl = manager.load_saved_layout()?;

    // 3. 切换回保存的输入法
    switch_to_layout(saved_hkl)?;

    // 4. 保存当前输入法作为新的状态
    manager.save_current_layout(current_hkl)
}

pub fn print_langs() -> Result<()> {
    unsafe {
        // 1. 获取键盘布局数量
        let layout_count = GetKeyboardLayoutList(None);
        if layout_count == 0 {
            return Err(Error::from_win32());
        }

        // 2. 获取所有键盘布局
        let mut layouts = vec![Default::default(); layout_count as usize];
        let actual_count = GetKeyboardLayoutList(Some(&mut layouts));
        if actual_count != layout_count {
            return Err(Error::new(E_INVALIDARG, "获取键盘布局失败"));
        }

        println!("系统安装的输入法列表 ({} 个):", layouts.len());
        for (index, hkl) in layouts.iter().enumerate() {
            let lang_id = (hkl.0 as u32) & 0xFFFF;
            let lang: LangID = lang_id.into();
            println!("  [{}] 0x{:04X} {lang}", index + 1, lang_id);
        }

        Ok(())
    }
}
