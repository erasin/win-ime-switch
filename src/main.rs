use anyhow::Result;
use std::env;

mod errors;
mod lang;
mod win;

use lang::LangID;
use win::{InputMethodManager, print_langs, switch_input_method, toggle_layout};
fn main() -> Result<()> {
    // 初始化状态管理器
    let manager = InputMethodManager::new()?;
    // 1. 获取当前输入法
    let current_hkl = manager.get_current_layout()?;

    // 解析命令行参数
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("--toggle") => {
            toggle_layout(&manager)?;
            // println!("已切换回上一个输入法");
        }

        Some("--current") => {
            let lang_id = (current_hkl.0 as u32) & 0xFFFF;
            let lang: LangID = lang_id.into();
            println!("{lang}")
        }

        Some("--list") => {
            if let Err(e) = print_langs() {
                println!("{e}");
            }
        }

        Some(lang_tag) => {
            manager.save_current_layout(current_hkl)?;
            switch_input_method(lang_tag.try_into()?)?
        }

        _ => {
            // 显示用法说明
            println!("输入法切换工具");
            println!(
                "用法: {} [语言]",
                args.first().unwrap_or(&"program".to_string())
            );
            println!("  en 英语");
            println!("  zh,zh-cn 中文(简体)");
            println!("  zh-tw 中文(繁体)");
            println!("  ja jp 日语");
            println!("  ko 韩语");
            println!("  fr 法语");
            println!("  de 德语");
            println!("  0x0409 十六进制参数 自定义输入法");
            println!("  --toggle  - 切换回上一个输入法");
            println!("  --current - 显示当前输入法");
            println!("  --list    - 显示已启用的输入法");
            return Ok(());
        }
    };

    Ok(())
}
