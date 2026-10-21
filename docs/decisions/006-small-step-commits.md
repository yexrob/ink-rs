# 允许 agent 小步提交

- 日期：2026-10-20
- 状态：已实施
- 提交：`5bf8b1f` docs: allow agents to commit in small steps
- Brief：[m0-text.resume](briefs/m0-text.resume.md)

## 背景

M0 第一阶段的 Codex 在跑时，用户要求：「在CLAUDE.md中补充允许agent小步提交」。

## 决策

- `CLAUDE.md` 加第 3 条：agent 可以不经询问小步提交；每个提交只含一个逻辑改动，能编译并通过覆盖它的测试，用 Conventional Commits；多个 agent 共用工作区，按路径暂存自己的文件；推送要用户同意。用户提出，条文由 Claude 起草。
- 按路径暂存的理由：Codex 和 Claude 在同一个工作区改文件，按路径暂存能防止把对方的改动带进提交。Claude 说明。
- worker 运行期间，主 agent 不在共享工作区提交。Claude 从下面的事故得出，用户未另行表态。

## 方案

主 agent 改 `CLAUDE.md` 并提交，同时把 design.md 的全局架构提交为 `0720bc7`（见 [005](005-global-architecture.md)）。

## 结果

- 提交 `5bf8b1f`。
- 事故：这两次提交发生在 Codex 第一阶段运行中，Codex 看到工作区变化后停下。它的实现和测试已写完，剩 rustfmt、自查和报告。Claude 写 [m0-text.resume](briefs/m0-text.resume.md) 让它接着做：说明变化来自 Claude，本次运行仍按原指令不提交，由 Claude review 后提交。
- 第二阶段起 brief 按新规则允许 Codex 自己提交。Codex 的 sandbox 写不了 `.git`，实际由 Claude 代为提交，带 Codex 和 Claude 两条 `Co-Authored-By` 尾注，见 [008](008-m0-pipeline.md)。
