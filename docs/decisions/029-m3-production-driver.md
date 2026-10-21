# M3 P5：rustix termios、stdin 读线程与 PTY 测试
- 日期：2026-10-20
- 状态：已实施
- 提交：`33ee083` fix(output): leave newline translation to the tty；`a85eab2` feat(driver): read stdin in raw mode through rustix termios；`32c7bc5` test(driver): port process exit fixtures to a PTY harness
- Brief：[m3-p5](briefs/m3-p5.md)

## 背景
[023](023-m3-input-architecture.md) 中用户选定「rustix termios，去掉 crossterm (Recommended)」：照 libuv 的 raw 分支保留 OPOST、设置 ONLCR。这样 M2 外部输出中 `\n`→`\r\n` 的改写和对应的 Ink 差异可以删掉。P5 与 P4 并行，14:11 派出。

## 决策
Claude 写 brief 时决定，派出时告知用户可以改：
- stdin 读到 EOF 后，输入不再让应用保持运行。
- PTY 测试放在 `harness = false` 的测试目标里，运行时重新启动自身作为被测程序；另加一项 Ink 没有的检查：退出后 termios 恢复原样。Claude 称这是 P5 最大的风险点。
- 多个应用时，输入交给最近开启 raw mode 的应用，它关闭后交给前一个。M3 结束时 Claude 再次向用户列出此项，记录中未见用户修改。

## 方案
brief 要点：
- 去掉 crossterm，尺寸改用 rustix `tcgetwinsize`；termios 照 libuv 的标志位，模式切换用 `TCSADRAIN`，紧急恢复用 `TCSANOW`。
- 读线程用 `filedescriptor` 的 poll 同时等待 stdin 和 socketpair 唤醒端；新增 `Message::Input`、`InputEnded`；`natural_end` 加上 raw mode 与 EOF 条件。
- 正常卸载、panic hook、信号退出等路径都恢复 termios。
- 删除外部输出的 `\n` 改写，`Screen` 模拟 ONLCR。
- `tests/pty.rs`（portable-pty 0.9）：E1–E5、E7–E11、E13、HK-02、#725，E9 加 termios 标记。

规则见 [输入](../design.md#输入) 与 [平台](../design.md#平台)。

## 结果
- Codex 报告：读线程状态按进程保存，锁顺序为 `RestoreState → stdin lifecycle → input route → failure`；`panic = "abort"` 下 raw mode 仍会释放，输出快照的重试不会发生。
- Claude review：termios 标志与 libuv 一致，锁顺序写清，EOF 与错误路径正确；4 个原生测试覆盖 Ink 没有的规则，保留。一个 PTY 用例名改回 Ink 原文的弯引号。
- 全量 1167 通过、0 失败、3 忽略；PTY 13 个通过；clippy 无警告。
- Claude 把整块改动拆成三个提交，每个单独跑过全量测试。拆分时 `git stash` 操作使工作树不一致，用事先保存的快照恢复，`diff -r` 核对。
- Claude 给 demo 加上键盘操作，在 tmux 中检查：Tab、Shift+Tab、Enter、空格、方向键、Esc 正常；q 与 Ctrl+C 退出后 `stty -a` 前两行与启动前一致。
- 没有验证：Linux；panic 与终止信号的子进程场景。
- 用户试用 demo 后提出 resize 残影，见 [030](030-resize-residue-app-level.md)。
