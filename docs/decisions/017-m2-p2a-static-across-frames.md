# M2 P2 拆分与 P2a：Static 跨帧只输出一次

- 日期：2026-10-20
- 状态：已实施
- 提交：`01a3100` feat(runtime): emit Static items once across frames
- Brief：[m2-p2a](briefs/m2-p2a.md)、[m2-plan-notes](briefs/m2-plan-notes.md)

## 背景

M2 输出层完成后（见 [015](015-m2-p1-output-layer.md)），剩下的是 Static 跨帧、应用层、光标与外部写出、resize 和同步驱动。用户决定 Static 是「host 原语，M2 只输出新条目 (Recommended)」（见 [011](011-m1-runtime-plan.md)）。

## 决策

- 剩余工作拆成 P2a → P2b → P3a → P3b → P2c，按顺序执行。理由是它们改同一批文件，且 P2c 需要用户先定几个问题。由 Claude 在用户委托下定（「确认 按照你的思路来」）。
- Static 的条目以实例区分，卸载在呈现之后执行，仿 React 的 flushPassiveEffects 顺序。

## 方案

Codex 执行，02:45 UTC（本地 10:45）派发。Brief 要求：

1. 只写出新条目 `[written..len]`。
2. 条目卸载在呈现之后执行；按实例区分条目。
3. `Frame` 增加 `static_changed`。
4. `testing::render` 累积 Static 的行。
5. 移植 C1-C11 和 SB1/3/5。

后续阶段的设计要点写在 [m2-plan-notes](briefs/m2-plan-notes.md)。Static 的设计见 [终端输出](../design.md#终端输出)。

## 结果

- Codex 报告：用 `Rc<Dirty>` 的 `ptr_eq` 区分实例；新增 `pending_static`；lib 148 个、点名的集成测试 124 个通过。
- 主 agent review：新增 `paint::serialize_row`，去掉重复的行序列化代码。
- 全量 753 通过。
- 后续：P2b 见 [018](018-m2-p2b-app-layer.md)。
