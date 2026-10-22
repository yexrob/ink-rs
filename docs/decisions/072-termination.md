# 终止信号交给应用：进程级的 `set_termination`

- 日期：2026-10-22
- 状态：实施中
- 提交：无
- Brief：[termination](briefs/termination.md)；需求：[bingo-signal-ownership](briefs/bingo-signal-ownership.md)

## 背景

bingo 在 tokio 进程里处理 SIGINT、SIGTERM、SIGHUP：收到 SIGTERM 或 SIGHUP 时结束当前轮次、运行 hook、写盘、卸载应用，然后退出。ink-rs 在第一次渲染时装上进程级的信号处理，收到终止信号后恢复终端，以默认动作结束进程，宿主的清理不会运行。现状见 [071](071-bingo-research.md) 的背景。

## 决策

- 用户定：选「进程级函数 (Recommended)」。宿主在 `main` 开头调用 `ink_rs::set_termination(Termination::Application)`，之后的挂载，包括库发起的挂载，按这个设置。理由见 071：信号处理属于进程、卸不掉；放在 `Options` 里时，库用默认设置的挂载在前，宿主的设置失效。
- Claude 按用户确认的方案（「进程级函数 (Recommended)」）定细节：
  - 设置在 ink-rs 装上进程处理时读取并固定；之后用相同的值调用返回 `Ok`，不同的值返回错误。
  - `Application` 下 Unix 不为 SIGINT、SIGTERM、SIGHUP 注册条件默认动作和 `Signals` 处理；SIGWINCH、SIGTSTP、SIGCONT 不变。Windows 把控制台的终止事件留给应用，worker 核对 `control_handler` 的现有职责。
  - panic 时的终端恢复、`exit_on_ctrl_c` 不变。
  - `testing::mount` 和 `testing::render` 不装处理，不受设置影响。
- bingo 会话确认在 `main` 开头调用它（对方会话的转述，不算这里的批准；方案由用户在本会话确认）。

## 备选与取舍

- **`Options` 字段，第一个挂载说了算，后来不同的 panic**：主 agent 第一轮的推荐。库的默认挂载在前时，宿主的挂载 panic，见 071。
- **`Options` 字段，按挂载切换**：信号处理卸不掉，`Signals` 线程装上后无法撤回条件默认动作。

## 方案

一个 Codex worker（`codex exec -s workspace-write`），工作目录是 scratchpad 里基于 main 的 worktree `wt/ink-termination`。brief 见 [termination](briefs/termination.md)：

- API：`Termination { Handled, Application }` 和 `set_termination`，错误类型由 worker 按 `Error` 能否加变体决定。
- 测试：`tests/pty.rs` 加一个 Unix 用例。子进程设 `Application`，注册自己的 SIGTERM 标志，挂载，向自己发 SIGTERM，看到标志后渲染一帧，核对之后调用的返回值，卸载，以 0 退出；父进程检查退出码、信号之后的帧和恢复终端的字节。设置和信号处理是进程级的，所以要子进程。
- 验收：fmt；clippy 两组 feature 和 `x86_64-pc-windows-msvc`；`--lib`；`--test pty`。

主 agent 集成：review，提交，全量测试，`docs/design.md` 的信号一节，tmux 里手动发 SIGTERM 核对，补结果，删 worktree。和 073、074、075 并行，进 0.1.7。
