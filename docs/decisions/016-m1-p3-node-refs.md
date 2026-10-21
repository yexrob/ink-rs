# M1 P3：节点引用与元素测量

- 日期：2026-10-20
- 状态：已实施
- 提交：`637cd36` feat: add node refs and element measurement
- Brief：[m1-p3](briefs/m1-p3.md)、[m1-p3.resume1](briefs/m1-p3.resume1.md)

## 背景

用户决定把元素测量提前到 M1（「提前到 M1 (Recommended)」，见 [011](011-m1-runtime-plan.md)）。P3 是 M1 的最后一个阶段。

## 决策

- API 和语义由 Claude 在用户委托下定（「确认 按照你的思路来」）：
  - `NodeRef` 是 `Copy` 和 `PartialEq`；
  - `measure_element` 返回最近一次提交的布局，用整数单元格；
  - ref 的绑定与解绑仿 React。
- review 后要求 ref 的刷新代价与 ref 数量成正比，不与树的大小成正比。

## 方案

Codex 执行，02:30 UTC（本地 10:30）派发。Brief 要求：

1. `Scope::use_node_ref()`、`View::node_ref`、`measure_element`，加上 `ElementMetrics`。
2. 每次 `layout::compute` 之后、本轮 layout effect 之前，刷新已绑定 ref 的数据。
3. ref 未绑定、View 已卸载或组件已卸载时返回全零，不 panic。
4. 移植 ME1-9、ME12、ME13 到 `tests/measure_element.rs`。ME10 不移植，ME11 需要 M2 的节流。

设计见 [编程模型](../design.md#编程模型)。

## 结果

- 第一版全量通过，review 发现三个问题。02:42 UTC 由 resume1 退回：
  - `round_tree` 在每次计算后遍历整棵树。M2 里提交不节流，这样每轮提交都多一次 O(N) 遍历。改为删掉 `round_tree`，paint 恢复原样，每个 ref 沿路径以 O(depth) 刷新；
  - `Drop for Runtime` 刷新已失效的 ref，循环不做任何事，删掉；
  - 测试文件头引用了只存在于 scratchpad 的研究 ID，改写。
- 修改后 lib 148 个、点名的集成测试 159 个通过。
- 主 agent 跑全量：739 通过、0 失败、3 忽略，clippy 通过。
- M1 完成。
