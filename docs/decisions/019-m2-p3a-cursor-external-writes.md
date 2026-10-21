# M2 P3a：`use_cursor` 与外部写出

- 日期：2026-10-20
- 状态：已实施
- 提交：`089c703` feat(runtime): add use_cursor, use_stdout and use_stderr
- Brief：[m2-p3a](briefs/m2-p3a.md)、[m2-plan-notes](briefs/m2-plan-notes.md)

## 背景

用户决定 console 输出「只提供 use_stdout/use_stderr 的 write」（见 [013](013-m2-output-plan.md)）。P3a 实现这两个 hook 和光标定位。

## 决策

都由 Claude 在用户委托下定（「确认 按照你的思路来」）：

- `use_cursor` 是内部的 insertion effect。
- `use_stdout` / `use_stderr` 的写出立即到达终端，与 Ink 相同。m2-plan-notes 原写「在步进结束时排空队列」，Claude 核对 Ink 后在 brief 中改了。
- 交互模式下写出时把 `\n` 转成 `\r\n`，末尾补 `\r\n`。这偏离 Ink。原因是 raw 模式会清掉 OPOST，而重绘从第 0 列开始。

## 方案

Codex 执行，03:23 UTC（本地 11:23）派发。Brief 要求：

1. `use_cursor` 及其在输出中的光标停放。
2. `use_stdout` / `use_stderr` 的写出路径。
3. 交互模式下的换行转换。
4. 移植 16 个测试。

设计见 [终端输出](../design.md#终端输出)。

## 结果

- Codex 报告：`CursorPosition { x, y: usize }`；`OutputWriter`、`OutputStream`；`WriteDestination`。lib 151 个、点名的集成测试 148 个通过。
- 主 agent review：`testing::render` 原先用快照链记录帧，改为在每次步进结束时合并。规则是：步进之外由保留的句柄写出的内容，出现在下一次步进之后。这条规则写进 design.md。
- 全量 818 通过。
- 后续：P3b 见 [020](020-m2-p3b-resize.md)。
