# M2 P1：内联的单元格差分输出层

- 日期：2026-10-20
- 状态：已实施
- 提交：`7095183` feat(output): add the inline cell-diff output layer
- Brief：[m2-p1](briefs/m2-p1.md)、[m2-p1.resume1](briefs/m2-p1.resume1.md)

## 背景

M2 方案定下后（见 [013](013-m2-output-plan.md)），输出层不依赖 M1 后续阶段，可以与 M1 P2 并行。

Codex 开始实现前报告了一处冲突：Ink 的 log-update 单元测试自己决定帧末尾带不带换行，ink-rs 的输出层按视口高度决定。

## 决策

都由 Claude 在用户的并行要求（「你看有没有能并行的任务」）和四项 M2 决定之下作出：

- 在 worktree `m2-output`（同名分支）中运行，避免与主工作区交叉。
- 末尾换行冲突的处理规则，见 resume1：
  - 每个测试挑一个视口高度，让它断言的那一帧落在 Ink 给的位置；
  - LU44（从满屏缩短）记为与 Ink 测试的差异，光标停在帧下面一行；
  - LU42 不移植，它只差末尾换行，在 ink-rs 里只会由 resize 引起，由 P3 的 resize 测试覆盖。

## 方案

Codex 执行，02:11 UTC（本地 10:11）派发，02:13 带决定恢复会话。Brief 要求：

1. 按行做单元格差分，只写变化的部分。
2. 只用相对移动，新行用 `\r\n` 到达。
3. 帧高度以屏高为上限。
4. 规定光标停放位置；Static 的写出路径；同步输出。
5. 移植 log-update 的 4 个测试文件，用 vt100 检查屏幕结果。

规则见 [终端输出](../design.md#终端输出)。

## 结果

- Codex 报告：44 个测试，分布为 31/2/8/3；design.md 改为由实例层记录 Static 写到哪里；指出 clippy 1.98.1 在 styled.rs:22 有 question_mark 警告（后来在 `cd47a6c` 中处理，属下一时段）。
- 主 agent review：
  - 删掉研究 ID 和不移植小节中的已移植列表；
  - 删掉脆弱的字节断言、多余的光标移动和多余的挂载检查；
  - 变基时解决 ink-tests.md 的冲突。
- 先在 `m2-output` 分支提交，变基到 master。master 上全量 728 通过、0 失败，clippy 干净。
- 之后删除 worktree、分支和它的 target 目录。
