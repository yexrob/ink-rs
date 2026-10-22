# `use_animation` 与共享计时器

- 日期：2026-10-20
- 状态：已实施
- 提交：`8d21372` feat(runtime): add use_animation
- Brief：[m4-p1a](briefs/m4-p1a.md)

## 背景

M4 计划的 P1（见 031）。Ink 的 `useAnimation` 让所有动画共用 App 里的一个定时器，受 `max_fps` 节流。ink-rs 的核心不读时钟，驱动传入 `now`；[design.md](../design.md#事件循环) 原写「runtime 不接收节流参数」，与每个动画受节流的语义冲突。

## 决策

- 用户批准 M4 计划时（「可以」）同意：节流窗口由 app 每步传入，runtime 不保存；design.md 改为「runtime 不保存节流窗口」。
- Claude 在 brief 中定下实现（属 M4 计划内）：
  - 调度表放在 Store（`RefCell<AnimationState>`），与输入的 Esc 截止时间并列。不加新的 hook `Kind`，hook 由 `use_state` 与 `use_layout_effect` 组成。
  - `tick_animations(now, throttle)` 按 Ink 的追赶公式前移到期条目，迟到的唤醒跳过错过的格点；在节流窗口内的条目不写状态。
  - `animation_deadline(throttle)` 取各条目第一个不早于 `max(next_due, last_render + throttle)` 的格点的最小值，没有条目时为 `None`，空闲零唤醒的保证不变。
  - tick 经 `State::set` 写入，每次 tick 多一次空的驱动循环，同输入；不加「静默写入」路径。
  - 不在渲染时清零：ink-rs 的 layout effect 写状态会在 paint 前重渲染，零值进入同一帧。
  - `TestInstance::settle` 停在未来的截止时间；`advance` 逐个截止时间推进，每次 tick 有自己的 `delta`。
  - 同步驱动不变：活动的动画给出唤醒时间，`natural_end` 不成立，app 不会自然结束，同 Ink 被 ref 的定时器。

## 方案

- 派给一个 Codex，与 P1b（见 032）并行。Store 字段放在 `cursor` 之后，hook 放在 `use_window_size` 之后。
- API：`AnimationOptions { interval, active }`（默认 100ms、true），`Animation { frame, time, delta, reset() }`。间隔夹在 1ms..=2_147_483_647ms，同 Ink 对 `setTimeout` 的夹取。
- 测试：
  - `tests/use_animation.rs` 移植 38/45。7 个不适用：NaN、Infinity、负 Infinity、负间隔（`Duration` 表达不了）；墙钟回拨（核心不读时钟）；两个并发与 transition 用例。
  - `tests/pty.rs` 加 `render.tsx` 的两个管道 fixture：动画驱动非交互进程退出。
  - runtime 单元测试 1 个，覆盖 design.md 承诺的「空闲零唤醒」：`animation_deadline` 取最小值，无活动动画时为 `None`。
- 只跑指定测试目标与 clippy。

## 结果

- Codex 的报告：38 个测试与两个 fixture 通过。断言调整：Ink 的 `>=` 下界改为精确值；定时器调用次数的断言省略，由 runtime 单元测试覆盖共享唤醒；`debug: true` 的用例改用 `testing::render`（零节流）。
- review 发现一处文档注释写了 "Debug renders"，在 ink-rs 里没有对应概念，修改后集成。应用 P1a 补丁时与 P1b 在 `src/lib.rs` 的导出行冲突，手工合并。
- 全量 1222 个测试加 15 个 PTY 用例通过，clippy 无警告。
- 文档：`docs/ink-tests.md` 计数改为 38，`render.tsx` 的两个动画用例移出待办；[design.md](../design.md#事件循环) 加「动画」一条，节流一句改为「runtime 不保存节流窗口」，测试驱动与自然结束两段补上动画规则。
- 遗留：默认 `max_fps` 30 的窗口按 Ink 规则向上取整为 34ms，比 30fps 动画的 33.3ms 间隔长，30fps 动画实际每两格交付一次。这是在仓库外的 demo 中观察到的，列为待办，本区间内没有决定改动。后续：[063](063-frame-grid.md) 改为不取整的窗口和固定节拍。
