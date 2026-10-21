# M1 P1：组件、hooks 与带 key 的协调

- 日期：2026-10-20
- 状态：已实施
- 提交：`4cda548` feat(runtime): add components, hooks and keyed reconciliation；`2154fe7` test: port the render-to-string, static replacement and hooks cases
- Brief：[m1-p1](briefs/m1-p1.md)

## 背景

用户确认 M1 方案（「确认 按照你的思路来」，见 [011](011-m1-runtime-plan.md)）后，派发第一阶段。

## 决策

- 按 [011](011-m1-runtime-plan.md) 的方案实施 P1。实现细节由 Claude 在用户委托下定（「确认 按照你的思路来」）。

## 方案

Codex 执行，于 01:34 UTC（本地 09:34）派发。Brief 要求：

1. 组件函数用 const 断言保证是零大小类型（ZST）。
2. 新建 `src/runtime/`，按 React 的规则协调子节点；被移除的节点进空闲链表复用。
3. effect 按 React 的顺序执行；重渲染上限 50 轮。
4. RS4 的卸载通过 `Drop` 完成，测试用 `panic_any`。
5. 移植 9 个测试，只跑点名的测试目标。

设计见 [编程模型](../design.md#编程模型)。

## 结果

- Codex 报告没有偏离方案；lib 103 个、集成 260 个通过。
- 主 agent review：删掉 ink-tests.md 中多出的一行「4 个已移植」；把多余的脏标记清除换成 `dirty.replace(false)`。
- 发现三个问题，留给 P2：
  - `has_dirty` 是 O(n·depth)；
  - passive 清理在 layout setup 之前执行；
  - state 槽位的释放时机。
- 全量 625 通过、0 失败、3 忽略，clippy 干净。
- `render_to_string` 改为经运行时挂载，先跑 layout effect 再 paint，drop 时卸载（见 `4cda548` 正文）。
- 后续：P2 处理上面三个问题，见 [014](014-m1-p2-stepping-core.md)。
