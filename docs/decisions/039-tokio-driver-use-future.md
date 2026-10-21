# M4 P4：tokio 驱动与 `use_future`

- 日期：2026-10-20
- 状态：已实施
- 提交：`664b637` refactor(driver): share the event loop between drivers；`2a30dfa` feat(hooks): add use_future；`5defbce` feat(driver): add a tokio driver；`0e1b04c` docs: record the tokio driver and use_future
- Brief：[m4-p4](briefs/m4-p4.md)

## 背景

M4 计划里的 P4 是 tokio 驱动和 `use_future`。design.md 定过 feature `tokio` 默认开启。Claude 在 P4 派发前列出方案（公开 API、两个驱动共用每一步、消息通道抽成 trait、测试范围），只留一点给用户定：tokio 驱动里 `suspend_terminal(f)` 的 `f` 用 `spawn_blocking` 跑，还是在 UI 线程上调用。用户答「tokio按照你推荐的来」。

## 决策

- `f` 交给 `spawn_blocking(f).await`，交出终端期间其他 tokio 任务继续运行；`f` panic 时在 UI 线程 `resume_unwind`。用户决定（「tokio按照你推荐的来」）。
- 以下由 Claude 在 brief 中定，属于用户批准的方案：
  - 公开 `ink_rs::tokio::{render, Instance}`，`wait_until_exit` 为 async，须在 `LocalSet` 或 local runtime 里运行。
  - 两个驱动只在"如何等待"和"如何运行挂起回调"上不同；循环的每一步抽成共用的 `Looper::turn`。
  - 消息通道抽成 `Post` trait，std mpsc 与 tokio unbounded channel 各实现一份。
  - tokio 应用的时钟全部取 tokio 的 `Instant`，`start_paused` 测试不用等真实时间。
  - 收到终止信号时 tokio 版 await `pending()`，让同线程其他应用完成清理。
  - 同线程多个 tokio 应用并发，不算嵌套；同步 wait 会阻塞同线程全部 tokio 应用。
  - `use_future` 建在 `use_effect` 上，deps 变化或卸载时 abort；任务持有 `AppHandle`，运行期间应用不自然结束。
  - 任务里的 panic 按组件 panic 处理：显示错误界面，返回 `Err(Error::Panic)`。原因：任务在所有 runtime guard 之外轮询，原有的进程 panic hook 会先恢复终端，tokio 再吞掉 panic，应用继续运行而终端已恢复。

结果见 [事件循环](../design.md#事件循环) 的 tokio 驱动与 `use_future` 两条。

## 方案

- 派 Codex，worktree `wt/m4-p4`，与 [038](038-write-control.md) 并行。背景调研沿用第一会话 `m4/research-examples-tokio.md` §4，brief 写明以 brief 为准。
- Brief 要点：
  - Cargo：tokio 1.54（rt、sync、time），本地缓存离线构建。
  - 驱动：`Post`、共用的 mount 与 `Instance` 核心、`Looper`；同步行为不变，`src/driver/tests.rs` 只随签名改。
  - `use_future` 的轮询放进 `runtime::panic::guard`，panic 交给 runtime 由下一步报告，存放位置由 worker 选并说明。
  - 测试：`tests/use_future.rs` 4 个进程内用例；`tests/pty.rs` 2 个 tokio 用例（任务结束后应用自然结束；挂起回调运行时其他任务继续）。
  - 分三阶段，每阶段打 `.stages/<n>.tar` 快照，主 agent 据此重建三个提交。
- 验收：两种 feature 配置的 build 与 clippy、`cargo test --lib`、指定的 7 个测试文件。

## 结果

- Codex 把首个 panic 槽从 `Runtime` 移到 `Store`，`Guarded` 持 `Weak<Store>`，在 guard 内轮询并 `notify` 队列。偏差：tokio 的 `Post` 实现落在第 3 阶段，因为 tokio 依赖在第 2 阶段才加入。
- Claude 审查：`Looper::turn` 与原 `run_loop` 逐行对应，注册规则符合 brief。按快照重建三个提交，逐个检查中间提交能构建、clippy 通过。
- 全量 1307 个测试通过；`--no-default-features` 下 1302 个通过；tokio PTY 用例通过。
- 文档提交 `0e1b04c` 改写 tokio 驱动条目，新增 `use_future` 条目，记录通道、挂起和 panic 的处理。
