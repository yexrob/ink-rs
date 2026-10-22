# 渲染性能：squash 缓存与 reconcile 去复制

- 日期：2026-10-22
- 状态：实施中
- 提交：无
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
