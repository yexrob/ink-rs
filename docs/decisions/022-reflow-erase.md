# 宽度变小时按重排行数擦除
- 日期：2026-10-20
- 状态：已实施
- 提交：`fa894b7` fix(output): erase reflowed frames on width shrink；`0c239fa` fix(output): avoid tmux scroll-on-clear when erasing
- Brief：[m2-reflow](briefs/m2-reflow.md)

## 背景
[021](021-m2-p2c-sync-driver-decisions.md) 的 tmux 验收中，窗口变窄后屏幕上留下旧帧顶部。M2 照 Ink 的 `log.clear` 按绘制行数擦除，终端把长行折成多行后，上移距离不够。

Claude 提出：「可以改成按重排后的行数擦除：旧帧每行按新列宽算出会折成几行，就擦几行。」并列出代价：xterm 这类不重排的终端会多擦掉帧上方的历史行；行为偏离 Ink。用户回复：「我也觉得 可以改」。

## 决策
- 宽度变小时按上一帧重排后的行数擦除。Claude 提出，用户同意。
- 接受上述两项代价，写进 design.md。
- tmux 的 `scroll-on-clear` 用 EL + 右移一格 + ED + CR 规避。Claude 在集成时决定。
- 不改为整行重写来应对 tmux 的行长规则，因为 Ink 增量模式的测试要求截短的行不重写未变的前缀。Claude 决定。

## 备选与取舍
- 按绘制行数擦除（Ink 的做法）：在 tmux 等重排终端留下旧帧顶部。
- 整行重写：能让 tmux 的行长与内容一致，但违反 Ink 增量模式测试，放弃。

## 方案
派给 Codex。brief 要点：
- Output 按字形遍历计算重排：上移距离为光标之前各逻辑行的重排高度加光标在本逻辑行内的偏移，然后 CR + `CSI J`。
- 测试用的 `Screen::resize` 补上重排模型。
- `tests/terminal_resize.rs` 新增 2 个 ink-rs 自有测试。
- 更新 design.md 与 ink-tests.md。
- 验收命令：`cargo test --lib output`；`cargo test --test terminal_resize --test render --test cursor --test clear_rerender`；clippy。

规则见 [终端输出](../design.md#终端输出) 的 resize 一条，模型见 [测试](../design.md#测试)。

## 结果
- Codex 报告：新 Screen 模型配旧 Output 时 4 个测试失败（`clear_screen_when_terminal_width_decreases`、`consecutive_width_decreases_trigger_screen_clear_each_time`、`width_decrease_clears_last_output_to_force_rerender`、`rerender_on_resize`），配新 Output 全部通过。vt100 对宽字符有限制，写入 design.md。
- 集成后全量 842 通过、0 失败、3 忽略。tmux 中按 80→10→30→6 列调整，仍有残留。
- Claude 定位两处原因：
  - tmux 默认开启 `scroll-on-clear`，ED 从屏幕原点开始时先把整屏推进 scrollback。改为 `erase_down` = `\x1b[2K\x1b[C\x1b[J\r`，四处 ED 都改用它。
  - 光标位于内容之后时，tmux 把它映射到最后一个物理行。Output 和 Screen 同步改正。
- 写入 design.md 的 tmux 限制：以屏幕底部为锚重排；行长按写过的最远格计算，EL 不缩短；一列宽的屏幕不在处理范围。
- 两个提交各自通过全量测试。`fa894b7` 含 Codex 的实现和 Claude 的改动；`0c239fa` 由 Claude 完成，ink-tests.md 的措辞一并修正。
- 含宽字符的行在 tmux 中仍会少擦，见 [030](030-resize-residue-app-level.md)。
