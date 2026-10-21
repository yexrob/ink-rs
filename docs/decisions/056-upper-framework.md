# 上层框架：新仓库，参照 Pastel 与 Ink UI

- 日期：2026-10-21
- 状态：部分实施（原语见 057，其余待做）
- 提交：无
- Brief：无。两份调研派给 Claude 子 agent（general-purpose），prompt 见会话记录，未存为 brief

## 背景

0.1.1 发布后，用户提出：「我们现在相当于有了一个比较完善的tui library了，但是还比较底层，我们是不是缺一个类似react 生态的nextjs 的框架 让用户可以更便捷的使用？ 包括state 管理，路由设计等等(别的方面你来补充) 先讨论一下」。

Claude 的答复：方向同意，不照搬 Next.js，它解决的 SSR、URL 路由、打包、部署在终端里没有。Ink 生态有对应物：Pastel（Ink 作者写的「Next.js-like framework for CLIs」）和 Ink UI（`@inkjs/ui` 组件库）。上层延续「移植 Ink」的做法，就有现成的行为和测试可以对照。Claude 核对 ink-rs API 后指出三处核心缺口：没有 context、children 不能当 props、回调不能当 props。这三项交给 [057](057-runtime-primitives.md)。

## 决策

- 上层放在一个新仓库，不和 ink-rs 放在一起。用户决定（「组织我觉得可以新开一个repo 而不是和ink-rs放到一起。」）。Claude 原提议是同一 workspace 里的两个 crate。
- context、children、回调这些原语放在 ink-rs 里，新仓库依赖它们；开发期新仓库用 git 依赖，它发布前 ink-rs 先发带原语的版本。Claude 提出，用户回复「对 ink-rs可以先开始搞原语部分」。
- 以 Pastel 和 Ink UI 为参照逐项讨论。用户决定（「Pastel， Ink UI这两个我们再详细讨论一下具体的功能」）。
- 下面四项来自 Claude 在 Pastel 讨论中提的三个问题（路由用 A 还是 B、Pastel 是否只借概念、Pastel 之外的几项要不要）和 rsx 组件语法。用户先答「1. A 和 B差异是什么 2. 同意 3. 这是什么」，Claude 解释 A、B 的差异和那几项后，用户答「rsx组件 需要做 按照你说的 等原语搞完。A+B 都做吧，3 那几项按照你的推荐来搞吧」。
  - Pastel 只借概念，不对齐 help 格式和 `-v` 这类细节，不移植它的测试（它的测试大多断言 commander 的 help 文本）。
  - 命令路由 A 和 B 都做：A 是 clap derive 的 enum 路由，做底座；B 是 build.rs 扫描 `src/commands/` 生成 A，叠在 A 上。
  - Pastel 之外补四项：退出码（`Err` 返回 1，参数错误沿用 clap 的 2）、异步命令（接 tokio 驱动）、shell 补全（clap_complete，放可选 feature）、测试（给 argv，在 `testing` 上断言帧）。
  - rsx 的组件语法（`Layout(title: "Deploy") { .. }`，在 props 结构体上派生 builder）放在 ink-rs-macros，等原语合并后做，细节另行讨论。
- Ink UI 的五点按 Claude 的推荐。用户答「别的按照你的推荐」：
  1. 主题：每个组件一个主题类型，各自作为 context 提供，外加一层颜色 token。
  2. 焦点：保持 `is_disabled`，用户自己接 `use_focus`。
  3. 各组件的状态层公开为 headless hook。
  4. 范围：先做 Ink UI 的 13 个组件，第二批做框架时按需定。
  5. 输入框：一个内核支持 mask 和补全来源，EmailInput、PasswordInput 写成薄封装。
- 新仓库名 `eink-rs`。用户决定（「我们原仓库叫ink-rs， 那新仓库就叫做 墨水屏-rs  翻译成英文」），Claude 把「墨水屏」译作 e-ink。crate 名 `eink-rs` 和 `eink` 在 crates.io 上没有被占用（2026-10-21 查）。

## 备选与取舍

- **文件路由（只做 B）**：贴近 Next.js 的手感。代价：加文件要靠 `rerun-if-changed` 触发构建；文件名要是合法标识符；生成的代码在 `OUT_DIR`，跳转和报错指向生成文件；clap 的全局参数、flatten、互斥组要另约定写在哪个文件。B 生成的就是 A，先做 A 不白做。
- **`isDefault`**：clap 没有对应概念。改写 argv 实现「第一个参数不是子命令就交给默认命令」，Claude 建议不做，用 `Option<Cmd>` 回落。
- **Claude 列出不做的**：SSR、打包、部署、服务端组件；热重载在 Rust 下代价高，暂不做。记录中未见用户对此表态。

## 方案

两个只读调研子 agent（general-purpose）并行，Pastel 4.0.1 与 Ink UI 2.0.0 的源码克隆在 scratchpad 的 `ref/` 下，按不可信代码处理，只读不运行。

- **Pastel**：要求列出命令发现、模块导出、zod 到 commander 的映射、位置参数、help 和 version、`_app`、入口，并标出哪些依赖 JS 运行时能力。结论：Pastel 在 commander 和 zod 上做了四件事（文件即命令、schema 生成参数、校验报错、`_app`）；没做环境变量、配置文件、补全、退出码、异步、测试工具；有 version 从当前目录找 package.json、同名命令静默覆盖、`_utils.ts` 也变成命令、数组默认值合并错误等问题，Rust 版用 `CARGO_PKG_VERSION` 和类型系统可以避开。
- **Ink UI**：要求逐个组件列 props、按键、状态机、主题键和边界情况，并按 ink-rs 的缺口逐一标注。结论：约 3100 行，13 个组件、`useSpinner` 和一套主题；不用焦点系统，靠 `isDisabled`；状态 hook 不导出；三个输入框是同一份代码复制三份。源码里的 bug 移植时修掉：开头按 Backspace 删掉首字符、中间 Enter 补全后显示值和 onSubmit 值不一致、光标按 UTF-16 单元计数、Select 的三个问题、Spinner 换 type 后空白、OrderedList 把非 item 子节点算进编号、StatusMessage 和 Alert 多数 variant 测试标成 `test.failing`。

## 结果

- 没有代码或提交。决定记进了项目 memory。
- 整体顺序由 Claude 提出，用户确认（「这个顺序我觉得没问题」）：原语 → rsx 组件语法 → 新仓库移植 Ink UI → Pastel 的 Rust 版（先 A 和那四项，再 B）。
- Ink UI 的五点和仓库名在原语合并后定下，见决策一节。
- Claude 开场提的三个问题（目标是一次运行的 CLI 还是常驻全屏 TUI、移植还是自行设计、有没有驱动案例），记录中未见用户逐条答复。屏内导航、store/query、键位帮助、表单、非交互降级等方向停留在讨论。
- 用户随后要求把历次决策写进 `docs/`，Ink UI 的讨论在此暂停。
