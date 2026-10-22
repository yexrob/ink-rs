# 决策记录

每次写代码或派 worker 之前，先在这里记下依据：做了什么决定、谁定的、比较过哪些方案、工作怎么拆。[design.md](../design.md) 写设计现在的样子，这里写它怎么来的。

项目开始到 0.1.1 的记录是按会话记录补写的。补写时找不到依据的地方，记录里写明「记录中未见」。

## 格式

文件名 `NNN-slug.md`，编号按时间顺序。

```markdown
# 标题

- 日期：2026-10-21
- 状态：实施中 | 已实施 | 部分实施 | 已延后 | 被 NNN 取代
- 提交：`abc1234` feat: …（没有写「无」）
- Brief：[name](briefs/name.md)（没有写「无」）

## 背景
为什么要做决定。用户的原话用「」引。

## 决策
逐条写，每条注明谁定的：用户（引原话），或 Claude 按用户的授权（引授权的话）。

## 备选与取舍
比较过的方案和放弃的原因。没有就省略。

## 方案
工作怎么拆：派给谁（Codex、Claude 子 agent、主 agent 自己），brief 的要点，验收命令和测试范围，调研问了什么、结论是什么。

## 结果
提交、review 发现的问题和处理、偏离方案的地方、后续由哪条记录接手。
```

背景和决策必填，其他小节空着就省略。写之前开一条记录，状态填「实施中」，和改动一起提交或先于改动提交；工作落地后补结果和提交。

## Brief

`briefs/` 存派给 worker 的原文。本机路径换成 `<repo>`（仓库根）、`<scratchpad>`（会话的临时目录）和 `~`。brief 引用的 `<scratchpad>` 下的文件（Ink 源码克隆、采集的期望值、调研笔记、worker 报告）没有入库，记录里写了它们的结论。

## 索引

