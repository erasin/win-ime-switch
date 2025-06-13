# windows 输入法切换

`win-ime-switch` 为 helix/vim 提供 windows 下的输入法切换。

1. 需要在 windows 设置语言选项，添加英文键盘 和中文键盘，或者其他的输入法键盘。
2. `win-ime-switch --list` 检查支持的键盘布局，以及语言标识。
3. `win-ime-switch en` 切换输入法为英文。

|     参数 | Lang ID | 标题       |
| -------: | :-----: | ---------- |
|       en | 0x0409  | 英文       |
| zh,zh-cn | 0x0804  | 中文(简体) |
|    zh-tw | 0x0404  | 中文(繁体) |
|       ja | 0x0411  | 日语       |
|       ko | 0x0412  | 韩语       |
|       fr | 0x040C  | 法语       |
|       de | 0x0407  | 德语       |

4. 可以直接使用键盘布局标识(十六进制) 作为参数，比如 `win-ime-switch 0x0409` 

win-ime-switch

    en 英语
    zh,zh-cn 中文(简体)
    zh-tw 中文(繁体)
    ja jp 日语
    ko 韩语
    fr 法语
    de 德语
    0x0409 十六进制参数 自定义输入法
    --toggle  - 切换回上一个输入法 (保留内部状态)
    --current - 显示当前输入法
    --list    - 显示已启用的输入法
