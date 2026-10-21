# M3 P2：退出 API 与退出值
- 日期：2026-10-20
- 状态：已实施
- 提交：`5661428` feat: add app exit requests and results
- Brief：[m3-p2](briefs/m3-p2.md)

## 背景
[023](023-m3-input-architecture.md) 定下：外部卸载走 `AppHandle::exit()`；`wait_until_exit` 返回退出值或错误，退出值类型为 `Box<dyn Any + Send>`。P2 与 P1 并行。

## 决策
- 退出值类型 `Box<dyn Any + Send>`。用户：「那个泛型还有Any的就按照你说的用Any」（13:20，P2 派出之后；brief 和 design.md 按此写成，不用改）。
- 三个方法区分无值、有值和错误：`exit`、`exit_with`、`exit_with_error`，以第一次请求为准。brief 规定。
- 退出请求在 app 一步结束时生效，Ink 在 `handleAppExit` 中进入卸载。Codex 列为与 Ink 的差异。
- 结果优先级：应用错误 > 卸载时的终端 I/O 错误 > 退出值。brief 规定。

## 方案
13:13 在 worktree `wt/m3-p2` 中派给 Codex。brief 要点：
- `Error { Io, App }`，`Display` 与 `source` 转发内部错误，实现 `From<io::Error>`。
- `AppHandle::{exit, exit_with, exit_with_error}`，任何线程可调用，唤醒驱动。
- App 在一步末尾卸载；`wait_until_exit -> Result<Option<Box<dyn Any + Send>>, Error>`。
- 测试驱动增加 `TestApp::exit_result`。
- 测试：render.rs 的 R107，components.rs 的 C21、C22，新文件 `tests/exit.rs` 的 E15，另加 1 个驱动单元测试。

规则见 [退出](../design.md#退出)。

## 结果
- Codex 报告四条验收命令通过。为满足本机 clippy 的 `question_mark` lint，Codex 改写了 `src/input/keypress.rs`，提议单独提交。
- Claude 没有合入这处改写，因为 master 上 clippy 是干净的。
- `examples/counter.rs` 跟随新的等待结果调整。
- 全量 1000 通过、0 失败。
- 同类 clippy 改写在 P3b 时以 `e15e185` 提交，见 [027](027-m3-paste-kitty.md)。