| 编号 | 日期 | 标题 | 状态 |
|---|---|---|---|
| [001](001-goal-and-approach.md) | 2026-10-19 | 项目目标与总体方案 | 已实施 |
| [002](002-skeleton-and-m0-test-port.md) | 2026-10-19 至 2026-10-20 | 骨架与 M0 测试移植 | 已实施 |
| [003](003-event-loop-name-platform.md) | 2026-10-20 | 事件循环、写法、名字与平台 | 已实施 |
| [004](004-component-scope.md) | 2026-10-20 | 核心组件范围 | 已实施 |
| [005](005-global-architecture.md) | 2026-10-20 | 全局架构与 M0 的 11 项决策 | 已实施 |
| [006](006-small-step-commits.md) | 2026-10-20 | 允许 agent 小步提交 | 已实施 |
| [007](007-m0-text-layer.md) | 2026-10-20 | M0 第一阶段：文本层 | 已实施 |
| [008](008-m0-pipeline.md) | 2026-10-20 | M0 第二阶段：host 树、布局、绘制与移植偏差修正 | 已实施 |
| [009](009-m0-fix1-and-yoga-gap.md) | 2026-10-20 | M0 第三阶段第一批修复与 Yoga 差异的取舍 | 已实施 |
| [010](010-m0-own-host-tree.md) | 2026-10-20 | 自有 host 树，模拟 Yoga 的 flex basis 与 aspect ratio | 已实施 |
| [011](011-m1-runtime-plan.md) | 2026-10-20 | M1 运行时方案 | 已实施 |
| [012](012-m1-p1-runtime-core.md) | 2026-10-20 | M1 P1：组件、hooks 与带 key 的协调 | 已实施 |
| [013](013-m2-output-plan.md) | 2026-10-20 | M2 终端输出方案与并行调研 | 已实施 |
| [014](014-m1-p2-stepping-core.md) | 2026-10-20 | M1 P2：步进核心与测试驱动 | 已实施 |
| [015](015-m2-p1-output-layer.md) | 2026-10-20 | M2 P1：内联的单元格差分输出层 | 已实施 |
| [016](016-m1-p3-node-refs.md) | 2026-10-20 | M1 P3：节点引用与元素测量 | 已实施 |
| [017](017-m2-p2a-static-across-frames.md) | 2026-10-20 | M2 P2 拆分与 P2a：Static 跨帧只输出一次 | 已实施 |
| [018](018-m2-p2b-app-layer.md) | 2026-10-20 | M2 P2b：带节流的应用层与测试驱动 | 已实施 |
| [019](019-m2-p3a-cursor-external-writes.md) | 2026-10-20 | M2 P3a：`use_cursor` 与外部写出 | 已实施 |
| [020](020-m2-p3b-resize.md) | 2026-10-20 | M2 P3b：resize 与 `use_window_size` | 已实施 |
| [021](021-m2-p2c-sync-driver-decisions.md) | 2026-10-20 | M2 P2c：同步终端驱动 | 已实施 |
| [022](022-reflow-erase.md) | 2026-10-20 | 宽度变小时按重排行数擦除 | 已实施 |
| [023](023-m3-input-architecture.md) | 2026-10-20 | M3 调研与输入架构 | 已实施 |
| [024](024-m3-input-parser.md) | 2026-10-20 | M3 P1：移植 Ink 输入解析器 | 已实施 |
| [025](025-m3-exit-api.md) | 2026-10-20 | M3 P2：退出 API 与退出值 | 已实施 |
| [026](026-m3-use-input.md) | 2026-10-20 | M3 P3a：输入派发与 use_input | 已实施 |
| [027](027-m3-paste-kitty.md) | 2026-10-20 | M3 P3b：use_paste 与 kitty 键盘协商 | 已实施 |
| [028](028-m3-focus.md) | 2026-10-20 | M3 P4：use_focus 与 use_focus_manager | 已实施 |
| [029](029-m3-production-driver.md) | 2026-10-20 | M3 P5：rustix termios、stdin 读线程与 PTY 测试 | 已实施 |
| [030](030-resize-residue-app-level.md) | 2026-10-20 | resize 残影：清屏重放留给应用 | 已实施 |
| [031](031-m4-plan.md) | 2026-10-20 | M4 的阶段划分与八项决策 | 已实施 |
| [032](032-use-box-metrics.md) | 2026-10-20 | `use_box_metrics` | 已实施 |
| [033](033-use-animation.md) | 2026-10-20 | `use_animation` 与共享计时器 | 已实施 |
| [034](034-examples-batch1.md) | 2026-10-20 | 示例第一批与 `text_width` | 已实施 |
| [035](035-panic-error-overview.md) | 2026-10-20 | 组件 panic 的捕获与错误界面 | 已实施 |
| [036](036-suspend-terminal.md) | 2026-10-20 | 挂起终端、Ctrl+Z 与 SIGTSTP/SIGCONT | 已实施 |
| [037](037-final-frame.md) | 2026-10-20 | 卸载时完整写出被裁的最终帧 | 已实施 |
| [038](038-write-control.md) | 2026-10-20 | 带外控制写入 `write_control` | 已实施 |
| [039](039-tokio-driver-use-future.md) | 2026-10-20 | M4 P4：tokio 驱动与 `use_future` | 已实施 |
| [040](040-m4-p5-examples.md) | 2026-10-20 | M4 P5 示例第 2、3 批：terminal-resize、suspend-terminal、jest、subprocess-output | 已实施 |
| [041](041-rsx-macro.md) | 2026-10-20 | M5：宏 DSL `rsx!` | 已实施 |
| [042](042-defer-screen-reader.md) | 2026-10-20 | 屏幕阅读器模式延后 | 已延后 |
| [043](043-release-prep.md) | 2026-10-20 | 发布准备：许可证、仓库、README 与包元数据 | 已实施 |
| [044](044-examples-rsx.md) | 2026-10-20 | 示例改写为 `rsx!` | 已实施 |
| [045](045-external-write-newline.md) | 2026-10-20 | 外部写入以换行加转义序列结尾时多出空行 | 已实施 |
| [046](046-nfc-wrap.md) | 2026-10-20 | 折行时做 NFC 归一化 | 已实施 |
| [047](047-color-reset.md) | 2026-10-20 | `Color::Reset`：对应 Ink 的 `backgroundColor=""` | 已实施 |
| [048](048-resize-scrollback-erase.md) | 2026-10-20 | 屏幕增高时擦除从 scrollback 拉回的旧帧行 | 已实施 |
| [049](049-width-tables-unicode-18.md) | 2026-10-20 | 宽度表生成脚本，改用 Unicode 18.0，去掉 unicode-width | 已实施 |
| [050](050-release-0-1-0.md) | 2026-10-20 | 发布 0.1.0 | 已实施 |
| [051](051-windows-plan-sys-split.md) | 2026-10-21 | Windows 支持方案与 driver 的 sys 拆分 | 已实施 |
| [052](052-unsafe-exemption-rule.md) | 2026-10-21 | 把 Windows console FFI 的 unsafe 豁免写进 CLAUDE.md | 已实施 |
| [053](053-windows-driver.md) | 2026-10-21 | Windows 驱动 `sys/windows.rs` | 已实施 |
| [054](054-conpty-tests-ci.md) | 2026-10-21 | ConPTY 下的测试、三平台 CI 与 #969 实测 | 已实施 |
| [055](055-release-0-1-1.md) | 2026-10-21 | 发布 0.1.1 | 已实施 |
| [056](056-upper-framework.md) | 2026-10-21 | 上层框架：新仓库，参照 Pastel 与 Ink UI | 部分实施（原语见 057，其余待做） |
| [057](057-runtime-primitives.md) | 2026-10-21 | 运行时原语：context、`Element: PartialEq`、`Callback` 与 `use_callback` | 已实施 |
| [058](058-decision-records.md) | 2026-10-21 | 决策记录与记录规则 | 已实施 |
| [059](059-rsx-props-derive.md) | 2026-10-21 | rsx 组件语法：`#[derive(Props)]` | 已实施 |
| [060](060-release-0-1-2-and-eink-rs.md) | 2026-10-21 | 发布 0.1.2，建 eink-rs 仓库 | 已实施 |
| [061](061-split-cr-tab.md) | 2026-10-21 | 输入解析拆开 CR 与 TAB | 已实施 |
| [062](062-release-0-1-3.md) | 2026-10-21 | 发布 0.1.3 | 已实施 |
| [063](063-frame-grid.md) | 2026-10-22 | 帧率：窗口不取整，晚醒不降速 | 已实施 |
| [064](064-release-0-1-4.md) | 2026-10-22 | 发布 0.1.4 | 已实施 |
| [065](065-frame-rate-example.md) | 2026-10-22 | 帧率示例 `frame-rate` | 已实施 |
| [066](066-static-from-fn.md) | 2026-10-22 | `Static::from_fn`：按下标渲染应用自己存的条目 | 已实施 |
| [067](067-release-0-1-5.md) | 2026-10-22 | 发布 0.1.5 | 已实施 |
| [068](068-render-perf.md) | 2026-10-22 | 渲染性能：squash 缓存与 reconcile 去复制 | 已实施 |
| [069](069-terminal-graphics.md) | 2026-10-22 | 终端图形原语：回复解析、`use_graphics` 探测与 `write_passthrough` | 已实施 |
| [070](070-release-0-1-6.md) | 2026-10-22 | 发布 0.1.6 | 实施中 |
