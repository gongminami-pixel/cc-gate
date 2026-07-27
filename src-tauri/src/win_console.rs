//! Windows 控制台闪窗根治 —— 给 spawn 的每个子进程加 `CREATE_NO_WINDOW`。
//!
//! 这是本仓库作为「标准参考项目」沉淀的通用模块，**可整文件拷进任何
//! Tauri / Rust 桌面项目**复用。完整原理、自查清单与本仓库落地清单见
//! `readme.md`「跨项目标准：Windows 子进程无黑窗」节。
//!
//! ## 为什么需要
//! Tauri 是 GUI 程序、自身没有控制台（`windows_subsystem = "windows"`）。
//! 它用 `Command` spawn 一个**控制台子进程**（frpc / ffmpeg / cmd / where …）
//! 时，Windows 会为该子进程**新分配一个控制台窗口** → 屏幕上黑窗一闪即逝。
//! macOS / Linux 的进程模型不涉及「分配控制台」，天然无此问题。
//!
//! ## 解法
//! `CreateProcess` 的 `CREATE_NO_WINDOW = 0x0800_0000` creation flag：子进程
//! 不分配控制台窗口（但仍是控制台程序，stdout / stderr 照常可重定向捕获）。
//!
//! ## 用法
//! 在 `.spawn()` / `.output()` / `.status()` **之前**调用：
//! - [`hide_console`]        —— 吃同步的 `std::process::Command`
//! - [`hide_console_async`]  —— 吃 `tokio::process::Command`
//!
//! 两者在**非 Windows** 上都是空函数（no-op），调用点无脑加即可、不必自己写
//! `#[cfg]`。别混用（两种 Command 类型不同）。
//!
//! ## 什么时候「不用」加
//! - 只在 macOS/Linux 跑的命令（本项目的 `/bin/ps`、`launchctl`）—— 不在
//!   Windows 跑，也不是闪窗源，无需处理。
//! - `explorer.exe`（GUI 程序，不弹控制台）无需加。
//! - 开 URL / 打开文件夹别图省事用 `cmd /C start`（cmd 本身就是控制台、必闪），
//!   优先 `tauri-plugin-opener` / `open` crate；本项目当前不开 URL，故无此调用点。

/// Windows `CREATE_NO_WINDOW`：spawn 命令行工具时不弹黑色控制台窗。
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// 给【同步】子进程隐藏控制台窗口（仅 Windows 生效；其它平台 no-op）。
///
/// 必须在 `.spawn()` / `.output()` / `.status()` 之前调用。
#[cfg_attr(not(windows), allow(unused_variables))]
pub fn hide_console(cmd: &mut std::process::Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
}

/// 给【tokio 异步】子进程隐藏控制台窗口（仅 Windows 生效；其它平台 no-op）。
///
/// 必须在 `.spawn()` / `.output()` / `.status()` 之前调用。
#[cfg_attr(not(windows), allow(unused_variables))]
pub fn hide_console_async(cmd: &mut tokio::process::Command) {
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
}
