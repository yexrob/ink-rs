# 全局架构与 M0 的 11 项决策

- 日期：2026-10-20
- 状态：已实施
- 提交：`0720bc7` docs: describe global architecture and M0 decisions
- Brief：无

## 背景

用户要求推进 M0：「可以 推进M0吧 使用codex作为worker，你来出详细的方案 和 后面review。注意子agent 在工作的时候不要运行全量的tests，待主agent集成的时候进行全量测试 有问题再抛给子agent解决。」随后补充：「对了 做完M0调研之后先停一下， 先整体讨论一下技术方案和架构」。

Claude 调研 Ink 渲染管线后，先给了 M0 管线方案和 8 项决策。用户指出范围不对：「你这个架构是全局的架构吗 我是让你做全局架构」。Claude 改出 M0–M5 全部完成后的整体架构，列 11 项决策。用户又贴回 [003](003-event-loop-name-platform.md) 的四条答复，问「你是不是忘了这个」，Claude 把它们补进架构。

## 决策

工作方式，用户决定（见背景第一段引文）：

- Codex 做 worker，Claude 出方案和 review。
- worker 只跑指定测试，全量测试由主 agent 集成时跑，有问题再派回 worker。
- 里程碑调研后先停下讨论方案，再派活。

11 项决策由 Claude 提出，用户授权：「那11个决策你按照你的思路来」。

1. host 树用 `TaffyTree<HostNode>`，跨帧常驻。
2. paint 每帧整帧重画，差分放在 output 层，paint 保持纯函数。
3. Text 内部的嵌套内容不进 taffy，taffy 只看到一个叶子；任一处变化整个 Text 重新合并、测量。
4. `State` 经线程局部变量访问；代价是离开 UI 线程会在运行时 panic。
5. 时间作为参数传入，不设 Clock 接口。
6. 测量缓存放在 Text 节点上，M0 不做；不做 Ink 的全局字符串缓存。
7. Text 样式按"内层没设的属性由外层补上"叠加，不复刻 chalk 的字符串包裹。两者只在内容里出现原始 `\e[0m` 时不同，没有测试覆盖。
8. 换行按 wrap-ansi，不按 UAX #14：连续中文放得进下一行时整段下移。
9. 超链接（OSC 8）以外的 OSC 在解析时丢掉：单元格模型没地方放，外部内容也不该能改窗口标题或剪贴板。
10. 由 Ink 实现副作用产生的期望值不去模拟，改期望并记录原因。已知两个：`tests/c1_rendering.rs:54` 的三重 OSC 8；`out_of_bounds_writes_do_not_crash` 的 JS 稀疏数组空洞。
11. 内部函数的单元测试改调内部 API（`parse` → 调用 → `serialize`），不为测试保留字符串签名。

## 备选与取舍

- **host 树自存节点、实现 taffy 的 trait**：控制力更强，多写两三百行。taffy 类型只出现在 host、layout 两个模块，以后要换改动集中。M0 第三阶段改选了这条路，见 [009](009-m0-fix1-and-yoga-gap.md)。
- **局部重画**：等有性能数据再讨论。
- **UAX #14**：设计文档原写法，和 Ink 测试期望冲突，留作以后的选项。

## 方案

主 agent 改写 `docs/design.md`：

- [分层](../design.md#分层)：全局分层图，从上到下为 API、driver（sync / tokio / testing）、步进式 App 核心、runtime、host、layout + text、paint、output、`Terminal` trait（crossterm 与内存两个实现）。
- [跨层约定](../design.md#跨层约定)：单元格是唯一内部表示；时间由外部传入；终端 I/O 走 `Terminal` trait；UI 单线程，跨线程用 `AppHandle`；任务随组件卸载取消；终端恢复用 RAII。
- [一帧](../design.md#一帧)：重跑脏组件、按 key 比对、taffy 重算、paint、`use_layout_effect`、output 写出、`use_effect`，返回下次唤醒时间。
- [事件循环](../design.md#事件循环)：tokio 默认开启，`use_future` 归 tokio feature；同步驱动用读线程加 channel；驱动排期 testing M1、sync M2、tokio M4。
- [布局与文本](../design.md#布局与文本)、[测试](../design.md#测试)：写入决策 6–11 和 Yoga 兼容处理。

Claude 问过 tokio 驱动要不要提前到 M2，记录中未见答复，保持 M4。

## 结果

- 提交 `0720bc7`。
- 用户问「各个M分别有几个Phase」。Claude 答：Phase 只是派活的拆分，当时只拆了 M0（文本层、管线、集成三段）；M1–M4 初估各 3 段，M5 2 段，以届时讨论为准，每个里程碑最后一段是集成。
- M0 第一阶段随即派出，见 [007](007-m0-text-layer.md)。
