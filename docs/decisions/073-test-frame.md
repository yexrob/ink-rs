# `TestApp` 读屏：`live_frame` 和 `static_output`

- 日期：2026-10-22
- 状态：实施中
- 提交：无
- Brief：[test-frame](briefs/test-frame.md)；需求：[bingo-test-frame](briefs/bingo-test-frame.md)

## 背景

bingo 的界面测试用 `testing::mount` 挂载整个应用，想断言屏幕上的内容。`TestApp` 给出的是原始字节（`TestTerminal::writes`、`output`），测试要解析转义序列，读出一帧。调研见 [071](071-bingo-research.md)，报告的结论：在 `AppOutput::paint` 里、备用屏合并 Static 之前记录行，由 `testing::mount` 打开。

## 决策

- 用户随计划确认（「按行定稿进 0.3.0」一轮确认了其余计划）：新增 `TestApp::live_frame()` 和 `static_output()`，读渲染器的帧，不解析字节。
- Claude 按用户确认的计划定细节：
  - 不沿用 `last_frame` 这个名字：`TestInstance::last_frame` 含 Static 的历史，名字相同、含义不同会误导。
  - 记录器默认关闭，`testing::mount` 打开；生产路径不多存一份帧。
  - 格式沿用 `TestInstance` 的行序列化，用 `render` 写的断言在 `mount` 下读到同样的文本。
  - `static_output` 累积，不随第一个 `Static` 实例的变化清空：写进 scrollback 的行留在那里。
  - 读不到的内容：光标位置、结构化的颜色、`use_stdout`/`use_stderr` 的外部写入、`write_control` 发出的字节。

## 备选与取舍

- **解析 `TestTerminal` 的字节**：要实现一个终端模拟器的子集，和渲染器的格式重复。

## 方案

一个 Codex worker，worktree `wt/ink-test-frame`。brief 见 [test-frame](briefs/test-frame.md)。测试两例：内联和备用屏各一，Static 列表加一行动态内容，输入加一项并 `advance` 后，`static_output` 含新项，`live_frame` 含动态行、不含新项。备用屏一例的理由：备用屏路径把 Static 行并进动态行，记录要在合并之前。验收：fmt；clippy 两组 feature；`--lib`；brief 列出的测试文件。

主 agent 集成后补结果。进 0.1.7。
