# M2 P3b：resize 与 `use_window_size`

- 日期：2026-10-20
- 状态：已实施
- 提交：`8bcc30d` feat(app): handle resize and add use_window_size
- Brief：[m2-p3b](briefs/m2-p3b.md)、[m2-plan-notes](briefs/m2-plan-notes.md)

## 背景

用户决定 resize「照搬 Ink 的规则」（见 [013](013-m2-output-plan.md)）。M2 P1 把 LU42 留给了 resize 测试（见 [015](015-m2-p1-output-layer.md)）。

## 决策

- 按 Ink 的规则实现 resize 步骤，由用户决定（「照搬 Ink 的规则」）。
- 测试侧用 xterm 的行收缩模型补 vt100 的缺口。由 Claude 在用户委托下定（「确认 按照你的思路来」）。

## 方案

Codex 执行，03:41 UTC（本地 11:41）派发。Brief 要求：

1. 按 Ink 的规则实现 resize 步骤。
2. 新增 `use_window_size`。
3. 在 `tests/common` 中加 xterm 的行收缩模型。
4. 移植 terminal_resize 的测试。

设计见 [终端输出](../design.md#终端输出)。

## 结果

- Codex 报告：`WindowSize { columns: u16, rows: u16 }`；`layout_pending`。移植 terminal_resize 15 个、render 3 个，13 个不移植。lib 151 个、点名的集成测试 167 个通过。
- 主 agent review（03:52 UTC，本时段末尾）：resize 逻辑与 Ink 对照一致；全量 836 通过、0 失败，clippy 干净。
- 提交时间 11:53，在本时段结束之后。
- 「照搬 Ink」的擦除方式在 P2c 的 tmux 实测中留下残留，用户在下一时段同意改为按重排行数擦除。
