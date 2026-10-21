# 自有 host 树，模拟 Yoga 的 flex basis 与 aspect ratio

- 日期：2026-10-20
- 状态：已实施
- 提交：`f863c3d` docs: own the host tree and emulate Yoga's flex basis and aspect ratio；`d374fa6` test: expect CSS line sizing for row baseline alignment；`5b8876e` test: run the percentage min width case that passes under taffy；`7174b48` feat(layout): own the host tree and size flex items like Yoga；`1bf580a` docs: update the ignored test count
- Brief：[m0-hosttree](briefs/m0-hosttree.md)

## 背景

M0 第 3 阶段第 1 批之后，布局测试里还有几项与 Ink 不一致，都来自 taffy 与 Yoga 的差异：

- content_offset 的 `disjoint_nested_clips…`：flex basis 的算法不同；
- width_height 的 `set_aspect_ratio_with_height` 和 `set_aspect_ratio_with_width_and_height`：aspect ratio 的处理不同；
- `row_align_self_baseline`：baseline 的行高算法不同。

Claude 向用户提问「布局与 Yoga 的差异按哪个方向处理？」。问题在上一时段提出，用户在本时段开头作答。

## 决策

- 用自有的 host 树替换 `TaffyTree`，在这棵树上模拟 Yoga 的 flex basis 和 aspect ratio。用户选「自有 host 树 (Recommended)」。
- baseline 不模拟，保留 CSS 的行高算法，记为与 Ink 的差异。这一点在 brief 中定下，属 Claude 在用户选项范围内的取舍。

## 备选与取舍

- 「保留 TaffyTree」：aspect ratio 在样式映射层处理，flex basis 和 baseline 用 CSS 语义，改两条期望值并记为差异。代价是长短文本并排时布局与 Ink 不同。
- 自有 host 树：约 200–300 行胶水代码。M1 的测量缓存和脏标记可以建在同一结构上。用户选了这一项。

## 方案

Codex 执行，于 00:17 UTC（本地 08:17）派发。先提交文档和测试期望（`f863c3d`、`d374fa6`、`5b8876e`），再由 Codex 实现。Brief 要求：

1. 自有 Vec arena，实现 taffy 0.15 的低层 trait。
2. 按 Yoga 的 markDirty 清缓存：节点和它的祖先。
3. 在查缓存之前把 flex basis 的约束改写为 AtMost。
4. 通过 `get_flexbox_child_style` 给子项一个带 aspect ratio 的样式视图。
5. baseline 保留为差异；不新增测试；只跑 brief 点名的测试目标。

布局设计见 [布局与文本](../design.md#布局与文本)。

## 结果

- Codex 报告：加了 `record_inner_size` 和 `constrain_measurement`，以及 aspect ratio 视图。lib 103 个、集成 409 个通过，clippy 干净。
- Codex 列出的局限：taffy 在 grow/shrink 之前就定下交叉轴尺寸；两轴都由内容定尺寸的子项拿不到 Yoga 的比例；min/max 与 Yoga 是否一致没有验证。
- 主 agent review 时的修改：
  - 删掉 `fixed_limits`（把 min ≥ max 当作固定尺寸），它偏离 Yoga；
  - 删掉没有用到的 MinContent→MaxContent 改写；
  - 删掉环检测断言；
  - 访问器改名为 `props`；
  - 精简 Codex 写进 design.md 的段落。
- 全量 616 通过、0 失败、3 忽略。M0 完成。
