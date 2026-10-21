# rsx 组件语法：`#[derive(Props)]`

- 日期：2026-10-21
- 状态：已实施
- 提交：`cb5e456` feat(macros): derive Props for components in rsx!
- Brief：[rsx-props](briefs/rsx-props.md)

## 背景

[056](056-upper-framework.md) 定下原语之后做 rsx 的组件语法，[057](057-runtime-primitives.md) 的原语合并后开始。`rsx!` 里组件只能写成 `{component(f, props)}` 表达式，和 `View`、`Text` 的写法不一致。Ink UI 移植和上层框架的组件都要接 children 和回调，这个写法要先定。

Claude 给出设计和四个待定点，用户问示例里的组件函数有什么用，Claude 解释后用户答「剩下的就按照你的推荐来」。

## 决策

以下由 Claude 提出，用户授权（「剩下的就按照你的推荐来」）：

- 在 props 结构体上派生 `Props`，`#[props(component = f)]` 指明渲染它的组件函数。派生生成每个字段的 setter、`children` 字段的 `child` 和 `children`、`key`，以及 `impl From<结构体> for Element`。`rsx!` 的语法不变，组件写成 `Layout(title: "Deploy") { .. }`。
- setter 默认收字段本身的类型，字段标 `#[props(into)]` 时收 `impl Into<T>`。
- `rsx!` 的路径没写泛型参数时用 `Path::default()` 构造，让泛型 props 推断类型参数；写了泛型参数时保留 `<Path>::default()`。
- 不做：必填 props 的编译期检查、默认值属性、`Callback` 的 `Default`、闭包到 `Callback` 的 `From`。必填 props 由用户写 `new`，在 rsx 里作位置参数；可选回调写成 `Option<Callback>`。

## 备选与取舍

- **Dioxus 式 `#[component]`**：在函数上加属性，参数变成 props。它改变 `fn(&mut Scope, &P)` 的组件模型，宏要生成结构体、改写函数签名。不选。
- **setter 一律收 `impl Into<T>`**：`impl Into<u16>` 收不了整数字面量 `40`，字面量回落到 `i32`，报 `u16: From<i32>` 未实现。用 rustc 验证过。
- **用 bon、typed-builder 这类 crate**：它们生成 `builder()…build()` 的类型状态 builder，和 `rsx!` 的 `<Path>::default()` 加链式方法的展开对不上，还要加依赖。
- **保留 `<Path>::default()`**：泛型结构体在这个位置不推断参数，报 E0107。用 rustc 验证过。

## 方案

- 派一个 Codex worker，worktree 在 scratchpad 的 `wt/rsx-props`，brief 见 [rsx-props](briefs/rsx-props.md)。要点：
  - derive 放在 macros crate 的独立模块，`ink_rs` 根导出 `Props`。
  - 选项只有 `component` 和 `into`，错误带 span；字段名 `key`、`child` 报错。
  - 生成代码用绝对路径，保留用户 span。
  - `rsx!` 按路径有无泛型参数选构造形式。
- 测试：`tests/rsx.rs` 两个，派生组件和 builder 写法比较渲染结果与 `Element` 相等；泛型 props 推断。这是新公开 API 的验收，不加 trybuild。
- 验收命令：`--test rsx`、workspace 的 doc 测试、fmt、两套 feature 的 clippy（含示例）、`cargo doc -D warnings`。全量由主 agent 集成时跑。
- `docs/design.md` 写法一节由主 agent 集成时更新。

## 结果

- Codex 按 brief 完成，报告无偏离。derive 在 `macros/src/props.rs`，`Self: PartialEq + 'static` 约束只加在 `key` 和 `From` 上，setter 不受约束。
- review 后主 agent 在 worktree 里改了三处，没有派回：
  - worker 按语法检查 `children` 是否写成 `Vec<Element>`，类型别名会被拒。改为在生成代码里按字段类型的 span 绑定 `&mut Vec<Element>`：别名可用，类型写错时错误指向字段类型。
  - worker 为避开结构体上同名的泛型参数 `I`，给 `children` 方法的类型参数循环改名。改用 `impl IntoIterator<Item = impl Into<Element>>`，不再需要类型参数名。
  - 第一个测试把同一棵 rsx 树写了两遍，另有一个只传 `None` 的回调字段。合成一棵树，同时比较 `Element` 相等和渲染结果，去掉回调字段。
- 手工核对：结构体带泛型参数 `I` 和 `children` 时能编译；`children` 写成类型别名能编译；写成 `Vec<String>` 时报错位置是字段类型。
- 全量：`cargo test --workspace` 1330 通过，`--no-default-features` 1296 通过，0 失败；fmt、两套 feature 的 clippy、`cargo doc -D warnings` 通过。
- `docs/design.md` 写法一节加组件写法和示例，构造规则补泛型参数的情况。
- 用户问示例里的 `Element::fragment(p.children.clone())` 能否直接写 `p.children.clone()`：能，`Vec<T>` 实现了 `Into<Element>`。design.md 的示例用后者。
- worktree 删除。
