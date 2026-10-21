# M4 的阶段划分与八项决策

- 日期：2026-10-20
- 状态：已实施
- 提交：无（各阶段的提交见 032–037 及后续记录）
- Brief：无。调研派给 4 个 Claude 子 agent（general-purpose），prompt 未存档为 brief，要点见方案

## 背景

resize 残影的讨论结束后，用户说「好的 没问题了 你接着下一个任务吧 接下来该做什么了」。按 [design.md 的里程碑表](../design.md#里程碑)，M4 包括动画、`use_box_metrics`、panic 时的错误界面、挂起恢复、tokio 驱动，以及移植 Ink 的示例，验收方式是逐个对照示例。

按「先调研、跟用户定方案、再派 Codex」的规矩，Claude 先派了 4 个调研 agent（15:21–15:22），调研期间用户问「M4做完 是不是其实就差不多移植完了」。Claude 答：M4 之后只剩屏幕阅读器模式（M5）和宏 DSL；按 `docs/ink-tests.md`，Ink 共 1646 个测试，已移植 1163 个，M4 专属文件 111 个，另有二十多个示例测试。

## 决策

15:47 Claude 给出五个阶段和八个待定问题，每条附推荐。用户回复「可以」，八条按推荐执行：

1. 动画节流：Ink 的每个动画 hook 受 `max_fps` 限速，runtime 需要节流窗口。app 每次调用时传入窗口，runtime 不保存。design.md 的「runtime 不接收节流参数」改为「runtime 不保存节流窗口」。
2. `use_box_metrics` 在第一帧给出真实值，Ink 第一帧为 0、第二帧才是真实值。记为与 Ink 的差异。
3. 错误界面的调用栈只在设置 `RUST_BACKTRACE` 时显示，没设置时省略。备选是 v1 不显示调用栈、放弃 2 个测试。
4. 卸载过程中、Transform 里、`testing::render` 里的 panic 不捕获，走现有恢复路径。同一步里 panic 覆盖之前的 `exit()`，不覆盖之前的 `exit_with_error`。
5. `suspend_on_ctrl_z` 默认开启，与 `exit_on_ctrl_c` 对应。Ink 把 Ctrl+Z 交给 `useInput`，记为差异。
6. 挂起 API 只提供闭包形式 `suspend_terminal(f)`：UI 线程不能阻塞等待自己的循环，句柄形式找不到执行恢复的线程。挂起期间新增的 Static 条目留到恢复后再画（Ink 丢弃）。
7. 新增公开函数 `ink_rs::text_width(&str)`：`use_cursor` 的使用者计算光标列时要与渲染器一致，例如 👩‍💻 在 unicode-width 下算 4，在渲染器里算 2。
8. jest 和 subprocess-output 两个示例用 tokio 驱动，其余用同步驱动。周期 tick 改用 `use_animation`，测试用假时钟复现。

## 方案

阶段与顺序：

- P1 动画与盒子度量，两个 Codex 并行（见 032、033）。
- P2 panic 错误界面（见 035）。
- P3 挂起与恢复，含 Ctrl+Z 与 SIGTSTP/SIGCONT（见 036）。
- P4 tokio 驱动与 `use_future`。
- P5 移植示例，分 2–3 批：第一批是不依赖 P2–P4 的示例（见 034）；PTY 用例（terminal-resize、suspend-terminal）放第二批；jest、subprocess-output 在 P4 之后。
- 顺序 P1 → P2 → P3 → P4。P1 合入后，P5 第一批与 P2 并行。

四份调研（报告在第一个会话 scratchpad 的 `m4/`）：

- `research-animation-metrics.md`：Ink 两个 hook 的语义；共享计时器放在 Store，截止时间并入 `render_if_due` 的唤醒时间；`TestInstance::settle` 要停在未来的截止时间、`advance` 要逐个截止时间推进；`NodeRef` 改为 `Option<布局>`；useAnimation 可移植 38/45，useBoxMetrics 16/19；11 个待定问题。
- `research-errors.md`：Ink 的 ErrorBoundary 捕获 render 与 effect 的错误，不捕获输入回调、Transform 与 teardown。阻塞点：驱动的 panic hook 在展开前恢复终端，之后的写入被丢弃，std 的 hook 也会打印在帧上。解法是线程局部标记，对会被捕获的 panic 跳过恢复与打印。
- `research-suspend.md`：Ink `suspendTerminal` 的拆卸与恢复顺序、挂起期间的行为；Ink 没有 SIGTSTP/SIGCONT 处理，这部分是 ink-rs 自己的设计；35 个测试；10 个待定问题。
- `research-examples-tokio.md`：29 个示例目录，20 个可用现有 API 表达，3 个需要 M4 功能，4 个属 React 专属（suspense、concurrent-suspense、use-transition、router）；27 个示例测试中 22 个可在进程内运行，5 个需要 PTY；tokio 驱动与 `use_future` 的草图。

## 结果

- P1–P3 与「最终帧完整写出」在本日 17:42 前完成，见 032–037。
- P4 的方案在 17:39 提出，用户 17:47 回复「tokio按照你推荐的来」，派发与实施在后续记录。
- P5 第二、三批在后续记录。
