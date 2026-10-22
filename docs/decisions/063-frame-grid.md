# 帧率：窗口不取整，晚醒不降速

- 日期：2026-10-22
- 状态：已实施
- 提交：`762686d` feat(app): keep max_fps on a fixed frame grid
- Brief：无，主 agent 自己做

## 背景

eink-rs 做 Markdown 流式渲染的 demo（eink-rs 的记录 016），用户问能否支持 120Hz/60Hz。demo 的状态栏显示 `max_fps: 120` 只有 89 fps，用户问「这个是终端的限制还是ink-rs的限制」。

Claude 在 tmux 里测了 tick 间隔（每组 3 秒，macOS）：

| 配置 | 间隔 p50 | fps |
|---|---|---|
| `max_fps` 120，interval 1ms | 11.18ms | 89 |
| `max_fps` 120，interval 9ms | 17.87ms | 55 |
| `max_fps` 60，interval 1ms | 20.81ms | 49 |
| `max_fps` 60，interval 17ms | 33.86ms | 30 |

不经过 ink-rs 的 `recv_timeout(9ms)` 中位数晚醒 1.89ms，`recv_timeout(17ms)` 晚醒 2.76ms。

原因有三处，都在 ink-rs，终端不在测量路径上：

1. 窗口是 `ceil(1000 / max_fps)` 毫秒。120 取 9ms，上限 111 fps。默认 30 取 34ms，比 30fps 动画的 33.3ms 长，记录 033 遗留的「30fps 动画每两格交付一次」就是它。
2. 下一帧的截止时间从实际绘制的时刻算（`now + window`），动画的门槛从上次交付的时刻算（`last_render + throttle`）。系统每次晚醒的时间都累积成速率损失。
3. 动画的 tick 落在 interval 格点上，被门槛挡住的 tick 要等下一格。interval 等于窗口时，晚醒让一半的 tick 落进窗口，帧率减半。

Claude 提出两处改法，说明代价是偏离 Ink 的节流语义。用户答「可以 你再ink-rs里面修改一下吧 这是ink-rs一个很好的特性  支持高帧率」。

## 决策

- 用户定（「可以 你再ink-rs里面修改一下吧」）：
  - 窗口按 `1s / max_fps` 计算，不取整。120 是 8.33ms，60 是 16.67ms，默认 30 是 33.33ms。
  - 帧按固定节拍排：晚醒不降低平均帧率。
- 以下由 Claude 按同一授权定：
  - 绘制：一帧比截止时间晚不到半个窗口，记在原来的格子上，下一个截止时间是「原截止时间 + 窗口」；晚半个窗口以上，从这一刻重新起格。
  - 动画：每个条目记一个格点锚，替代 `last_render` 的门槛作用（`last_render` 只留给 `delta`）。交付时，取这次 tick 所在的 interval 格点，比门槛晚不到半个窗口时，锚前移一个窗口；否则锚取这个格点。格点不含晚醒，interval 大于窗口的动画不会因为锚漂移丢 tick。
  - 截止时间的公式不变，`last_render + throttle` 换成「锚 + 窗口」。（实施中改了，见下一条。）
  - 实施中补的一条：被窗口挡住的 tick 保持到期，窗口一开就交付，截止时间是 `max(next_due, 锚 + 窗口)`，不再取下一个 interval 格点。复测时 `max_fps` 30 配 33ms 只有 15 fps：tick 比窗口早 0.33ms 被挡住，要等下一格。60 fps 配 16ms 是常见写法，同样会减半。Ink 也这样丢 tick。
  - `max_fps = 0` 仍是 0 窗口，行为不变。
  - runtime 仍不保存窗口，由 app 每步传入。
  - 不发布。推送和发布 0.1.4 等用户批准。

## 备选与取舍

- **不改 ink-rs，让应用把 interval 设成 1ms**：demo 用的就是这个办法，晚醒仍累积，120 只到 89 fps。
- **晚醒多少都记在原格子上**：空闲后的第一帧可能紧贴下一格，两帧间隔接近 0。半个窗口的门限把最短间隔限在半个窗口。
- **动画与绘制共用一个门槛（app 的截止时间）**：两个格子天然对齐。但非交互模式不绘制、没有截止时间，动画会失去节流；`use_animation` 的 `non_interactive` 测试（`max_fps` 120 下 8ms 动画在 25ms 时为 2）会变。
- **提前醒来再自旋等到截止时间**：能消掉每帧约 2ms 的抖动，代价是 CPU。不做。

## 方案

主 agent 自己做：

1. `src/app.rs`：窗口改为 `1s / max_fps`；绘制后的截止时间按上面的规则；`max_fps` 的文档。
2. `src/runtime/animation.rs`：`Entry` 加锚，`tick_animations` 与 `animation_deadline` 改用锚。
3. 测试：现有的假时钟不会晚醒，覆盖不到新行为，加一个 runtime 单元测试，用晚 2ms 的步进跑 1 秒，断言 120 档交付约 120 次。`animation_deadline_is_the_minimum…` 的断言依赖 `last_render`，按新语义改。
4. `docs/design.md` 的调度与动画两段；记录 033 的遗留补上「见 063」。
5. 全量 `cargo test --all-features`、clippy、fmt；再用 eink-rs 的 demo 和测量程序在 tmux 里复测。

## 结果

- `762686d`：
  - `src/app.rs`：窗口改为 `1s / max_fps`，绘制后的截止时间用 `keep_slot`。
  - `src/runtime/animation.rs`：`Entry` 加了 `slot`；挡住的 tick 保持到期；截止时间去掉格点取整。
  - 新测试 `app::tests::late_wakes_keep_the_frame_rate`：直接驱动 `App`，每次唤醒晚 2ms，跑 1 秒，`max_fps` 120 下 interval 1ms、8ms 各约 120 帧，9ms 约 111 帧。改动前 1ms 只有 92 帧。
  - 改了两处断言，都按新语义：
    - runtime 单元测试的截止时间：150ms 改为 101ms，320ms 改为 261ms。
    - 移植自 Ink 的 `animations_advance_in_debug_mode_when_interactive_is_false`：`"2"` 改为 `"3"`。Ink 原测试断言的是下界，仍成立。
- `cargo test --all-features` 1331 个通过；`--no-default-features`、clippy、fmt 通过。
- tmux 里复测（每组 3 秒，macOS）：

| 配置 | 改前 | 改后 |
|---|---|---|
| `max_fps` 120，interval 1ms | 89 | 119 |
| `max_fps` 120，interval 8ms | — | 120 |
| `max_fps` 120，interval 9ms | 55 | 111 |
| `max_fps` 60，interval 16ms | — | 60 |
| `max_fps` 60，interval 17ms | 30 | 59 |
| `max_fps` 30，interval 33ms | — | 30 |

  单帧仍有约 2ms 的抖动（p10 到 p90），平均帧率达标。eink-rs 的 Markdown demo 用本地 ink-rs 构建（命令行 patch，不改文件），状态栏 121–123 fps。
- 未推送，未发布。eink-rs 要等 ink-rs 发布 0.1.4 后升级依赖；demo 里 1ms interval 的注释到时改写。
