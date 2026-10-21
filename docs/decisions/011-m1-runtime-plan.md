# M1 运行时方案

- 日期：2026-10-20
- 状态：已实施
- 提交：`3a76946` docs: plan the M1 runtime；`c0facbe` docs: keep components with equal props when a background changes
- Brief：无。调研派给两个 Claude 子 agent（general-purpose），prompt 未存档为 brief，要点见 方案。

## 背景

M0 完成后进入 M1（组件、hooks、运行时）。按约定，里程碑调研之后先与用户讨论架构，再派发。

## 决策

用户在 01:22 UTC（本地 09:22）回答了三个问题，01:29 确认整体方案：

- layout effect 的时机：用户答「你推荐哪一个？」。Claude 推荐「布局后、paint 前」，理由有四条：
  - 与 design.md 的帧顺序一致；
  - 不出现闪烁帧，与 React DOM 相同；
  - 与 `render_to_string` 共用一种顺序；
  - 代价是比 Ink 少一帧。
- 元素测量（`measure_element`）：用户选「提前到 M1 (Recommended)」。
- Static：用户选「host 原语，M2 只输出新条目 (Recommended)」。
- 整体方案：用户答「确认 按照你的思路来」。

## 方案

两个调研 agent 于 00:30 UTC 派出：

- 「Inventory M1 Ink tests」：盘点 M1 相关的 Ink 测试，共 105 个，写入 `m1-research/tests.md`。
- 「Summarise Ink runtime semantics」：总结 Ink 运行时语义，写入 `m1-research/runtime.md`。

Claude 在 01:28 UTC 给出的方案：

- 帧顺序；`component(f, props)` 以 `TypeId` 区分组件；key；hooks。
- `State` 是 `Copy` 的分代句柄，登记在线程局部的注册表里。
- `measure_element`、背景色 context、Text 的内联树、Static 作为 host 节点。
- 移除样式时整体替换 `Style`。
- `render_to_string` 走运行时。
- 测试驱动：100 列，每次渲染记录帧，提供 `settle`。
- 键盘节流与 Ink 相同，移到 M2。

测试分拣：

- 移植 74 个；ME11 移到 M2。
- 不移植 30 个：
  - React 专有：RC1-4、RC13、SA1；
  - 并发模式的重复：CR3、CR4、RC14；
  - ME10；
  - 在 Rust 里变成重复的：AC3、PO3、RC12；
  - styles.ts 的 17 个。
- 另写 5 个 Rust 验收测试。

分三阶段：

- P1：RS1-4、SSR1-4、CO2，见 [012](012-m1-p1-runtime-core.md)。
- P2：54 个测试加 5 个验收测试，见 [014](014-m1-p2-stepping-core.md)。
- P3：ME1-9、ME12-13，见 [016](016-m1-p3-node-refs.md)。

设计见 [编程模型](../design.md#编程模型) 和 [一帧](../design.md#一帧)。

## 结果

- `3a76946` 把方案写进 design.md。
- `c0facbe` 修正方案中的一处错误：背景色变化时，props 相等的组件不重新渲染，只有 Text 重新渲染，与 React context 相同。
