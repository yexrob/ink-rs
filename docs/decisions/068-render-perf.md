# 渲染性能：squash 缓存与 reconcile 去复制

- 日期：2026-10-22
- 状态：已实施
- 提交：4b1fe0a、216e816
- Brief：[render-perf](briefs/render-perf.md)

## 背景

eink-rs 的 Markdown 流式渲染（eink-rs 的记录 018）做完后，用户要求「做完记得压测大数据量的场景」。主 agent 用 eink-rs 的 `markdown-stress` 压测（release，`testing::mount`，120 fps），有两个场景落在阈值外：

| 场景 | p50 / p99 / max（ms） |
|---|---|
| 列表项里含 300 行代码块，尾部每帧重渲染整项 | 37.8 / 134 / 199 |
| 每帧到达 4 KB 混合回答，每帧 100 段左右进 `Static` | 12.97 / 36.9 / 288 |

macOS `sample` 的采样里，eink-rs 的代码不到帧时间的 10%，其余在 ink-rs：

- `TextNode::squash` 占 30%–48%。每个文本节点每帧被 squash 13 次左右：`constrain_measurement` 的 AtMost 改写让 taffy 缓存落空，每次测量各 squash 一次，绘制时多一次。
- `reconcile` 每一层 clone 元素，连同整棵子树；新建实例时多 clone 一次。深度为 d 的树每次渲染复制 d 次左右。

实验（之后还原）：惰性缓存 squash 后，第一个场景 37.8 → 20.5 ms，第二个 13.0 → 10.9 ms。这时 `Node::clone` 占 15%，`drop_in_place<Node>` 占 22.8%。在此之上关掉 AtMost 改写，第一个场景到 18.6 ms。

主 agent 给出 0.1.5 照发、性能另立记录的建议。用户选「性能做进 0.1.5」。主 agent 给出方案后，用户回复「派给codex」。

## 决策

- 用户定（「性能做进 0.1.5」）：这次性能工作进 0.1.5，0.1.5 的发布（[067](067-release-0-1-5.md)）排在它之后。
- 用户定（「派给codex」，回复主 agent 给出的方案）：
  - 文本 host 节点在设置 props 时 squash 一次、展开制表符，连同自然宽高存在节点上，测量和绘制读它。
  - reconcile 把子元素从传入的元素里移走，不复制；新实例不复制元素。
  - 与 Ink 的差别：`Text` 里嵌套的 Transform 函数在文本 props 变化时调用，Ink 在每次测量和渲染时调用。纯函数下输出相同，写进 `docs/design.md`。
  - 不做 AtMost 改写、绘制限于可见行、`Element` 子节点改 `Rc`。
  - 目标（eink-rs `markdown-stress`，release）：第一个场景 p50 ≤ 20 ms，第二个 p50 ≤ 10 ms，其余场景不退步。
- Claude 按同一方案定：撤掉本地没推送的 a777065（升 0.1.5 版本号），性能提交合入后升版本号，发布提交排在最后。
- Claude 定（review Codex 的实现）：squash 缓存在节点第一次测量或绘制时填，`set_props` 只清空它。Codex 在 `set_props` 里填，嵌套 Transform 因此在 reconcile 的 panic guard 里运行，为让它的 panic 不被捕获，Codex 加了 `uncaught` 包装和 `Uncaught` 载荷。改到布局和绘制时填，squash 和改动前一样在 guard 外运行，这套机制和为它加的测试都不需要。Transform 的调用次数不变：文本 props 每变化一次，至多调用一次。

## 备选与取舍

- **0.1.5 照发，性能进 0.1.6**：主 agent 的建议，用户没选。
- **去掉 AtMost 改写**：这是对齐 Yoga 的行为（`docs/design.md`「布局与文本」）。实验里在 squash 缓存之上去掉它，37.8 ms 的场景从 20.5 降到 18.6 ms，收益小。
- **绘制限于终端可见的行**：输出比终端高时，Ink 整帧重写，改了偏离 Ink。
- **`Element` 子节点改 `Rc`**：改公共表示；把子元素移走可以达到同样效果。
- **squash 结果按字符串全局缓存**：Ink 的 `measureText` 这样做，`docs/design.md` 写明常驻树不用它；缓存放在 host 节点上，跟节点的 props 走。

## 方案

一个 Codex worker（`codex exec -s workspace-write`）：

