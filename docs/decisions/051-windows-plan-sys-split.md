# Windows 支持方案与 driver 的 sys 拆分

- 日期：2026-10-21
- 状态：已实施
- 提交：`f52385c` refactor(driver): move Unix system calls behind a sys module
- Brief：[sys-split](briefs/sys-split.md)

## 背景

v0.1.0 发布后，用户问：「你看看上一个session， 我们已经发布了。 然后貌似windows 支持没做 是吗」。Claude 核对后答复：Windows 不在 v1 范围内；`driver` 模块在 `#[cfg(unix)]` 后面，Windows 上能编译，但没有 `render`、`Instance` 和 `ink_rs::tokio`。Unix 专属代码集中在 `src/driver/process.rs`（signal-hook、rustix）和 `src/driver/stdin.rs`（termios、`socketpair` 加 `poll`）。

Console API 要走 FFI，项目规则禁止 unsafe。Claude 给出两条路请用户定。用户答：「windows这里 支持用ffi把 豁免」。

## 决策

- 用 windows-sys 自己调 Console API，unsafe 只放在 Windows FFI 文件里。用户决定（「windows这里 支持用ffi把 豁免」）。规则写入见 [052](052-unsafe-exemption-rule.md)。
- 下面四点由 Claude 提出，用户回复「确认 windows的测试也得过」：
  1. 最低版本 Windows 10 1809（build 17763）。
  2. Ink #969（全屏帧之间整屏清除）不照搬，先在 Windows CI 里经 ConPTY 实测，有残留再设计修法。
  3. resize 用 100ms 轮询，不用 `SetWinEventHook`。
  4. 输出用 `WriteConsoleW`，不改代码页。
- 门槛来自用户的「windows的测试也得过」：GitHub Actions 上 Linux、macOS、Windows 三个平台的 fmt、clippy、全量 test 都通过。
- 分三步：先拆出 `sys/unix.rs`，再并行写 Windows 驱动（[053](053-windows-driver.md)）和测试、CI（[054](054-conpty-tests-ci.md)）。Codex 实现，Claude 写 brief、review、集成时跑全量测试。

## 备选与取舍

- **crossterm 一类封装 crate**：不用写 unsafe。代价是多一个大依赖，它把输入转成自己的事件结构，和 ink-rs 按字节解析 VT 的做法对不上。用户选了 FFI 豁免。
- **照搬 Ink 的 #969 修法**：Ink PR #971 的作者写明没在真 Windows 上验证；microsoft/terminal #3826 提到 VT 模式下 conhost 带 `WC_DELAY_EOL_WRAP`，写右下角不会滚屏。ink-rs 的输出层按 pending wrap 建模，若 conhost 真会滚屏，受影响的不止满屏帧，修法也和 Ink 不同。所以先实测。
- **`SetWinEventHook`**（libuv 的做法）：要 user32、一个消息循环线程，还要调 ntdll 未公开接口取 conhost 的 pid。轮询的代价是 resize 最多晚 100ms。
- **`SetConsoleOutputCP(65001)`**：写法更简单，但改的是进程全局状态，宿主程序的其他输出也受影响。

## 方案

调研由主 agent 自己做：读 driver 代码、上个 session 留下的 Ink 源码、libuv 的 Windows tty 源码（`win-research/libuv-tty.c`）和微软文档。结论：

- Node 的 `setRawMode(true)` 用 `UV_TTY_MODE_RAW_VT`，打开 `ENABLE_VIRTUAL_TERMINAL_INPUT`，按键由 console 转成 VT 序列，现有字节解析器可以继续用。
- 输出打开 `ENABLE_VIRTUAL_TERMINAL_PROCESSING` 后，conhost 把 LF 转成 CRLF，效果同 ONLCR，换行处理不用改。
- Ink 本身的 Windows 分支只有 #969 一处。

架构：driver 拆成共用部分（`process.rs`、`stdin.rs`）和平台部分（`sys/unix.rs`、`sys/windows.rs`），平台接口覆盖输出句柄、尺寸、raw mode、读线程启停、事件源和终止进程。Windows 实现要点后来写进 [design.md 平台](../design.md#平台)。

第一步派一个 Codex worker（brief `sys-split`），在独立 worktree 里做纯重构：

- 把 rustix、signal-hook、filedescriptor、`std::os::fd` 的用法全部移到 `src/driver/sys/unix.rs`。
- 列出 `sys` 要提供的 8 项职责，名字和签名由 worker 定，Windows 模块之后提供同一组。
- 读线程和事件线程在 `sys` 里只取 OS 的字节和事件，路由和分发移到 `stdin.rs`、`process.rs` 的共用函数。
- 锁顺序等共用不变式的注释留在共用代码，OS 专属注释跟着 OS 调用走。
- 不加测试；`stdin.rs` 的两个读线程单元测试随代码移动，断言不变。
- 验收：`cargo test --offline --lib`、四个 PTY 类测试目标、`--no-default-features` 的 lib 测试、`cargo check --target x86_64-pc-windows-msvc --lib`、fmt、两套 feature 的 clippy。

## 结果

- worker 定义的接口：`Output`、`SavedMode`、`Termination` 三个类型别名，`stdout`/`stderr`、两个 TTY 检查、`size`、`enter_raw_mode`/`restore_mode`、`Reader::start`/`stop`、`install_events`、`terminate`、`stop_process`。共用侧新增 `process::{resize, stop, resume, signal}` 与 `stdin::{input, ended}`。偏差只有 `src/driver/mod.rs` 多一行 `mod sys;`。
- Claude review 结论：「接口清晰，共用逻辑和注释都在原位」。集成时在 macOS 跑全量测试和 clippy，Windows 目标的 check 通过，有 4 条驱动外的未使用警告，驱动在 Windows 上编译后消失。
- 提交 `f52385c`。worktree 删除。
- 真机检查：Claude 问用户手上有没有 Windows 机器，记录中未见答复，验证以 CI 为准（见 [054](054-conpty-tests-ci.md)）。
