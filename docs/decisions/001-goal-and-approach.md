# 项目目标与总体方案

- 日期：2026-10-19
- 状态：已实施
- 提交：`c23bb71` feat: scaffold crate and port Ink M0 tests（design.md 初版随骨架提交）
- Brief：无

## 背景

用户开题：「我们的目标是参考https://github.com/vadimdemedes/ink 来做一个rs版本的对应版本 你觉得怎么样」。

Claude 第一次答复参考了 bingo 用过又拆掉的 iocraft：建议取 Ink 的形状、不取它的运行时，改用 Elm 式模型（`view(&State) -> Element`，不要 reconciler），终端驱动从 bingo 的 `term.rs` 抽出；并请用户先定库给谁用（只给 bingo、对外开源、学习探索）。

用户否掉这个方向：「不参考bingo做法，我们的目标是从零构建rs领域的ink，有些最佳实践可以参考Ink, 因为iocraft我觉得实现的太糙了。 关于这个想法你觉得应该如何实现呢」。

## 决策

- 从零构建 Rust 版 Ink，不沿用 bingo 的做法，行为以 Ink 8.0 为准。用户决定（见上一句引文）。
- 采用 Claude 第二次给出的实现方案：分层、React 式编程模型、单元格 diff 输出、以 Ink 测试集为规格、M0–M5 里程碑。用户回复：「可以 我觉得你的思路挺好的 先搭建一个骨架 然后把Ink的测试先搬过来吧」。
- 方案末尾的四个待定问题（异步、写法、名字、Windows）留到 [003](003-event-loop-name-platform.md) 定。

## 备选与取舍

- **Elm 式模型，从 bingo 抽代码**：Claude 第一次提出，用户否掉。
- **照 iocraft 的做法**：Claude 读 iocraft 0.9.3 源码，列出三处要避开的问题：
  - 拿可变引用就标脏，不比较新旧值（`use_state.rs:149-161`），渲染时写状态会不停重渲染；
  - 每次唤醒从根组件重渲染，不合并更新，不限帧率；
  - 非全屏模式下帧高到屏高时执行 `Clear(All)+Purge`（`backend/crossterm.rs:546`），清掉 scrollback。
- **宏 DSL 先行**：不选。builder 能被 rustfmt 格式化，有补全和类型检查；宏以后作为 builder 的语法糖。

## 方案

调研由主 agent 做：读 Ink 8.0 的模块和测试结构，读 iocraft 0.9.3 源码。方案要点：

- 分层：components/hooks → runtime → host tree → layout+text → paint → terminal，上层只依赖下层。见 [分层](../design.md#分层)。
- 编程模型：组件为 `fn(&mut Scope, &P) -> Element`，`P: PartialEq` 相当于默认 memo；debug 下检查 hooks 顺序；`State<T>` 是可复制句柄，`set` 要求 `PartialEq`；渲染中写状态在 debug 下 panic；一轮事件合并成一帧；定时器更新受 `max_fps`（默认 30）限制，输入不受限；空闲零唤醒；effect 在写终端之后执行。见 [编程模型](../design.md#编程模型)。
- `Box` 改名 `View`，避开 `std::boxed::Box`。
- 布局用 taffy 0.15；文本按字素切分；ANSI 在 Text 边界解析成带样式片段。
- 输出：绘制是纯函数，只有 diff 有状态；按单元格比较；行尾用 EL；每帧包 CSI ?2026；非全屏的可刷新区域最高等于屏高；声明式光标；非 TTY；退出、panic、挂起时恢复终端。crossterm 只用于输入。
- 测试分四层：`render_to_string`、`testing::render`、vt100、portable-pty。见 [测试](../design.md#测试)。
- 里程碑 M0–M5，见 [里程碑](../design.md#里程碑)。

## 结果

- 方案写成 `docs/design.md`，随骨架提交 `c23bb71`（见 [002](002-skeleton-and-m0-test-port.md)）。
- 四个待定问题在 [003](003-event-loop-name-platform.md) 定下；全局架构在 [005](005-global-architecture.md) 重写。
