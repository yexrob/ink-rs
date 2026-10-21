# M1 P2：步进核心与测试驱动

- 日期：2026-10-20
- 状态：已实施
- 提交：`3aaf02a` feat(runtime): add the stepping core and testing driver；`ed5ec79` test: port the rerender cases
- Brief：[m1-p2](briefs/m1-p2.md)

## 背景

P1 提交后，P2 接着做跨帧挂载、按 React 顺序执行 effect，并处理 P1 review 留下的三个问题（见 [012](012-m1-p1-runtime-core.md)）。用户要求找可以并行的任务（「你看有没有能并行的任务」），P2 与 M2 P1 同时派发。

## 决策

- P2 在主工作区运行，M2 P1 在 worktree 中运行，两者并行。由 Claude 在用户的并行要求下安排。
- 实现细节按 M1 方案，由 Claude 在用户委托下定（「确认 按照你的思路来」）。

## 方案

Codex 执行，于 02:11 UTC（本地 10:11）派发。02:10 的第一次派发没有启动：Bash 工具跑的是 zsh，fish 的 `set S` 没有生效。Brief 要求：

1. 把提交（commit）和呈现（present）分开；新增 `render_if_due`。
2. 脏标记改为 O(1)，仿 React 的 childLanes，沿父链传播标记。
3. effect 按 React 的顺序执行；layout 清理和 passive 清理分队列。
4. 新增 `testing::render` 和 `TestInstance`。
5. 移植 54 个测试，加 5 个验收测试；只跑点名的测试目标。

设计见 [一帧](../design.md#一帧) 和 [测试](../design.md#测试)。

## 结果

- Codex 报告没有偏离方案；lib 104 个、集成 333 个通过。
- 主 agent review：
  - 结构符合方案：父链传播脏标记，layout 与 passive 清理分开，passive 清理之后再释放 state；
  - `mark()` 不检查是否已挂载；
  - 在 state.rs 加了一处注释。
- 全量 684 通过、0 失败、3 忽略。
