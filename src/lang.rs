use std::fmt::{self, Display, Formatter};

use crate::errors::ImeError;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(u32)]
pub enum LangID {
    EN = 0x0409,   // 英文
    ZH = 0x0804,   // 中文(简体)
    Zhtw = 0x0404, // 中文(繁体)
    JA = 0x0411,   // 日语
    KO = 0x0412,   // 韩语
    FR = 0x040C,   // 法语
    DE = 0x0407,   // 德语
    Other(u32),    // 其他
}

impl From<u32> for LangID {
    fn from(value: u32) -> Self {
        match value {
            0x0409 => LangID::EN,
            0x0804 => LangID::ZH,
            0x0404 => LangID::Zhtw,
            0x0411 => LangID::JA,
            0x0412 => LangID::KO,
            0x040C => LangID::FR,
            0x0407 => LangID::DE,
            _ => LangID::Other(value),
        }
    }
}

impl TryFrom<&str> for LangID {
    type Error = ImeError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if let Some(num) = s.strip_prefix("0x") {
            if let Ok(num) = u32::from_str_radix(num, 16) {
                return Ok(num.into());
                // return Ok(LangID::Other(num & 0xFFFF));
            }
        }

        match s.to_lowercase().as_str() {
            "en" => Ok(LangID::EN),
            "zh" | "zh-cn" => Ok(LangID::ZH),
            "zh-tw" => Ok(LangID::Zhtw),
            "ja" | "jp" => Ok(LangID::JA),
            "ko" => Ok(LangID::KO),
            "fr" => Ok(LangID::FR),
            "de" => Ok(LangID::DE),
            _ => {
                if let Ok(num) = s.parse::<u32>() {
                    Ok(num.into())
                } else {
                    Err(ImeError::Unsupported(s.into()))
                }
            }
        }
    }
}

impl Display for LangID {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let name = match self {
            LangID::EN => "英语",
            LangID::ZH => "中文(简体)",
            LangID::Zhtw => "中文(繁体)",
            LangID::JA => "日语",
            LangID::KO => "韩语",
            LangID::FR => "法语",
            LangID::DE => "德语",
            LangID::Other(id) => &format!("自定义: {id}"),
        };
        write!(f, "{}", name)
    }
}
