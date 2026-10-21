# 折行时做 NFC 归一化

- 日期：2026-10-20
- 状态：已实施
- 提交：`5c17d3d` fix(text): NFC-normalize wrapped lines as wrap-ansi does
- Brief：[fix-nfc](briefs/fix-nfc.md)

## 背景

遗留项"NFC 偏差"：wrap-ansi 对输入做 `normalize()`（NFC），ink-rs 不做。用户要求遗留项「修完再发」（见 [043](043-release-prep.md)）。

## 决策

- 引入 `unicode-normalization`，只在 `Wrap` / `Hard` 的折行路径归一化，与 Ink 一致。用户在提问中选了推荐项「引入 unicode-normalization」（选择见会话摘要和 Claude 的回复「四项都按推荐方案做」）。
- 放得下的文本和截断模式不归一化：Ink 只在文本超过宽度时调用 wrap，`tests/text_width.rs` 也期望放得下的文本保持原样。
- 按格子逐个字素归一化，再用同一个字素宽度函数重算宽度。组合发生在字素簇内部，逐格归一化与整串归一化结果相同。Claude 在 brief 中定。

规则见 [布局与文本](../design.md#布局与文本) 的"断行"。

## 备选与取舍

- 记为偏差，不加依赖，在 ink-tests.md 写明。调研 agent 依据「如非必要，勿增实体」推荐这一项。Claude 认为用户要求修完再发，把取舍交给用户，用户选了加依赖。
- 在仓库里自带组合表：调研中列为一种做法，没有采用，记录中未见单独讨论。

## 方案

- 调研：Claude 子 agent（opus-medium，"Research NFC and width data"），与宽度表生成脚本合在一个任务里（宽度表见 [049](049-width-tables-unicode-18.md)）。
- 调研结论：
  - Ink 只在 wrap-ansi 里归一化，只在 Text 需要折行时。
  - 可见差别：组合后的字节（é、韩文音节）；42 个分写元音符号组合后宽度改变（如泰米尔文），折行点随之移动。
  - `unicode-normalization` 0.1.25 在本地 registry，支持 Unicode 17，带来 tinyvec。
- 派 Codex（worktree `wt/fix-nfc`）。Brief 要点：
  - 在 `src/text/wrap.rs` 的 `Wrap | Hard` 分支、`max_width == 0` 返回之后归一化。
  - 核对 measure 与 paint 都经过 `wrap_text`，布局高度与绘制一致。
  - `tests/text_width.rs` 加 1 个测试：含 `e` + U+0301 和泰米尔分写元音的 Text，折行后输出组合形式，按组合后的宽度断行。
  - design.md 断行一段加一句。
- 验收：`--test text_width`、`--test text`、`--lib`，加 fmt 和 workspace clippy。

## 结果

- Codex 加 `normalize_line`，调用顺序为先归一化，再展开制表符，再折行，与 wrap-ansi 一致。核对 `src/layout.rs` 和 `src/paint.rs` 都在文本超宽时才调用 `wrap_text`。
- 偏差：`tests/wrap.rs` 不存在，改跑 `wrap_text` 测试目标。
- Claude 审查通过，在 CRLF 之后提交 `5c17d3d`。
