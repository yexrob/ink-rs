# M0 第三阶段第一批修复与 Yoga 差异的取舍

- 日期：2026-10-20
- 状态：已实施
- 提交：`cdc2c95` fix(text): defer nested span styles past leading marks；`6e131a3` refactor(paint): split row serialization and carry absolute positions
- Brief：[m0-fix1](briefs/m0-fix1.md)

## 背景

第二阶段集成后（[008](008-m0-pipeline.md)），全量测试剩下两类失败：`styled_combining_marks` 的一个组合字符用例，和几个布局用例（`content_offset` 的 flex basis、行内 baseline、2 个 aspect ratio）。review 还发现 `Frame::serialize` 是 rustfmt 排不开的一长串链式调用，paint 每个节点向上遍历祖先取绝对坐标，一次绘制为 O(节点数 × 深度)，M2 每帧都要重画。

Claude 用 probe 核实：Yoga 计算 flex basis 时以容器宽度为上限（`AtMost`）测量子节点，taffy 用 max-content。这会影响"长文本加短文本并排"这种常见布局。

## 决策

- 布局差异不进这一批，先修组合字符和两处 paint 整理。Claude 决定。
- 组合字符规则：嵌套 span 的样式在每行从第一个不全由组合符组成的字素开始，前导组合符保留 span 外的样式，与 Ink 的 `sanitizeAnsi` 一致。Claude 决定，并要求和 `styled::append` 把组合符并入前一字素的规则读起来是同一个设计。
- Yoga 差异的方向由用户选，Claude 提出两个选项。用户选「自有 host 树 (Recommended)」：host 树改为自有结构，实现 taffy 的底层 trait，取代 `TaffyTree<HostNode>`，复刻 Yoga 的 flex basis 和 aspect ratio；baseline 记为偏差，改 1 个期望。这推翻了 [005](005-global-architecture.md) 的决策 1。

## 备选与取舍

- **保留 `TaffyTree`**：aspect ratio 在样式映射层处理；flex basis 和 baseline 接受 CSS 语义，改 2 个期望并记为偏差。代价是行内长短文本并排时，布局和 Ink 不同。
- **自有 host 树**：能拦截子节点测量，复刻 Yoga 语义；代价是两三百行对接代码，要改 design.md 的 host 决策。M1 的测量缓存和脏标记可以放在这个结构上。
- Claude 的 probe 结论：Yoga 的 aspect ratio 由主轴尺寸推出交叉轴尺寸，忽略交叉轴样式和 stretch；baseline 的行高差异属 CSS 与 Yoga 的分歧。

## 方案

worker：Codex，[m0-fix1](briefs/m0-fix1.md) 三项任务：

1. 实现上面的组合字符规则，修 `a_styled_mark_without_a_base_character_keeps_its_position_and_style`。
2. 拆分 `Frame::serialize`，例如单独一个把一行转成样式行、按 Ink `trimEnd()` 去掉末尾默认空格的函数。
3. paint 遍历时向下传父节点未取整的绝对坐标，查找变为 O(1)，取整规则不变。

规则：不改 `src/layout.rs` 中任务 3 以外的部分；不加测试，不改期望。只跑 `cargo test --lib`、8 个测试目标和 clippy；`content_offset` 有 1 个已知失败。sandbox 写不了 `.git`，不提交，在报告里列建议的提交。

## 结果

- Codex 报告：`styled_combining_marks` 9 个全过；指定目标 234 通过、1 个已知失败；`cargo test --lib` 103 通过；clippy 通过。它建议拆 3 个提交。
- Claude review 通过，提交 2 个：`cdc2c95`；`6e131a3` 合并了 Codex 建议的 refactor 和 perf 两个提交。
- 之后全量测试：611 通过，4 失败（`content_offset` 的 flex basis、`row_align_self_baseline`、2 个 aspect ratio），4 个 ignored。上游 skip 的 `set_min_width_in_percent` 在 taffy 下通过，Claude 计划去掉它的 `#[ignore]`。
- 自有 host 树的实施（design.md 改写、m0-hosttree brief、`f863c3d`、`d374fa6`、`5b8876e`、`7174b48`、`1bf580a`）见 [010](010-m0-own-host-tree.md)。
- 记录中未见用户回答这道选择题的准确时间，只知道在 2026-10-20 01:55 提问之后、08:09 提交本批之前。
