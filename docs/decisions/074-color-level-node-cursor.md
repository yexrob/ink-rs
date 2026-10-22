# `use_color_level` 和锚定节点的光标

- 日期：2026-10-22
- 状态：实施中
- 提交：无
- Brief：[primitives](briefs/primitives.md)

## 背景

eink-rs 的两个新组件缺 ink-rs 的原语（eink-rs 的规则 5：缺原语时停下问用户，原语放在 ink-rs）：

- `Code` 给 diff 行画底色。底色降到 16 色后难看，组件要按色深选底色或前景色；ink-rs 在绘制时按 `ColorLevel` 降级，组件读不到色深。
- `Composer` 自己折行，用光标给输入法定位。`use_cursor` 接收帧坐标，组件要从 `measure_element` 推算，读到的是上一次提交的布局；祖先移动时光标慢一帧，候选窗跟着跳。

## 决策

- 用户定：选「加两个 (Recommended)」：`use_color_level` 和锚定节点的光标进 0.1.7，由一个 Codex 来做。
- Claude 按用户的选择定细节：
  - `Scope::use_color_level() -> ColorLevel` 返回这个应用绘制用的色深。色深在应用的生命周期里不变，hook 不安排重绘。
  - `Scope::use_node_cursor(node, position)`（名字待 worker 定）和 `use_cursor` 共用一个光标槽；存储里保留节点和偏移，应用在布局之后、绘制之前按同一步提交的布局算出位置。偏移从节点边框盒的左上角算，和 `ElementMetrics` 同一原点。节点没有布局时隐藏光标。
  - `use_cursor` 和 `CursorPosition` 不变。

## 备选与取舍

- **不加，用固定 RGB 底色**：16 色终端下难看，组件无从判断。
- **不加光标锚定，接受慢一帧**：输入法候选窗在多行输入增长时跳动。
- **给 `CursorPosition` 加锚点字段**：公开结构体加字段，字面量构造会编译失败，属于破坏性改动。

## 方案

一个 Codex worker，worktree `wt/ink-primitives`。brief 见 [primitives](briefs/primitives.md)。测试三例：祖先移动时光标在同一帧跟上（这个 hook 存在的理由）；锚定的节点卸载后光标隐藏；`render_with` 和 `testing::mount` 两条路径读到设置的色深。验收：fmt；clippy 两组 feature；`--lib`；`hooks`、`runtime` 和覆盖 `use_cursor` 的测试文件。

它第一个派出：eink-rs 的 `Composer`（记录 027）和 `Code`（记录 028）等它合入 main 后派出，worktree 用 `[patch.crates-io]` 指向 ink-rs 的本地 main，ink-rs 发 0.1.7 后去掉。进 0.1.7。
