# M2 终端输出方案与并行调研

- 日期：2026-10-20
- 状态：已实施
- 提交：`d1ff80f` docs: plan the M2 terminal output
- Brief：无。调研派给三个 Claude 子 agent（general-purpose），prompt 未存档为 brief，要点见 方案。

## 背景

M1 P1 在 Codex 上运行时，用户问：「你看有没有能并行的任务」。

Claude 的回答：

- M1 的 P2、P3 依赖 P1 的接口；
- 测试移植依赖 API；
- M2 的实现方案还没讨论。

能并行的是 M2 的调研。Claude 也提出可以做 M3 调研，用户没有回应。

## 决策

用户在 02:01 UTC（本地 10:01）对四个问题都选了推荐项：

- 帧高于屏幕时：「上限为屏高，裁掉顶部」。
- resize：「照搬 Ink 的规则」。
- console 输出：「只提供 use_stdout/use_stderr 的 write」。
- `use_effect` 的时机：「提交之后执行」。

## 备选与取舍

- 超高帧：另一项是照 Ink 清屏重写。选了推荐项后，固定 Ink 清屏字节的 9 个测试不适用。
- resize：另一项是按重排后的物理行擦除。
- console：另一项是用 `dup2` 截获输出，需要 `unsafe` 或平台依赖。选了推荐项后，4 个 patchConsole 测试不适用。
- `use_effect`：另一项是在写出终端之后执行。

## 方案

三个调研 agent 于 01:36–01:37 UTC 派出：

- `tests-render.md`：盘点 render 类测试，181 个。
- `tests-output.md`：盘点输出类测试，173 个。
- `output.md`：总结 Ink 的输出语义。

Claude 在 01:57 UTC 给出 M2 草案：

- 范围：移植约 180 个，约 120 个不适用，约 45 个归 M3–M5。
- 测试用 vt100 作 dev-dependency。它有三个缺口：
  - 字符宽度；
  - resize 时没有重排，也没有回滚缓冲区；
  - pending-wrap 状态下的 EL。
- 输出层规则；节流只作用于 paint 和写出。
- 新增 API；同步驱动。
- 分 P1、P2、P3 三阶段。

决定写入 [终端输出](../design.md#终端输出) 和 [事件循环](../design.md#事件循环)。

## 结果

- `d1ff80f` 提交 design.md 的 M2 部分。
- 之后 M1 P2 在主工作区、M2 P1 在 worktree 中并行运行，见 [014](014-m1-p2-stepping-core.md) 和 [015](015-m2-p1-output-layer.md)。
- 「照搬 Ink 的规则」处理 resize 的决定，后来在 tmux 实测中暴露缩窄窗口后的残留。用户在本时段之后同意改为按重排行数擦除（04:23 UTC），属下一时段。
