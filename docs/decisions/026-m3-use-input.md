# M3 P3a：输入派发与 use_input
- 日期：2026-10-20
- 状态：已实施
- 提交：`ca49dd3` feat(input): dispatch stdin input to use_input hooks
- Brief：[m3-p3a](briefs/m3-p3a.md)

## 背景
[023](023-m3-input-architecture.md) 的 P3 是输入派发与 hooks。Claude 把它拆成 P3a（`use_input`、Ctrl+C 退出、raw mode 计数、Esc 冲刷）和 P3b（粘贴、kitty）。P3a 的输入管线放在 Runtime，`testing::render` 和 `testing::mount` 都能喂入输入。

## 决策
Claude 在规划时决定：
- BC-04、BC-05、RI-03 推到 P3b。
- HK-02 推到 P5 的 PTY 测试。
- CMP-06 对应 Ink 的 `isRawModeSupported`，ink-rs 没有这个 API，记为缺 API。
- raw mode 的开关推迟到一步结束时执行；Ink 在 Node microtask 中关闭。
- stdin 不是 TTY 时，应用以 `Unsupported` I/O 错误退出；Ink 抛 JavaScript 异常。

## 方案
13:31 派给 Codex。brief 要点：
- `Scope::use_input(active, handler)`；`Options::exit_on_ctrl_c`，默认 true；`Terminal::{stdin_is_tty, set_raw_mode}`。
- 持有者在 passive effect 中注册，按注册顺序派发；处理函数取最近一次 render 的值。
- 过滤未映射的控制序列；Ctrl+C 的按下与重复触发退出，释放事件照常派发；回调请求退出后，同一次读取剩下的事件不再派发；Esc 等待 20 ms 后交出。
- 测试驱动：`TestTerminal::{with_stdin_tty, raw_mode_calls}`，`TestApp::input`。
- 移植约 101 个测试，分布在 10 个测试文件。

规则见 [输入](../design.md#输入)。

## 结果
- Codex 报告：输入状态 `InputState` 放在 runtime store，`src/runtime/input.rs` 负责解析、过滤、粘贴回退、Ctrl+C 与派发；派发时不跨处理函数借用 store。卸载时先关 raw mode。`ProcessTerminal` 的 raw mode 设置留作桩，P5 实现。模块级 dead_code 豁免去掉。
- Claude review 的改动：回调替换改为一次赋值；测试中的 `std::rc::Rc`、`std::cell::RefCell` 全路径改为 import。
- 全量 1101 通过（新增 101）、0 失败、3 忽略；clippy 无警告。
- ink-tests.md 与 design.md 同步。不移植：React transition、Node 监听器计数、需要 PTY 的 legacy Ctrl+C。
- 后续：BC-04、BC-05、RI-03 见 [027](027-m3-paste-kitty.md)；HK-02 见 [029](029-m3-production-driver.md)。
