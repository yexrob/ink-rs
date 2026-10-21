# 卸载时完整写出被裁的最终帧

- 日期：2026-10-20
- 状态：已实施
- 提交：`2fcc2c3` feat(output): write a cut final frame in full at unmount
- Brief：[final-frame](briefs/final-frame.md)

## 背景

P2 的 tmux 手动检查中（见 035），错误界面高于屏幕，顶部被裁掉。非全屏模式下，ink-rs 的活动帧以屏高为上限，超出部分从顶部裁掉；卸载时也是这样，被裁的行不进 scrollback。Ink 整帧写出，顶部滚进 scrollback，翻上去能看到。

## 决策

- Claude 在 16:32 提出，16:40 交付 P2 时再次询问：「卸载时最终帧比屏幕高，现在 ink-rs 会裁掉顶部。Ink 的做法是整帧写出，让顶部进入 scrollback。我建议改成和 Ink 一样，作为 P3 之后的一个单独任务。」理由是卸载后没有新帧会擦掉它，不会污染 scrollback。
- 用户 16:45 回复「同意」。
- Claude 在 P3 运行期间调研，改动落在 `src/app.rs`，与 P3 重叠，定在 P3 集成后派发。

## 方案

- 只改卸载：运行期间帧超过屏高时照样裁顶。
- `Output::uncap(frame, columns, output)`：最后一帧被裁过时，擦掉可见部分，以 `rows = usize::MAX` 重写整帧，被裁的行进入 scrollback，只进一次。
- `AppOutput::restore` 在 `done` 之前调用它。
- 不受影响：备用屏、非交互模式、非 TTY、挂起中、`clear()` 之后卸载。被信号终止或 panic 时走紧急恢复，`restore_bytes` 快照仍是裁剪后的帧。
- 测试：`tests/render.rs` 加 `unmount_writes_a_frame_taller_than_the_viewport_in_full`，5 行终端里放 8 行的帧，卸载后 scrollback 与屏幕依次是 shell 行和全部 8 行，每行一次。改动前失败。

## 结果

- 17:38 派给 Codex，17:42 提交 `2fcc2c3`。全量 1297 个测试通过，clippy 无警告，示例能编译。
- tmux 手动验证：80 行的帧，按 q 退出后 scrollback 依次是 `shell-before`、`tall line 0`–`79`（各一次）、`q to quit`、shell 提示符。
- 文档：[design.md 终端输出一节的非全屏模式规则](../design.md#终端输出)写入卸载时整帧重写；`docs/ink-tests.md` 同步更新。
