# 挂起终端、Ctrl+Z 与 SIGTSTP/SIGCONT

- 日期：2026-10-20
- 状态：已实施
- 提交：`dca3f7f` test(pty): extract a shared PTY session；`0631879` feat(runtime): suspend the terminal for a callback；`b596163` feat(driver): stop on Ctrl+Z and SIGTSTP；`ed69515` docs: record terminal suspension and Ctrl+Z
- Brief：[m4-p3](briefs/m4-p3.md)

## 背景

M4 计划的 P3（见 031）。Ink 有 `suspendTerminal`：把终端交给回调（例如运行编辑器），回调结束后恢复并重画。Ink 没有作业控制；Node 的 raw mode 把 Ctrl+Z 当普通按键。design.md 原承诺「SIGTSTP/SIGCONT 在 M4 接入」。

## 决策

- 用户批准 M4 计划时（「可以」）同意：`suspend_on_ctrl_z` 默认开启；只提供闭包形式 `suspend_terminal(f)`；挂起期间新增的 Static 条目留到恢复后再画。
- Claude 写 brief 时定下以下各项，并说明「P3 的关键决定都在已批准的 M4 计划内」，派发后向用户汇报，没有另行请示。与 `research-suspend.md` 不同处以 brief 为准：
  - app 卸载后调用 `suspend_terminal`：丢弃 `f`、不调用、返回 `Ok`，同 `update`（调研建议仍运行 `f`）。已有排队请求或正在挂起时返回 `Err(AlreadySuspended)`。
  - 两个挂起原因 `Api` 与 `Stop`，任一存在即为挂起。
  - 输入在所有模式下交出（raw、粘贴、kitty flags），子进程因此能读 stdin；Ink 在非交互模式不交出，没有测试覆盖这一差异。屏幕部分只在交互模式交出。
  - 挂起期间 commit 与 effect 照常运行，交互模式不绘制，唤醒时间不能因待绘制帧而空转；输入丢弃，resize 只更新尺寸，交互模式丢弃外部写入。
  - Ctrl+Z 按下或重复时向进程组发 SIGTSTP（rustix `kill_current_process_group`，加 `process` feature，无 unsafe），事件被消费；释放事件交给 `use_input`。
  - SIGTSTP 只由信号线程处理，不注册条件默认处理（调研建议注册）。信号线程在注册表锁下给各个 app 发 `Stop`，等所有 app 交还终端后再停止进程。
  - 孤儿组保护：进程组是会话首进程组时忽略 SIGTSTP，否则 SIGSTOP 之后没有 shell 能让它继续。
  - SIGCONT 只重画因 `Stop` 挂起的 app；外部 SIGSTOP 后的 SIGCONT 不重画，避免 inline 帧重复（调研建议所有 app 都重画）。
  - 同步驱动在 UI 线程上运行 `f`；运行前先取出通道中的 `Stop` 并应答，避免信号线程等到 `f` 返回。

## 方案

- 派给一个 Codex。派发前 Claude 在 PTY 中实测作业控制：`bash --norc --noprofile -i -c '"$0"; fg'` 可用，调研建议的非交互 `bash -c 'set -m; …'` 会挂住。
- 分三个提交以便审阅：先把 PTY 会话代码抽到 `tests/common/pty.rs`（不放进 `tests/common/mod.rs`，其他测试 crate 不需要 portable-pty）；再提交挂起 API；最后提交 Ctrl+Z 与信号。
- 测试：按 Ink 文件逐个移植 `suspend-terminal`、`suspension-exit`、`suspension-input-disable`、`suspension-output`、`suspension-resize` 与 `kitty-negotiation` 的两个挂起用例；`src/app/tests.rs` 加 2 个 Ctrl+Z 测试；PTY 新增子进程交接、`bash -i` 下的作业控制、孤儿组保护 3 个用例。`suspension-handle.tsx` 两个用例不适用（没有句柄对象）。

## 结果

- Codex 报告没有偏离 brief；Linux 上的编译、PTY 与作业控制没有实测，多应用与信号竞争没有压力测试。
- Claude review 未发现问题。集成时按 Codex 给出的分块清单拆成三个提交，每步单独验证；最终全量测试、clippy 与示例构建通过。
- 在 tmux 中手动验证 Ctrl+Z → `fg`，inline 与备用屏两种模式都正常。
- 文档提交 `ed69515`：[design.md 的终端输出一节](../design.md#终端输出)写入挂起与 Ctrl+Z 的规则及与 Ink 的差异；`docs/ink-tests.md` 更新计数（如 `suspend-terminal.tsx` 9/14），加「挂起的观测边界」。初稿中按印象写的 Ink 测试名改成从 Ink 源码查到的原名。
- 后续：terminal-resize 与 suspend-terminal 示例的 PTY 用例在 P5 第二批；tokio 驱动中 `f` 的运行方式在 P4 决定。
