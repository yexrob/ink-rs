# 带外控制写入 `write_control`

- 日期：2026-10-20
- 状态：已实施
- 提交：`dc64637` feat(output): add write_control for out-of-band bytes
- Brief：[write-control](briefs/write-control.md)

## 背景

kitty 图片 demo（`~/tmp/ink-rs-demo`，不属于 ink-rs）用 Unicode placeholder 显示图片。图片数据要绕过帧写给终端：`Text` 里的 APC 会被过滤，`use_stdout().write` 会擦帧、补换行、重画。demo 只能在挂载前和按键回调里写进程 stdout，这会和帧的 diff 交错，ink-rs 也不知道这些写入。

用户要求「你看看bingo 之前的图片是怎么展示的」。只读调研发现 bingo 用同一个协议，图片数据走 `Screen::place`：帧写完之后发，不进 cell buffer。Claude 据此建议 ink-rs 加一个带外写入 API。用户问「这个是什么」，Claude 给出设想：字节先进队列，在下一帧写完之后原样发出；不擦帧，不补换行，不动光标，不计入帧高度。用途是 kitty 图片数据、窗口标题（OSC 0）、剪贴板（OSC 52）、桌面通知（OSC 9）这类不显示字符的字节。Ink 没有对应 API，属 ink-rs 的扩展。

用户随后在 P4 的讨论中途回复「那你加一个带外写入通道 api吧 按照你规划的」。

## 决策

- 加 `OutputWriter::write_control(bytes)`，经 `use_stdout()` / `use_stderr()` 取得。用户决定加，细节交给 Claude（「按照你规划的」）。
- 发送时机：在包含调用前所有状态改动的那一帧写完后发出；这一步没有帧要画时，在步骤末尾发出。用 `runtime.has_work()` 判断 effect 设置的状态是否还要再提交一帧。
- 交出终端期间（`suspend_terminal`、Ctrl+Z）字节留到恢复后的第一帧之后；交出期间卸载则丢弃。`testing::render` 丢弃。
- `external_write` 和 `clear` 不发送队列：它们重画的是上一帧，不是下一帧。
- 字节若打印文字或移动光标，会让下一帧的 diff 对不上，文档写明，代码不做检查。

规则见 [终端输出](../design.md#终端输出) 的"带外控制写入"一条。

## 方案

- 派 Codex，worktree `wt/write-control`，与 P4（[039](039-tokio-driver-use-future.md)）并行。两者文件基本不重叠；brief 让本任务不碰 `src/driver/` 和 `src/runtime/mod.rs`。
- Brief 要点：
  - `WriteDestination` 加 `write_control`，没有默认实现；`AppOutput` 加队列 `control`。
  - 三个发送点：`AppOutput::paint` 写完帧后；`render_if_due` 末尾无帧待画时；`unmount` 在最终帧之后、`begin_unmount` 之前（挂起中卸载或 panic 时清空队列）。
  - 每段字节单独一次 write，不包同步输出序列。
  - 新文件 `tests/write_control.rs`，每条发送规则一个测试，共 5 个。
- 验收：`cargo build`、`cargo test --lib`、指定的 7 个测试文件、clippy，都带 `--offline`。全量测试由主 agent 在集成时跑。

## 结果

- Codex 无偏差，指定测试 76 个通过。集成后全量 1302 个测试通过，clippy、examples 和文档构建通过。
- 集成时 Claude 改写了 `use_stdout` 文档里 Codex 加的一句，在 `docs/design.md` 加"带外控制写入"条目。
- kitty demo 改用 `write_control`：首帧后由 effect 发送图片，按 `r` 重发。用户在终端里确认「能看到图」。
- demo 的每格行列变音符是 demo 自身的改动，用户答「改」后由 Claude 在 demo 中完成，ink-rs 不变。
