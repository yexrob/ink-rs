# `use_box_metrics`

- 日期：2026-10-20
- 状态：已实施
- 提交：`f3ca035` feat(runtime): add use_box_metrics
- Brief：[m4-p1b](briefs/m4-p1b.md)

## 背景

M4 计划的 P1（见 031）。Ink 的 `useBoxMetrics` 返回一个 Box 的尺寸与相对父节点的位置，并在布局变化时重渲染。ink-rs 有 `use_node_ref` 与 `measure_element`，但 `NodeRef` 分不清「未挂载」与「尺寸为 0」，也没有布局变化时通知组件的机制。

## 决策

- 用户批准 M4 计划时（「可以」）同意：第一帧就给真实值，比 Ink 早一帧，记为差异。
- Claude 在 brief 中定下实现形状（属 M4 计划内，未另行请示）：
  - `NodeRef` 的槽改为 `State<Option<NodeLayout>>`，`None` 表示未挂载。不另加「已挂载」标记，避免两个字段不同步。
  - Store 中加 box reader 注册表，仿照 focus reader：弱引用，死掉或已卸载的条目在检查时清除。
  - 每轮提交在 ref 刷新之后、layout effect 之前比较每个 reader 当前的度量与上次返回值，不同则标脏，本轮循环在 paint 前重渲染。空闲时没有提交，也就没有检查和唤醒；布局不变时不标脏。

## 方案

- 派给一个 Codex，与 P1a（`use_animation`，见 033）并行，各自在独立 worktree。为减少合并冲突，brief 指定 Store 字段放在 `focus` 之后，hook 放在 `use_node_ref` 之后，不碰 P1a 修改的函数。
- API：`BoxMetrics { width, height, left, top, client_width, client_height, has_measured }`，`left`/`top` 相对父节点布局盒，不含滚动偏移；`Scope::use_box_metrics(NodeRef)`。
- `measure_element` 的公开约定不变，`None` 时返回 0。ref 刷新不调度任何工作（同 React）。
- 测试：`tests/use_box_metrics.rs` 移植 16/19。3 个不适用：`concurrent=true` 重复 `concurrent=false`；`removes resize listener on unmount` 与 `does not crash when resize fires after unmount` 测的是 Node 的 EventEmitter，且 ink-rs 的卸载消耗 app。
- 只跑指定的测试目标与 clippy，全量测试留给集成。

## 结果

- Codex 的报告：16 个测试全部通过。调整的断言：两个 resize 用例检查屏幕；`does_not_trigger_extra_re_renders_when_layout_is_unchanged` 把 Ink 允许的 2..=3 次渲染改为 2 次；首帧用例加了第 0 帧含 `100x1` 的断言，覆盖首帧交付的规则。
- Claude review 通过后在主仓库集成：全量 1183 个测试加 13 个 PTY 用例通过，clippy 无警告。
- 文档：`docs/ink-tests.md` 计数改为 16，新增「M4 文件中不移植的测试」一节和首帧差异条目；[design.md 的元素尺寸一条](../design.md#编程模型)写入 `use_box_metrics`。
- worktree 与分支已删除。
