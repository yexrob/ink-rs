# `Color::Reset`：对应 Ink 的 `backgroundColor=""`

- 日期：2026-10-20
- 状态：已实施
- 提交：`0d05344` feat(color): add Color::Reset for Ink's empty backgroundColor
- Brief：[fix-color](briefs/fix-color.md)

## 背景

Ink 的 `<Text backgroundColor="">` 表示不要自己的背景，也不继承外层 `Box` 的背景。ink-rs 的 builder 接受 `impl Into<Option<Color>>`，`None` 表示继承，没有"显式无背景"的写法。Ink 测试 `Mixed text with and without background inheritance` 因此记在"缺 API"里。用户要求遗留项「修完再发」（见 [043](043-release-prep.md)）。

## 决策

- 给 `Color` 加一个变体表示终端默认色，不改现有签名。前景色和边框也能用。
- 变体名用 `Color::Reset`。用户在提问中选了推荐项（选择见会话摘要和 Claude 的回复「四项都按推荐方案做」）。理由：与 crossterm、ratatui 同名，语义相同（SGR 39/49），不会和 `Default` trait 的 `Color::default()` 混淆。
- `FromStr` 不解析空串。随名称一起在提问中给出。
- `at_level` 对 `Reset` 返回 `None`，不加 SGR；`Some(Reset)` 在继承的 `.or(..)` 中胜出，再解析为无色，由此阻断继承。嵌套在 Text 里时保留外层 Text 的样式，同 Ink。

规则见 [布局与文本](../design.md#布局与文本) 的"样式叠加"。

## 备选与取舍

- `Color::Default`：调研 agent 推荐的名字。和 `Color::default()` 名字相近，没有采用。
- 单独的 builder 方法（如 `no_background()`）、把 setter 改成 `Background { Inherit, None, Color }` 枚举：调研任务列出的选项，调研推荐加变体，前者不覆盖前景和边框，后者改公开签名。

## 方案

- 调研：Claude 子 agent（opus-medium，"Research Text no-background API"）。要求弄清 Ink 的语义、ink-rs 的继承模型、各 API 选项与 `rsx!` 的配合，给出测试计划。期望输出取自第一会话记录的 Ink 测试结果。
- 派 Codex（worktree `wt/fix-color`）。Brief 要点：
  - `src/color.rs` 加变体和文档；`sgr` 映射 39/49 让 match 保持完整。
  - 检查所有解析颜色的地方（`host.rs`、`paint.rs` 背景与边框、runtime 继承、错误界面），`Reset` 都按无色处理。
  - 按 Ink 的顺序移植该测试到 `tests/background.rs`，期望 `{bg=green}Inherited {/}No BG {bg=red}Red BG{/}`。
  - 重新统计并更新 `docs/ink-tests.md` 的数字，design.md 加一句。
- 验收：`--test background`、`--lib`、`--test borders --test components`，加 fmt 和 workspace clippy。

## 结果

- Codex 在 `src/paint.rs` 的背景填充里跳过 `Reset`，避免无色空格盖住底层内容；按颜色值过滤，不按 `at_level` 结果过滤，保留禁色模式下其他背景的覆盖行为。
- `docs/ink-tests.md`：移植 618，`#[test]` 622，未移植 137；background 28；删去缺 API 条目。
- Claude 集成时把 Codex 放在枚举首位的 `Reset` 移到 `Rgb` 之后。
- 提交 `0d05344`。
