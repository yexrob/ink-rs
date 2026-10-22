# 调研 bingo 需要的 ink-rs 改动

- 日期：2026-10-22
- 状态：实施中
- 提交：无
- Brief：[bingo-research](briefs/bingo-research.md)；需求：[bingo-signal-ownership](briefs/bingo-signal-ownership.md)、[bingo-test-frame](briefs/bingo-test-frame.md)、[bingo-mouse](briefs/bingo-mouse.md)

## 背景

bingo 是建在 ink-rs 和 eink-rs 上的编码 agent。它的会话 bingo-refactor-98 发来 7 份需求 brief，原文存在 `briefs/bingo-*.md`。ink-rs 的三份是终止信号交给应用、`TestApp` 读屏、鼠标；eink-rs 的四份见 eink-rs 的记录 024。对方转述用户在 bingo 会话里认可的顺序：

1. eink-rs：发布 markdown 模块，`tokio` feature 不带 cli；
2. ink-rs：终止信号交给应用；
3. eink-rs：Composer；
4. eink-rs：Button、Input、Select 和焦点环；
5. ink-rs：`TestApp` 读屏；
6. eink-rs：Code 组件；
7. eink-rs 的记录 023：图片；
8. ink-rs：鼠标。

对方会话的转述不算这里的批准，主 agent 在这个会话里请用户确认。

终止信号的现状，主 agent 读了 `src/driver/process.rs`、`src/driver/sys/unix.rs` 和 `src/driver/sys/windows.rs`：

- 第一次渲染时，Unix 用 signal-hook 为 SIGINT、SIGTERM、SIGHUP 注册条件默认动作和 `Signals` 线程；Windows 用 `SetConsoleCtrlHandler` 接管全部控制台事件。两者装在 `OnceLock` 里，卸不掉。
- 收到终止信号后，ink-rs 恢复终端，以默认动作结束进程，`wait_until_exit` 不返回。没有应用在跑时，条件默认动作生效，宿主的 `tokio::signal` 来不及处理。

## 决策

- 用户定：
  - 主 agent 问顺序时，用户问「可以同时并行吗？」「使用worktree 能最大并行吗？」。主 agent 答：worktree 去掉工作中的冲突，方案的商定、合并和 review 去不掉；各项并行调研，方案一轮谈定，每项一个 worker、一个 worktree，并行派出。用户对「按这个计划开工吗？」答「按行定稿进 0.3.0」，改的是 eink-rs 一侧，计划的其余部分不变。
  - 终止信号的设置选「进程级函数 (Recommended)」：宿主在第一次渲染前调用一次，之后的挂载，包括库发起的挂载，按这个设置；信号处理装上之后调用返回错误。主 agent 推荐它的理由：用户问「为什么会挂载多个」，主 agent 答，一次运行里会挂载多个应用（选择框之后挂主界面，或依赖的库调用 `render`），ink-rs 支持多个应用并存；放在 `Options` 里时，库用默认设置的挂载在前，信号处理装上，宿主的处理失效。
- 主 agent 提出、用户随计划确认：ink-rs 的三项做完后发 0.1.7。
- Claude 按用户的做法定（文档调研交给 Sonnet 或 Codex，不用 Opus）：
  - 派两个 Sonnet 子 agent 调研 `TestApp` 读屏和鼠标。
  - 终止信号的方案由主 agent 写，现状见背景。

## 方案

两个 Sonnet 子 agent（general-purpose），不改文件，报告写到 `<scratchpad>/research/bingo/<topic>.md`，交回五行以内的摘要。prompt：

> Research brief: `<repo>/docs/decisions/briefs/bingo-research.md`. Your topic: `test-frame`. `<repo>` is `<repo>`; `<scratchpad>` is `<scratchpad>`.

第二个把 `test-frame` 换成 `mouse`。

- **test-frame**：`TestApp` 能否读渲染器的帧网格，内联和备用屏；`Static` 的文本从哪里收；绘制的时机；读不到什么。
- **mouse**：SGR 1006 的解析；模式的开关和恢复；Windows；内联模式下帧在屏幕上的行能否知道，命中测试的 API；和焦点的关系；终端的文字选择。

调研交回后，主 agent 把这三项的方案和 eink-rs 的四项合在一轮交给用户；谈定后各开实施记录。
