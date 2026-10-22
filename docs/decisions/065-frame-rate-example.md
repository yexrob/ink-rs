# 帧率示例 `frame-rate`

- 日期：2026-10-22
- 状态：已实施
- 提交：`a329d34` feat(examples): add a frame-rate example
- Brief：无，主 agent 自己做

## 背景

验收 0.1.4（[064](064-release-0-1-4.md)）时，Claude 给了 scratchpad 里的测帧程序：参数是 `<max_fps> <interval_ms>`，跑 3 秒，统计写进文件。用户说「example 也提交一下吧」。Claude 问指哪个，用户选「两个都要」：ink-rs 加帧率示例，随 0.1.4 发布；eink-rs 合入 Markdown demo（eink-rs 的记录 016）。

## 决策

- 用户定（「两个都要」）：ink-rs 加帧率示例，随 0.1.4 发布。
- 以下由 Claude 按同一授权定：
  - 单文件 `examples/frame-rate.rs`，不加测试，同 `counter.rs`。示例是 ink-rs 自有的，不是移植：文件开头写明，README 和 design.md 的示例一节各补一句。
  - 画面：一条按时间增长的进度条，用 1/8 格的方块字符，让 120 Hz 下每帧的位移看得见；下面是最近 1 秒的实测帧率和帧间隔的 p50、p90。
  - 参数：`max_fps` 取命令行的第一个参数，默认 120。interval 默认取 `1000 / max_fps` 向下取整的毫秒数（120 → 8ms，60 → 16ms），这是贴着窗口的常见写法，也是 063 之前帧率减半的配置。↑/↓ 在运行中改 interval，`q` 退出。
  - 统计在以 `frame` 为键的 `use_effect` 里每交付一帧记一次 `delta`，进度条由 `time` 推出。按 design.md 的约定在注释里写明。
  - 不显示系统晚醒的对照（测帧程序里的 `recv_timeout`）：那是诊断用的，不属于示例。

## 备选与取舍

- **把测帧程序不加改动放进 examples**：它不画东西，结果写进文件，不像示例。
- **写成测试，断言帧率**：真实时钟下帧率随机器负载波动，CI 上不稳。063 有假时钟下的单元测试。
- **运行中切换 `max_fps`**：`max_fps` 是 `Options` 的字段，渲染开始后改不了，要改得重启；用参数传入。

## 方案

主 agent 自己做：

1. `examples/frame-rate.rs`；`Cargo.toml` 注册，`required-features = ["macros"]`。
2. README 的 Examples 一节、design.md 的示例一节。
3. fmt、clippy、全量测试；在 tmux 会话 `ink-rs` 里跑 120、60、30 三档给用户看。

## 结果

- `a329d34`：`examples/frame-rate.rs`，`Cargo.toml` 注册，README 和 design.md 各补一句。
- `cargo test --all-features` 1331 个通过，`--no-default-features` 1297 个通过；clippy、fmt 通过；`cargo publish --dry-run` 打包 198 个文件，含这个示例。
- tmux 里三档同时跑（全量测试同时在跑，CPU 有负载）：

| `max_fps` | interval | 实测 | 帧间隔 p50 / p90 |
|---|---|---|---|
| 120 | 8ms | 120.0 fps | 8.32 / 8.56 ms |
| 60 | 16ms | 60.1 fps | 16.63 / 17.19 ms |
| 30 | 33ms | 30.0 fps | 33.63 / 36.57 ms |
| 120 | 9ms（↑ 调到） | 111.2 fps | 9.15 / 9.55 ms |
| 60 | 9ms（↓ 调到） | 60.0 fps | 16.34 / 19.04 ms |

  interval 超过窗口时按 interval 的节拍走，不超过时跑满 `max_fps`。
