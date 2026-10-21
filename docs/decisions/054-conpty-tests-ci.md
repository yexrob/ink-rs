# ConPTY 下的测试、三平台 CI 与 #969 实测

- 日期：2026-10-21
- 状态：已实施
- 提交：`f6e1339` test: run the PTY suites under ConPTY and add the #969 fixture；`613126d` ci: check Linux, macOS and Windows；`55f4f55` docs: record the #969 port and Windows PTY assertions
- Brief：[win-tests](briefs/win-tests.md)

## 背景

用户要求「确认 windows的测试也得过」。PTY 测试只在 Unix 上跑过，项目没有 CI。[051](051-windows-plan-sys-split.md) 定下：门槛是 GitHub Actions 上三个平台的 fmt、clippy、全量 test；Ink #969 不照搬，用一个 ConPTY fixture 实测。Windows 测试只能在 GitHub Actions 的 `windows-latest` 上跑，推送要用户批准。

## 决策

- 推 `windows-support` 分支到 origin 跑 CI，合进 main 前再问一次。用户决定（「可以」）。
- ConPTY 输出是 conhost 重绘后的字节流，Windows 上的断言改为比对 vt100 解析出的屏幕和回滚，Unix 上测 ink-rs 自身字节的断言不变。Claude 在方案中提出，随方案确认。
- 作业控制和会话首进程组两个用例留在 Unix。
- #969 的结论由 CI 结果决定（见结果）。

## 方案

一个 Codex worker（brief `win-tests`），与 win-sys worker（[053](053-windows-driver.md)）并行，只改 `tests/`、`examples/`、`.github/`、`.gitattributes`，看不到驱动代码，按 brief 描述的驱动行为写。brief 要点：

- **ConPTY 差异**，Claude 写 brief 前在 portable-pty 0.9.0 源码里查到：它带 `PSEUDOCONSOLE_INHERIT_CURSOR` 创建 ConPTY，conhost 启动时发 `ESC[6n` 并等回应，`Session` 要回 `ESC[1;1R`（含跨读取分段的请求）；子进程退出后关掉 pseudoconsole 才有 EOF；错误 109（`ERROR_BROKEN_PIPE`）按 EOF 处理。
- **测试改造**：raw mode 恢复检查加 Windows 实现，比较 `GetConsoleMode` 前后的值，这是测试里唯一的 unsafe（豁免见 [052](052-unsafe-exemption-rule.md)）；`tests/errors.rs` 的期望行用 `file!()` 构造；suspend-terminal 示例的信号分支加 `cfg(unix)`，保留 `sh`。
- **行尾**：Windows runner 用 `core.autocrlf=true` 检出，加 `.gitattributes` 的 `* text=auto eol=lf`。
- **#969 fixture**：移植 Ink 的 `issue-969-windows-full-height-rerender`，高度等于屏高，每 100ms 重绘到 `frame 8`；底行写满宽度，每帧都写右下角一格。在最终屏幕和回滚上断言顶行、底行、`frame 8` 各出现一次，没有旧帧。除它以外不加测试，不为 Windows 放松 Unix 断言。
- **CI**：push 与 pull_request 触发；fmt 在 ubuntu；三个 OS × 两套 feature 跑 clippy `-D warnings` 和 `cargo test --workspace`；`fail-fast: false`；不加 release、coverage、MSRV。
- 验收（macOS 上能跑的部分）：`pty`、`examples_pty`、`errors` 测试，`--no-default-features` 的 PTY 测试，fmt，两套 feature 的 clippy。

## 结果

- worker 交付：6×40 的 #969 fixture；`check_jest_frames` 在 Windows 上不查帧顺序（conhost 丢弃 `?2026`），改查最终结果、完成标记和光标。偏差：`--no-default-features --test examples_pty` 被 Cargo 拒绝（该目标要求 `macros`），worker 拆成两条命令验证。
- Claude review 后改了一处：worker 在 Windows 上把所有输出当 conhost 重绘解析，但 `run_pipe` 读到的是管道原字节。Claude 加了 `printed()` 辅助函数（Windows 读 vt100 屏幕和回滚，Unix 读原字节），去掉了大部分成对的 cfg 分支。
- 集成验证：win-sys 合入后，macOS 上 fmt、clippy、全量测试通过，Windows 目标的 clippy（all-targets，两套 feature）通过。分两次提交 `f6e1339`、`613126d`，推送 `windows-support`。
- CI run 39467439993 首轮 7 个 job 全部通过。Claude 看了 Windows 日志，确认用例不是被跳过：22 个 PTY 用例、5 个示例 PTY 用例、console 输入解码单元测试都跑了。
- **#969 结论**：不照搬 Ink 的整屏清除。ConPTY 下全屏帧每次重绘都写到右下角，连续 8 次后屏幕和回滚里每行各出现一次。`docs/ink-tests.md` 中 #969 从「不适用」改为「已移植」，render.tsx 移植数 61 改为 62，写明 Windows 的 PTY 观测边界，修正拆分后过时的文件路径。提交 `55f4f55`。
- 两个 worktree 用 `git worktree remove --force` 删除。
- 合进 main 和发布见 [055](055-release-0-1-1.md)。
