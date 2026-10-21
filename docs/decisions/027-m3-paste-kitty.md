# M3 P3b：use_paste 与 kitty 键盘协商
- 日期：2026-10-20
- 状态：已实施
- 提交：`e15e185` refactor(input): use ? for the SS3 N prefix；`9a74654` feat(input): add use_paste and kitty keyboard negotiation
- Brief：[m3-p3b](briefs/m3-p3b.md)

## 背景
P3b 接在 P3a 的内部结构上（见 [026](026-m3-use-input.md)），P3a 合入后定稿。范围：`use_paste` 与 bracketed paste 模式、kitty 键盘协议的协商、恢复终端时的模式字节，以及 P3a 推过来的 BC-04、BC-05、RI-03。

## 决策
- `Options::kitty_keyboard` 默认 `Disabled`，同 Ink 的 opt-in。
- 两处与 Ink 不同，由 Claude 决定，派出时告知用户，合入时写进 design.md：
  - kitty 的 200 ms 期限：Ink 用定时器；ink-rs 在收到回复时检查是否超时，不加入唤醒时间。
  - 粘贴模式的开关：Ink 在 effect 中立即写序列；ink-rs 在一步结束时比较前后状态，同一次提交中替换 `use_paste` 组件时不写任何序列。
- 恢复顺序：先 `?2004l` 关闭粘贴模式，再 `<u` 弹出 kitty 模式，然后是原有的屏幕或最后一帧字节。

## 方案
13:50 派给 Codex。brief 要点：
- `Scope::use_paste`；粘贴 hook 活跃时，粘贴内容不交给 `use_input`；没有粘贴 hook 时沿用 P3a 的回退。
- 新文件 `src/input/kitty.rs`：`KittyKeyboard { Disabled, Auto(flags), Enabled(flags) }`，`KittyFlags` 支持 `|` 组合；`REPORT_ASSOCIATED_TEXT` 取 24，因为协议要求它同时开启全键上报。
- `App::new` 接收 `now`，用于 kitty 期限。
- 移植 21 个测试。brief 初稿写了不存在的测试目标，Claude 在派出前改正了验收命令。

规则见 [输入](../design.md#输入)。

## 结果
- Codex 报告：粘贴与按键共用一个有序的处理函数集合，按通道标记；模式写入绕过同步输出包裹；panic 卸载路径使用同一套恢复逻辑。
- Codex 再次为本机 clippy 的 `question_mark` lint 改写 `src/input/keypress.rs`，这次以 `e15e185` 单独提交。记录中未见与 [025](025-m3-exit-api.md) 时不合入的做法不同的原因。
- 全量 1122 通过，clippy 干净；ink-tests.md 与 design.md 更新；worktree 删除。
- 同一时段 Claude 在 `~/tmp/ink-rs-demo` 写了第一个 demo。退出时标题栏右边框被挤到下一行；在固定 100×30 的 tmux 窗口重跑，最后一帧正常。Claude 判断原因是 tmux attach 时窗口尺寸变化，重排了已输出的内容，不是 ink-rs 的问题，没有改库。
