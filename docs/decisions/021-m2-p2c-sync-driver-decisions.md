# M2 P2c：同步终端驱动

- 日期：2026-10-20
- 状态：已实施
- 提交：`e1de8fc` feat(driver): add the synchronous terminal driver；`cd47a6c` refactor(text): use ? for the hyperlink terminator
- Brief：[m2-p2c](briefs/m2-p2c.md)、[m2-plan-notes](briefs/m2-plan-notes.md)

## 背景

P2c 是在真实终端上运行应用的入口。Claude 在 03:06 UTC 提了 4 个问题。用户在 03:25 UTC 问：「P2c（同步驱动）还在等你回答之前提的 4 个问题：入口 API、事件循环何时自行结束、跨线程更新的方式、依赖的选择。 这个是啥」。Claude 解释后，在 03:26 UTC 发出提问。

## 决策

用户在 03:28 UTC（本地 11:28）四项都选了推荐项：

- 入口 API：「render + wait_until_exit」。
- 事件循环何时结束：「无事可做时结束」。
- 跨线程更新：「AppHandle::update 闭包」，`State` 因此改为 `Send`。
- 颜色检测：「自己写，照 chalk 的规则」。

下面几项由 Claude 定，写 brief 时告知用户：

- 循环中收到 Ctrl+C 时先卸载再重发信号；循环外只恢复终端。
- panic 时先恢复终端再打印信息。
- 管道断开时循环以错误结束。
- M2 不读键盘，resize 由 SIGWINCH 报告。
- 加 `examples/counter` 作手动验收。

手动验收的方式由用户提出：「你可以使用tmux吗？ 我现在主要用herdr了 不用tmux 你可以用tmux ，这样我也可以观察你用tmux」。此后手动验收都在 tmux 会话 `ink-rs` 中进行，用户可以旁观。

## 备选与取舍

- 入口：另一项是只有一个阻塞的 `run`，没有 `Instance`。
- 结束时机：另一项是只在 `exit()` 或信号时结束。
- 跨线程更新：另一项是 `AppHandle<M>` 消息（Elm 风格）。
- 颜色检测：另一项是 supports-color crate，它的规则与 chalk 不同。
- 所有选项都依赖 crossterm 和 signal-hook。

## 方案

Codex 执行。brief 在 03:34 UTC 写好，03:53 UTC P3b 提交后派发。Brief 要求：

1. `render` 与 `Instance`；循环在无事可做且没有外部 `AppHandle` 时结束，不丢唤醒。
2. `AppHandle` 的队列放在核心层，驱动提供唤醒，M4 的 tokio 驱动复用它。
3. 生产终端；CI 与颜色检测，用传入的环境查询函数实现以便测试。
4. 用 signal-hook 处理 SIGINT、SIGTERM、SIGHUP、SIGWINCH；panic hook 与信号共用恢复状态。
5. 移植两个 CI 测试；CI 规则和颜色规则各一个表格测试；信号和 panic 只做手动检查。

设计见 [事件循环](../design.md#事件循环)、[编程模型](../design.md#编程模型)、[退出](../design.md#退出)。

## 结果

- Codex 报告：lib 155 个、6 个目标 125 个测试通过，clippy 通过。非交互运行的第一帧在 unmount 时写出。Codex 在 ink-tests.md 加了一节手动验收清单。
- Claude review：全量 840 通过、0 失败；默认 feature 和 `--no-default-features` 下 clippy 都干净。改动：
  - design.md 中预设「M3 用 crossterm 事件接管 resize」的句子改为中性说法，因为 M3 的输入方案还没定；
  - 手动验收清单移出仓库，放到 scratchpad；
  - `FORCE_COLOR` 取非数字值时的处理改为照 chalk 的 parseInt，加 `parse_int` 辅助函数。
- `cd47a6c` 是 Codex 顺手做的 clippy 修正，单独提交。
- tmux 手动验收：
  - Ctrl+C 退出码 130，光标恢复显示，帧保留；SIGTERM 143，SIGHUP 129，各自只清理一次；
  - CI 或输出到文件时只写最后一帧加换行；EPIPE 退出码 1，错误为 BrokenPipe；
  - 备用屏收到信号后回到主屏，不重放；`wait_until_exit` 之前收到信号：恢复终端，退出码 143，不跑清理；unmount 之后收到信号：按默认行为处理；
  - UI 线程和 render 中 panic 退出码 101；其他线程 panic 时 UI 继续运行；`panic=abort` 构建在 abort 前恢复终端，退出码 134；
  - 多个 app 同时运行没有验收。
- tmux 验收发现窗口变窄后留下旧帧，见 [022](022-reflow-erase.md)。
