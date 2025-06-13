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

// 输入法状态管理器
struct InputMethodManager {
    config_path: PathBuf,
}

impl InputMethodManager {
    fn new() -> Result<Self> {
        // 获取用户配置目录
        let mut config_path = match env::var("APPDATA") {
            Ok(path) => PathBuf::from(path),
            Err(_) => PathBuf::from("."),
        };
        config_path.push("input_switcher_state");

        Ok(Self { config_path })
    }

    // 保存当前输入法状态
    fn save_current_layout(&self, hkl: HKL) -> Result<()> {
        let data = format!("{:X}", hkl.0 as u64);
        fs::write(&self.config_path, data)
            .map_err(|e| Error::new(E_FAIL.into(), format!("保存状态失败: {}", e)))
    }

    // 加载保存的输入法状态
    fn load_saved_layout(&self) -> Result<HKL> {
        let data = fs::read_to_string(&self.config_path)
            .map_err(|e| Error::new(E_FAIL.into(), format!("加载状态失败: {}", e)))?;

        let hkl_value = u64::from_str_radix(&data, 16)
            .map_err(|e| Error::new(E_INVALIDARG.into(), format!("无效状态: {}", e)))?;

        Ok(HKL(hkl_value as *mut std::ffi::c_void))
    }

    // 获取当前输入法
    fn get_current_layout(&self) -> Result<HKL> {
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

fn switch_input_method(lang_id: LangID) -> Result<()> {
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
            return Err(Error::new(
                E_INVALIDARG.into(), // 转换为 i32 错误码
                "获取键盘布局失败",
            ));
        }

        #[cfg(debug_assertions)]
        {
            println!("系统安装的输入法列表 ({} 个):", layouts.len());
            for (index, hkl) in layouts.iter().enumerate() {
                let lang_id = (hkl.0 as u32) & 0xFFFF;
                println!(
                    "  [{}] HKL: 0x{:08X}, LangID: 0x{:04X}",
                    index + 1,
                    hkl.0 as u32,
                    lang_id
                );
            }
        }

        // 3. 寻找英文输入法 (0x0409)
        let english_layout = layouts
            .iter()
            .find(|hkl| {
                let current_lang_id = (hkl.0 as u32) & 0xFFFF;
                current_lang_id == lang_id as u32
            })
            .ok_or_else(|| Error::new(E_FAIL.into(), format!("未找到{}输入法", lang_id.name())))?;

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
fn toggle_layout(manager: &InputMethodManager) -> Result<()> {
    // 1. 获取当前输入法
    let current_hkl = manager.get_current_layout()?;

    // 2. 加载保存的输入法
    let saved_hkl = manager.load_saved_layout()?;

    // 3. 切换回保存的输入法
    switch_to_layout(saved_hkl)?;

    // 4. 保存当前输入法作为新的状态
    manager.save_current_layout(current_hkl)
}

#[derive(Debug, Clone, Copy)]
enum LangID {
    EN = 0x0409,
    ZH = 0x0804,
}

// 0x0404 => "中文 (繁体)",
// 0x0411 => "日语",
// 0x0412 => "韩语",
// 0x040C => "法语",
// 0x0407 => "德语",

impl LangID {
    fn name(self) -> &'static str {
        match self {
            LangID::EN => "英文",
            LangID::ZH => "中文 (简体)",
        }
    }
}

// impl From<u32> for LangID {
//     fn from(value: u32) -> Self {
//     }
// }

fn main() -> Result<()> {
    // 初始化状态管理器
    let manager = InputMethodManager::new()?;
    // 1. 获取当前输入法
    let current_hkl = manager.get_current_layout()?;

    // 解析命令行参数
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("en") => {
            manager.save_current_layout(current_hkl)?;
            switch_input_method(LangID::EN)?
        }
        Some("zh") => {
            manager.save_current_layout(current_hkl)?;
            switch_input_method(LangID::ZH)?
        }
        Some("--toggle") => {
            toggle_layout(&manager)?;
            // println!("已切换回上一个输入法");
        }
        Some("--current") => {
            let lang_id = (current_hkl.0 as u32) & 0xFFFF;
            let lang_name = match lang_id {
                0x0409 => "en",
                0x0804 => "zh",
                _ => "其他",
            };
            println!("{}", lang_name)
            // println!("当前输入法: {} (0x{:04X})", lang_name, lang_id);
        }

        _ => {
            // 显示用法说明
            println!("输入法切换工具");
            println!(
                "用法: {} [命令]",
                args.get(0).unwrap_or(&"program".to_string())
            );
            println!("  en        - 切换到英文输入法");
            println!("  zh        - 切换到中文输入法");
            println!("  --toggle  - 切换回上一个输入法");
            println!("  --current - 显示当前输入法");
            return Ok(());
        }
    };

    Ok(())
}
