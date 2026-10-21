# Windows 驱动 `sys/windows.rs`

- 日期：2026-10-21
- 状态：已实施
- 提交：`4c1058f` feat(driver): support Windows consoles
- Brief：[win-sys](briefs/win-sys.md)

## 背景

[051](051-windows-plan-sys-split.md) 的第一步把 driver 的系统调用收进 `sys` 接口（`f52385c`）。第二步按同一接口写 Windows 实现。方案中的四点（1809、#969 实测、100ms 轮询、`WriteConsoleW`）由用户以「确认 windows的测试也得过」确认；unsafe 豁免来自用户的「windows这里 支持用ffi把 豁免」。

## 决策

以下实现规则由 Claude 在方案中提出，随方案一并确认，逐条写进 brief：

- **unsafe 设置**：`lib.rs` 在非 Windows 用 `forbid(unsafe_code)`，在 Windows 用 `deny(unsafe_code)`；只在 `sys/mod.rs` 的 `mod windows` 上 `allow`。每个块只包一个调用，带 `// SAFETY:`。
- **依赖**：rustix、signal-hook、filedescriptor 移到 `cfg(unix)`；新增 `windows-sys` 0.61，放在 `cfg(windows)`，只开用到的 feature。
- **TTY 判定**：`GetConsoleMode` 成功才算 TTY，同 libuv；stdout 还要求 VT processing 开得起来，legacy console 走非交互、无颜色路径。msys/mintty 管道不算 TTY，不用 std 的 `IsTerminal`。
- **raw mode**：只设 `ENABLE_VIRTUAL_TERMINAL_INPUT`，Ctrl+C 作为 0x03 进入输入，同 Unix 清掉 ISIG。
- **控制事件**：`SetConsoleCtrlHandler`；没有 live app 时返回 FALSE 走系统默认；否则交给 `process::signal` 后阻塞 handler，因为 CLOSE 事件下 handler 一返回系统就结束进程。退出码 `STATUS_CONTROL_C_EXIT`，用 `std::process::exit`，不需要 FFI。
- **Ctrl+Z**：Windows 没有作业控制，新增 `JOB_CONTROL` 常量，`mount` 里清掉 `suspend_on_ctrl_z`，Ctrl+Z 交给 `use_input`，同 Node。
- **颜色**：移植 chalk supports-color 的 Windows 分支，作为平台参数传给 `color_level`。1809 以上该分支都是真彩，不读系统版本号。

具体规则见 [design.md 平台](../design.md#平台)。

## 方案

一个 Codex worker（brief `win-sys`）在独立 worktree 里做，和 win-tests worker（[054](054-conpty-tests-ci.md)）并行，两边文件不重叠。brief 要点：

- 按 `sys/unix.rs` 的接口逐项给出 Windows 行为，每项写明 libuv 的对应函数作为依据。
- 记录解码与 FFI 调用分开，放进一个小函数，可以脱离 console 测试。
- 只加一个 Windows 单元测试：跨读取的代理对、丢弃 key-up、保留 Alt+numpad 组合的 key-up。理由：ConPTY 不能按需造出分段读取，而解码跨读取保存状态。
- 更新 `design.md` 平台一节和其他因此失真的段落（resize、非 TTY 颜色、Ctrl+Z），README 的 Platform support。
- 验收：Windows 目标的 clippy（两套 feature）与 `check --profile test`，macOS 的 lib 测试与 PTY 测试，fmt，clippy。本机跑不了 Windows 程序。

## 结果

- worker 交付 12 个 unsafe 块，各带 SAFETY 注释。自选的细节：停止事件排在等待数组首位，两者同时就绪时先停；输入缓冲 128 条记录；VT 开启失败不算挂载错误；Windows 保留空的 `stop_process`；Unix 的 Stop/Continue 消息和 `Reason::Stop` 加 `cfg(unix)`，避免 Windows 上的死代码。Cargo 生成的 `Cargo.lock` 改动被 worker 按文件范围撤回，集成时补入。
- Claude review 后改了两处：
  - raw mode 的注释和文档原写「同 libuv」，但 libuv 还加 `ENABLE_WINDOW_INPUT` 用尺寸记录检测 resize。改为写明不加这个标志，尺寸由轮询得到。
  - `lib.rs` 一行文档注释折行。
- 集成验证：macOS 上两套 feature 的全量测试通过，Windows 目标的 clippy 和 check 通过。提交 `4c1058f`。
- Windows 上的实际运行在 CI 上完成，解码单元测试在 Windows job 中运行通过，见 [054](054-conpty-tests-ci.md)。
