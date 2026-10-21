# M4 P5 示例第 2、3 批：terminal-resize、suspend-terminal、jest、subprocess-output

- 日期：2026-10-20
- 状态：已实施
- 提交：`65315e0` feat(examples): port terminal-resize and suspend-terminal；`8a1b006` feat(examples): port jest and subprocess-output on tokio
- Brief：[m4-p5b2](briefs/m4-p5b2.md)、[m4-p5b3](briefs/m4-p5b3.md)

## 背景

M4 计划的 P5 是移植 Ink 示例。第 1 批在本时段之前完成。据会话摘要，第 2 批（terminal-resize、suspend-terminal，带 PTY 测试）和第 3 批（jest、subprocess-output 跑在 tokio 上）在 M4 计划中获批，"jest 和 subprocess-output 用 tokio"也列在已批准的决策里。用户的常设指示是「好的 你来协调整体进程吧」。

## 决策

- 两批串行派发：两批都改 `tests/examples_pty.rs` 和 `Cargo.toml`，第 2 批合入后再派第 3 批。Claude 决定。
- subprocess-output 的组件只接收字节通道（`UnboundedReceiver<Chunk>`），不启动子进程；`main` 是生产用的喂数据方，测试是本地喂数据方。Claude 决定，brief 写明"组件依赖接口，不依赖进程"；Ink 在测试里替换 `spawn`，这里改为替换通道的喂数据方。
- jest 用 `use_future` 里 `tokio::join!` 4 个 worker 跑 10 个用例，abort 任务即取消全部 worker；随机数用本地 xorshift，不加 `rand`。
- 两个 tokio 示例声明 `required-features = ["tokio"]`，`--no-default-features` 构建时跳过。
- 挂起和终端尺寸需要真实 TTY，Ink 也经终端测试，所以第 2 批只写 PTY 测试。cell diff 只重写变化的格子，帧内容在 vt100 屏幕上判断，不搜原始字节。

## 方案

- 第 2 批，Codex，worktree `wt/m4-p5b2`：
  - 两个示例逐行翻译 Ink，丢掉 `patchConsole`；suspend-terminal 的子进程结果经 `AppHandle::update` 回到 UI 线程。
  - 新 PTY 测试文件 `tests/examples_pty.rs`（`harness = false`），`Session` 加 `resize`。
  - 4 个用例对应 Ink `render.tsx` 的 terminal resize 用例和 3 个 `EDITOR` 取值的 suspend 用例。
- 第 3 批，Codex，worktree `wt/m4-p5b3`：
  - jest 按 Ink 的 `PQueue` 并发度 4，`Static` 按路径作 key，`ms` 短格式本地实现。
  - subprocess-output 跨块 UTF-8 解码用 `utf8_chunks`，保留最后 5 行，渲染时剥离 ANSI。
  - 测试：4 个 subprocess 进程内用例（`start_paused`，非交互）；1 个 jest PTY 用例，按 `\x1b[?2026l` 切帧重放到 vt100，`Ran all test suites.` 不早于 `10 total`。
- 第 3 批派发期间，Claude 自己调研了第 3 批方案，没有派调研 agent。

## 结果

- 第 2 批：Codex 无偏差。全量测试通过，PTY 用例 23 → 27；两种 feature 配置的 clippy 通过。`docs/ink-tests.md` 中 `render.tsx` 移植数 52 → 56。
- 第 3 批：Codex 让解码器在通道关闭时把不完整的尾部输出为 U+FFFD。集成时 Claude 改了一处：输出末尾未结束的转义序列原先会留在画面上（可能闪出 `[3`），改为先隐藏，等下一块补全后再剥离。
- 第 3 批集成后全量 1311 个测试通过，PTY 28 个通过；Claude 在 tmux 里手动跑两个示例，退出码为 0。`render.tsx` 移植数 56 → 61，`docs/design.md` 与 `docs/ink-tests.md` 记录两个示例的结构和测法。
- M4 计划项至此完成。
