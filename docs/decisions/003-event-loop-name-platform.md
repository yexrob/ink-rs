# 事件循环、写法、名字与平台

- 日期：2026-10-20
- 状态：已实施
- 提交：`54b8da5` docs: record event loop, naming and platform decisions
- Brief：无

## 背景

[001](001-goal-and-approach.md) 的方案留了四个问题给用户：异步怎么接、v1 用 builder 还是宏、crate 名、v1 是否支持 Windows。用户贴回逐条答复，问「这是我对于你刚才问题的回复，你觉得如何」。

## 决策

1. **事件循环选 A**：用户答「选 A，前提是把核心做成步进式：核心只提供 dispatch 和 render_if_due，循环由外层驱动。tokio 作为默认开启的 feature。」用户给的理由：
   - 改方向的代价不对称：A 加 tokio 写法只多一个驱动；B 退回运行时无关要改公开 API。
   - 调度规则要由 ink-rs 自己掌握：批量更新、`max_fps`、空闲零唤醒写在自己的循环里才能用测试锁住；B 把输入、更新、任务放进一个 `select!`，正是 iocraft 出现饥饿的结构。
   - 测试确定性：假时钟、帧计数、空闲零渲染断言依赖手动推进循环。
   - B 的写法在 A 里也能拿到：tokio 驱动把 UI 循环放在 `LocalSet`，`spawn_local` 的任务可以写 `State`。
2. **写法**：「先做builder吧 框架整体可用了再做 宏」。
3. **名字**：用户先说「你换一个 和我确认」，看过候选后问「叫Ink-rs不行吗」，最后「确定用ink-rs」。包名 `ink-rs`，lib 名 `ink_rs`。
4. **Windows**：「先不支持」。
5. **占名**：用户答「没问题 先占住吧」，Claude 在 crates.io 发布 `ink-rs` 0.0.0 占位版本。

Claude 对第 1、4 条的补充，随提交写进 design.md：

- `render_if_due` 返回下一次唤醒时间 `Option<Instant>`，`None` 表示空闲，驱动按它睡眠。
- 核心定义唤醒接口：其他线程经 `AppHandle` 提交更新，或 UI 线程上的任务写 `State`，都要叫醒正在睡的驱动；同步驱动用 channel，tokio 驱动用 `Notify`。
- tokio 的 `JoinHandle` drop 时不取消任务，要包一层句柄，drop 时 abort。
- CI 跑 `--no-default-features`，覆盖同步驱动。
- Unix 专属代码（终端模式、SIGCONT、挂起）放在 `cfg(unix)` 后面，不写 Windows 分支。

## 备选与取舍

- **B：一开始基于 tokio**：API 写起来顺，但公开 API 带 tokio 类型，绑死运行时。理由见决策 1。
- **名字候选**：Claude 给了 `tinta`（推荐，几种语言里的"墨水"）、`irongall`（铁胆墨水，iron/rust 双关）、`ink-tui`。用户选 `ink-rs`。Claude 说明了两项代价：在 Rust 生态里容易和 Parity 的 ink! 混淆；Rust API Guidelines 建议 crate 名不加 `-rs`。好处是一眼能看出和 Ink 的关系。

## 方案

主 agent 改 `docs/design.md`：

- 新增 [事件循环](../design.md#事件循环) 一节，写步进式核心、唤醒接口、两个驱动，以及不让 tokio 进核心的理由。
- 名字一行写明对外介绍时说明和 ink! 无关。
- "待定"一节换成 [平台](../design.md#平台)：v1 支持 Linux 和 macOS；"输入"一节删去"含 Windows"。
- 写法一节原文就是先 builder、宏放 M5，没改。

## 结果

- 提交 `54b8da5`，代码不用改（项目名本来就是 `ink-rs`）。
- 占位包许可证由 Claude 定为 `MIT OR Apache-2.0`，并记下正式发布要带上 Ink 的 MIT 声明。
- 驱动的里程碑排期（testing M1、sync M2、tokio M4）在 [005](005-global-architecture.md) 定。Windows 支持后来重新提出，不在本范围。
