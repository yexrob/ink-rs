# M0 第一阶段：文本层

- 日期：2026-10-20
- 状态：已实施
- 提交：`2503571` build: add taffy and unicode crates；`a322a11` feat(color): parse color strings and convert by level；`c2d593b` feat(text): measure graphemes like string-width；`f186019` feat(text): port ANSI tokenizer and sanitizer；`f70cb59` feat(text): parse strings into styled cells；`f5ed7df` feat(text): wrap, truncate and measure styled lines
- Brief：[m0-text](briefs/m0-text.md)、[m0-text.resume](briefs/m0-text.resume.md)

## 背景

用户授权 11 项决策（「那11个决策你按照你的思路来」）后，Claude 按 [005](005-global-architecture.md) 的执行顺序先做文本层。M0 分三段：文本层、管线、集成，由 Claude 拆分。

## 决策

- 文本层单独成一个阶段，先于 host、layout、paint；接口按第二阶段的需要设计，不做第二阶段。Claude 决定。
- 依赖先由主 agent 加好：taffy 0.15（`std`、`taffy_tree`、`flexbox`）、unicode-segmentation 1.13、unicode-width 0.2，sandbox 离线编译。
- 截断省略号的样式改为跟随 cli-truncate：start、end 模式取保留一侧的样式并去掉超链接；middle 模式和宽度为 1 时无样式。Claude 在 review 时决定，原规则来自它自己 brief 的写法，与 Ink 有出入。
- `width_data.rs` 不补生成脚本，等升级 Unicode 版本时再补。Claude 依据「勿增实体」决定。

## 方案

worker：Codex（`codex exec -s workspace-write`）。[m0-text](briefs/m0-text.md) 要求：

- `src/color.rs`：`Color::hex`、`FromStr for Color`；按 chalk 6 / ansi-styles 7 的规则把颜色降到目标 `ColorLevel`（truecolor → 256 → 16，`None` 不着色）；颜色的 SGR 代码。
- 移植 `ansi-tokenizer.ts` 和 `sanitize-ansi.ts`，保持字符串级，作为外部字符串进入单元格前的安全边界。
- `src/text/width.rs`：按 string-width 8.3 的规则算字素宽度。
- `src/text/styled.rs`：`CharStyle`、`StyledChar`、`parse`、`serialize`，外层样式补未设属性的叠加函数，制表符展开。只保留 OSC 8。
- `measure`、`wrap` 改在样式行上工作，移植 wrap-ansi 10 和 cli-truncate 6；`tests.rs` 里写同名辅助函数适配旧调用。
- 规则：不加测试，不改期望；只跑 `cargo test --lib text::`、`--test colorize`、clippy；不提交；疑似 Ink 副作用的期望要停下报告，不写特例。

## 结果

- Codex 中途因 Claude 提交 `CLAUDE.md`、`design.md` 停下，用 [m0-text.resume](briefs/m0-text.resume.md) 续跑，见 [006](006-small-step-commits.md)。
- Codex 报告：lib 的 text 测试 103 个、colorize 16 个通过。`width_data.rs` 由临时脚本生成（Node 24.17 / ICU 78.3 / Unicode 17.0、unicode-width 0.2.2 附带的 emoji-test.txt、get-east-asian-width 1.7.0），脚本删除，文件头记数据来源。没做 NFC 归一化。字素宽度可以大于 2。
- Claude review：
  - 改省略号样式（见决策）。第二阶段会在截断后叠加 Text 自身样式，省略号因此带 Text 的颜色。
  - `wrap.rs`、`measure.rs` 两处提到 ANSI 字符串的过时注释改掉。
  - 怀疑 `rgb(...)` 的空白处理有误，核对 Ink 正则是 `\s*`，实现一致，不改。
  - NFC：wrap-ansi 做归一化，ink-rs 缺这一步；第三阶段若有测试依赖再提出。本范围内没有后续。
- 拆成 6 个提交，每个单独编译通过。测试：`cargo test --lib text::` 103 通过，`--test colorize` 16 通过，clippy 无警告。
- 下一步第二阶段，见 [008](008-m0-pipeline.md)。
