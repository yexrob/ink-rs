# 把 Windows console FFI 的 unsafe 豁免写进 CLAUDE.md

- 日期：2026-10-21
- 状态：已实施
- 提交：`1ee66d8` docs: exempt Windows console FFI from the unsafe rule
- Brief：无，主 agent 自己改

## 背景

项目规则第 1 条禁止 unsafe。用户为 Windows 支持批准了豁免：「windows这里 支持用ffi把 豁免」（见 [051](051-windows-plan-sys-split.md)）。Claude 先把豁免记进项目 memory，又提出把它写进 `CLAUDE.md` 和 `AGENTS.md`，让以后的 agent 知道，拟定的句子是：

> Windows console FFI in `src/driver/sys/windows.rs` is exempt (approved 2026-10-21); each unsafe block carries a `// SAFETY:` comment.

用户回复：「可以」。同一条回复也批准了把 `windows-support` 分支推到 origin 跑 CI（见 [054](054-conpty-tests-ci.md)）。

## 决策

- 在规则第 1 条后补一句例外。用户决定（「可以」）。
- `AGENTS.md` 是指向 `CLAUDE.md` 的符号链接，只改 `CLAUDE.md`。
- 范围加上 `tests/pty.rs` 的 console-mode 检查。这是 Claude 自己扩的：win-tests worker 要在测试里调 `GetConsoleMode` 比较输入模式，原本问用户的句子没包括它。Claude 在提交后的回复里向用户说明了这一点，记录中未见用户异议。
- 代码侧的落实（`forbid`/`deny`/`allow` 的组合）随 Windows 驱动一起做，见 [053](053-windows-driver.md) 与 [design.md 平台](../design.md#平台)。

## 结果

- 提交 `1ee66d8`。规则第 1 条现为：unsafe 禁止；`src/driver/sys/windows.rs` 与 `tests/pty.rs` 的 console-mode 检查中的 Windows console FFI 豁免，每个 unsafe 块带 `// SAFETY:` 注释。
- 项目 memory 的豁免条目补注了这次提交和范围。
- 提交在推送 `windows-support` 之前完成，随该分支进入 main。
