# 屏幕增高时擦除从 scrollback 拉回的旧帧行

- 日期：2026-10-20
- 状态：已实施
- 提交：`6921715` fix(output): erase frame rows pulled back from scrollback on height growth
- Brief：[fix-resize](briefs/fix-resize.md)

## 背景

遗留项"tmux attach 后 resize 留下旧帧"：一个 demo 在 detached 的 tmux 窗口（100x30）里运行，Ghostty 客户端 attach 后窗口变为 101x50，活动帧上方留下一份旧帧。宽度收缩的重排擦除见 [022](022-reflow-erase.md)，这个问题发生在尺寸增长时。

用户此前问过「resize的时候会写scrollback吗」。Claude 回答 ink-rs 自己不写 scrollback，终端在高度变小时会把帧的顶部行推进 scrollback，ink-rs 往上最多回到屏幕第一行，擦不到这部分，Ink 有同样的限制。

用户要求遗留项「修完再发」（见 [043](043-release-prep.md)）。

## 决策

- 记录溢出行，增高时擦除。用户在提问中选了推荐项「记录溢出行，增高时擦除」（选择见会话摘要和 Claude 的回复「四项都按推荐方案做」）。提问中的说明：只擦这些行，不碰正常的 shell 历史；Ink 也有这个 bug；加 vt100 回归测试。
- 以下由 Claude 在 brief 中定：
  - 计数与 `forget()` 分开，缩矮路径调用 `forget` 后计数仍在。
  - 可能滚屏的输出都清零计数：Static 行、`clear`（其后跟外部写入）、`done`、`uncap`、挂起恢复。理由：「a missed erase leaves today's behavior, an extra erase would delete the user's shell history」。
  - 宽度收缩只在帧顶到光标的各行都不折行时记数；之后宽度小于已记录行的最大宽度时清零，保留 design.md 已写明的限制。
  - 备用屏没有 scrollback，不记数。
  - 在增长路径的注释中写明这是对 Ink 的偏离。

规则见 [终端输出](../design.md#终端输出) 的 resize 一条。

## 备选与取舍

- 记为已知限制：不改代码，在 design.md 写明，Ink 行为相同。提问中的另一项，用户没有选。

## 方案

- 调研：Claude 子 agent（opus-high，"Research stale frame after resize"），在私有 tmux 服务（`tmux -L inkprobe`）上复现，不碰用户的 `ink-rs` 会话。
- 调研结论（写入 brief）：
  - 缩矮时，终端（tmux、kitty、iTerm2、VTE）把 `cursor_row + 1 - new_rows` 行从屏幕顶部推进 scrollback；帧在屏幕底部时这些是帧的顶部行。
  - 增高时，终端把 `min(推入历史的行数, 增量)` 行从 scrollback 拉回屏幕顶部，光标下移同样行数，拉回的是旧帧行。
  - `src/app.rs` 只向 `Output::resize` 传两个布尔值，增长落入 diff，diff 以帧顶为基准，帧上方的旧行不被擦除。
  - Ink 的 `resized` 有同样的 bug。
- 派 Codex（worktree `wt/fix-resize`），与其余 4 项修复并行。Brief 要点：
  - `Output` 加计数；增长且计数非 0 时隐藏光标，`CUU(cursor_row + spilled)`（在屏幕顶部停住），向下擦除，`forget()`，再按增量扣减计数。
  - `Output::resize` 改为接收前后尺寸。
  - `tests/common/mod.rs` 的 `Screen::resize` 在主屏增高时模拟拉回。
  - `tests/terminal_resize.rs` 加 3 个测试：28 行帧 100x30→100x20→100x50 后每行只出现一次；声明光标的同一场景；缩矮与增高之间打印 Static 行时不擦帧上方的内容。

## 结果

- Codex 加 `spilled`、`spilled_width` 字段，增长与宽度收缩共用一次上移和擦除。声明光标的缩矮路径改为上移、擦除、`forget()`，不调用会清零计数的 `clear()`。
- 偏差：Static 测试用 30→20→21 行，不用 →50。计数清零后增长到 50 行会拉回不再追踪的旧帧副本，"不擦帧上方内容"与"屏幕无重复行"冲突，Codex 保留前者。现有测试断言未改。
- Codex 验证：resize 相关 4 个测试目标共 72 项、`--lib` 342 项、examples 22 项与 PTY 5 项通过。
- Claude 审查 `src/output.rs`、`src/app.rs`、测试与 design.md 的改动，集成后跑 fmt、clippy，提交 `6921715`。
