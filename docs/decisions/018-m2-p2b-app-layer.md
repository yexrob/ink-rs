# M2 P2b：带节流的应用层与测试驱动

- 日期：2026-10-20
- 状态：已实施
- 提交：`48a5c4c` feat(app): add the throttled terminal app and test driver
- Brief：[m2-p2b](briefs/m2-p2b.md)、[m2-p2b.resume1](briefs/m2-p2b.resume1.md)

## 背景

运行时和输出层都有了，缺把两者连到终端的一层：节流、交互与非交互输出、备用屏、卸载。

## 决策

- 按 M2 方案和 [m2-plan-notes](briefs/m2-plan-notes.md) 实施。细节由 Claude 在用户委托下定（「确认 按照你的思路来」）。
- 派发前把 brief 中的名字改成 P2a 实际用的 `Frame::static_changed`、`pending_static`、`serialize_row`。

## 方案

Codex 执行，02:58 UTC（本地 10:58）派发。Brief 要求：

1. `Terminal` trait 和公开的 `Options`。
2. `App` 带节流；Static 变化时绕过节流立即绘制。
3. 交互与非交互两条输出路径；备用屏重放 Static；`clear`；卸载顺序。
4. 测试侧：`TestTerminal`、`mount`、`TestApp`，用假时钟。
5. 移植 render.tsx 的大部分测试。

设计见 [终端输出](../design.md#终端输出) 和 [退出](../design.md#退出)。

## 结果

- Codex 因自己转录 LC8 时的错误（7 个空格写成 6 个）停下。03:11 UTC 由 resume1 继续：
  - 改正 LC8；
  - 修 `.into()` 的编译错误；
  - 离开备用屏时不发 bsu/esu；
  - panic 时先恢复终端，再 drop 运行时；
  - `layout::compute` 留在 `Runtime::paint` 中，改正过期注释；
  - 合并 import。
- 全量 802 通过。两处为适应屏高上限改动的断言记进 ink-tests.md。
- 主 agent 的修改：
  - 调整列表格式和空行；
  - 从「缺 API」中删掉 `patchConsole` 一项。
- Codex 建议拆成两个提交。主 agent 合成一个，因为只提交 runtime 部分时 clippy 会报死代码。
- 后续：P3a 见 [019](019-m2-p3a-cursor-external-writes.md)。