- 工作目录是 scratchpad 里的 worktree `ink-perf`，基于 main；brief 见 [render-perf](briefs/render-perf.md)。
- 它不提交（沙箱写不了 `.git`），不改文档。它跑 fmt、两套 feature 的 clippy、`--lib` 和 brief 列出的集成测试。

主 agent 集成：

1. review，按两个逻辑改动分两次提交。
2. fmt、两套 feature 的 clippy、全量测试。
3. `docs/design.md` 写 squash 缓存和 Transform 调用次数的差别。
4. eink-rs 的 worktree 经 `[patch]` 指向本仓库，重跑 `markdown-stress` 的全部场景，和上表对比。
5. 补本记录的结果，删 worktree 和分支。之后按 067 发布 0.1.5。

## 结果

Codex 交回两项改动，没有测耗时。主 agent review 后改了一处，删了 Codex 加的两个测试，分两次提交：

- **4b1fe0a squash 缓存**：缓存是 host 节点上的 `OnceCell`，第一次测量或绘制时填，`set_props` 清空。Codex 在 `set_props` 里填，配套的 `uncaught` 包装和 `Uncaught` 载荷没有合入，理由见上面的决策。Codex 加的两个测试：一个数 Transform 的调用次数，主 agent 让 `set_props` 不清空缓存跑了一遍，现有测试里有 7 个失败，缓存失效已有覆盖；另一个测嵌套 Transform 的 panic 不被捕获，缓存改到布局和绘制时填后，这件事和改动前一样，不需要新测试。`docs/design.md`「布局与文本」写了缓存和 Transform 调用次数的差别。
- **216e816 reconcile 去复制**：按 Codex 的实现合入。View、Text、Transform、Fragment 的子元素用 `mem::take` 移走；新实例从 `Element::empty()` 开始，进入 reconcile 时换成传入的元素；组件 props 不动。`Instance::element` 的读者（`matches`、`first_static`、`finish_static`、`inline_children`、背景色、`wrap`、`transform`、`node_ref`、组件的 `same_props`）读的是节点种类、样式、key 和函数，不读子元素。改动前 `abandon` 克隆根实例的元素来重放错误总览，子元素移走后读不到，改为错误总览挂载期间 `rerender` 时存一份副本。
- **检查**：fmt；两套 feature 的 clippy；全量测试默认 feature 1331 个、`--no-default-features` 1297 个，全部通过。

eink-rs `markdown-stress`（release，120 fps，每帧 64 B，另注的除外），p50 / p90 / p99 / max（ms）：

| 场景 | 改动前 | 改动后 |
|---|---|---|
| Rust 代码块 5000 行 | 0.36 / 0.72 / 1.52 / 7.70 | 0.21 / 0.35 / 0.64 / 5.25 |
| TS 代码块 5000 行 | 0.35 / 0.51 / 0.99 / 8.15 | 0.22 / 0.30 / 0.41 / 2.09 |
| 300 项列表 | 0.33 / 0.41 / 0.78 / 1.98 | 0.20 / 0.24 / 0.32 / 0.53 |
| 300 行表格 | 8.96 / 16.85 / 38.31 / 66.77 | 4.58 / 8.45 / 9.87 / 20.09 |
| 长段落 | 7.65 / 13.63 / 15.31 / 40.71 | 5.13 / 9.05 / 10.05 / 12.41 |
| 列表项里 300 行代码 | 37.8 / 73.8 / 134 / 199 | 7.22 / 12.80 / 13.73 / 89.61 |
| 1 MB 混合，每帧 4 KB | 12.97 / 15.73 / 36.9 / 288 | 9.02 / 11.44 / 26.17 / 177.53 |
| 100 KB 积压，pacing 300 ms | p99 12.74，max 125 | p99 6.01，max 112 |
| 1 MB 混合 | p99 0.92，max 9.62，5.64 ms/KB | p99 0.64，max 21.27，4.00 ms/KB |

两个目标达到：列表项里 300 行代码的 p50 7.22 ms（目标 ≤ 20），每帧 4 KB 的 p50 9.02 ms（目标 ≤ 10）。1 MB 混合的 max 从 9.62 升到 21.27 ms，改动前后的版本每个跑两次：改动后 max 23.51、16.08 ms，改动前 max 90.92、8.70 ms，p50 改动后 0.23–0.25、改动前 0.36–0.38 ms。max 是单帧，随机器负载（load average 4–27）起落，不算退步。

worktree `ink-perf` 和分支删掉了。
