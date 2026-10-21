# M0 第二阶段：host 树、布局、绘制与移植偏差修正

- 日期：2026-10-20
- 状态：已实施
- 提交：`02b7134` feat(text): join graphemes across inline spans；`8bd762e` feat: build the host tree and lay it out with taffy；`77290c2` feat: paint frames and render elements to string；`5171096` test: fix port mismatches and drop Ink artifacts
- Brief：[m0-pipeline](briefs/m0-pipeline.md)

## 背景

文本层完成后（[007](007-m0-text-layer.md)），M0 第二阶段要让 `render_to_string` 跑通。架构依据 [005](005-global-architecture.md) 的 11 项决策，用户授权：「那11个决策你按照你的思路来」。

## 决策

- host 树为 `TaffyTree<HostNode>`，对外是一组变更操作（建节点、设属性、插入、删除）；`Element` 转 host 树只是它的一个调用方，M1 runtime 是下一个。Claude 决定，依据决策 1。
- `Text` 是一个 taffy 叶子，嵌套内容作为行内树存在 host 节点上。依据决策 3。
- paint 是纯函数，输出 `Frame`；宽字符的续格显式表示，供 M2 差分。依据决策 2。
- 集成时发现的移植错误和 Ink 副作用由 Claude 改期望，并记进 `docs/ink-tests.md`。依据决策 10；记录中未见用户对具体改动的单独确认。

## 方案

worker：Codex。[m0-pipeline](briefs/m0-pipeline.md) 要求：

- 按 Ink 的 `dom.ts`、`reconciler.ts`、各组件实现 host 树，含背景色继承、`Static`、`Transform`、`Text` 内 `View` 时 panic。
- 布局映射里处理 Yoga 差异：根节点用 Yoga 默认值，未设 min 尺寸为 0，`align_content` 默认 `FlexStart`，`Overflow` 不影响布局，`position: static` 忽略偏移；文本测量回调按 `measureTextNode`。
- 按 `render-node-to-output.ts`、`output.ts` 等实现 paint：遍历、裁剪栈、宽字符、边框、背景、样式叠加（`fill_from` 加 `Color::at_level`）。
- `render_to_string` 先输出 Static 再输出主区。删掉 `#![allow(dead_code)]`。
- 不加测试，不改期望。只跑 `cargo test --lib`、7 个测试目标（render_to_string、text、flex_direction、padding、borders、background、colorize）和 clippy。按 `CLAUDE.md` 第 3 条小步提交，带 `Co-Authored-By: Codex` 尾注。

## 结果

- Codex 的 sandbox 写不了 `.git`，改动未提交。新增 `host.rs`、`host/build.rs`、`layout.rs`、`paint.rs`，改 `render.rs`。报告 7 个期望冲突：`text` 6 个颜色用例，`background` 1 个 Static 末尾换行。
- Claude review 通过（rustfmt、clippy 干净），跑全量测试，失败分三类：
  - 移植错误：`text` 的 hex、rgb、ansi256 用例，Ink 的期望由同一个 chalk 实例生成，与色彩级别无关，改用 `render_truecolor`；Static 背景用例在 Ink 里经 `render()` 的 debug 模式取输出，末尾带 `\n`，而 `renderToString` 去掉它，期望随之去掉。
  - Ink 副作用（决策 10）：`overflow` 越界写入用例的内部行改为 12 格；`c1_rendering` 宽度 4 的超链接每段只打开、关闭一次。
  - 留给第三阶段：组合字符的样式、`content_offset` 的 flex basis、行内 baseline、2 个 aspect ratio。
- 提交：Claude 把 Codex 的改动拆成 `02b7134`、`8bd762e`、`77290c2`，带 Codex 和 Claude 两条尾注；期望修正为 `5171096`，`docs/ink-tests.md` 的"与 Ink 测试的差异"一节加 4 条，删去"反直觉行为"中关于越界行宽 11 格的一条。
- 后续见 [009](009-m0-fix1-and-yoga-gap.md)。
