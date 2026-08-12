# HUD 迁移到 Iced（统一框架）计划

| 字段 | 值 |
|------|-----|
| **状态** | Active — Phase 0 未启动 |
| **日期** | 2026-08-11 |
| **前置** | Lemon Fresh 视觉统一（`ui-development-plan.md` Phase 1–2）已完成；实机 QA 基本通过 |
| **目标** | 移除 conrod，HUD 与菜单统一使用 `ui/ice`（Iced fork）；单一渲染器、单一 widget 体系、单一字体/主题通道 |
| **原则** | 每阶段主分支可编译、可玩；模块逐个合入，conrod 并存至最后才拆除；不设长期 feature flag |

---

## 现状盘点（2026-08-11）

### 要迁的量

HUD 共 31 个模块、约 **24k 行** conrod 代码，按行数排序（`voxygen/src/hud/`）：

| 模块 | 行数 | 说明 |
|------|------|------|
| mod.rs | 5391 | HUD 总装：状态机 + 全部窗口调度 |
| diary | 3066 | 角色/技能书 |
| crafting | 2336 | 制作（槽位、配方树、tooltip 最重） |
| map | 1680 | 世界地图 |
| bag | 1633 | 背包（槽位 + 拖拽） |
| skillbar | 1403 | 技能栏 |
| chat | 1082 | 聊天（RichText 多色、输入） |
| minimap | 1054 | 小地图（旋转、target 标记） |
| group / img_ids / trade / util | ~900 | — |
| overhead | 716 | 头顶名称/标记（世界空间） |
| tutorial / slot_grid / social / buffs | 600–660 | — |
| loot_scroller / quest / slots / subtitles / buttons / overitem / popup / item_imgs / prompt_dialog / esc_menu / hotbar / controller_icons / change_notification / animation | <450 | 轻量模块（首批候选） |

### iced 侧已有（`ui/ice/`，菜单已生产使用）

- widget：button、checkbox、container、column、row、stack、space、image（含 Rotation）、scrollable、slider、text、text_input、tooltip、overlay、mouse_detector、aspect_ratio_container、background_container、compound_graphic
- 组件：neat_button；样式：button/checkbox/container/scrollable/slider
- 主题单源 `voxygen/src/ui/theme.rs`（`to_iced`/`to_conrod` 双转换，迁移期共存）
- 字体：`fonts.rs` 已有 iced 通道（含 zh-Hans 通用槽）

### iced 侧缺口（迁移真正难点）

| 缺口 | conrod 现状 | iced 方案 | 工作量 |
|------|------------|----------|--------|
| 世界空间元素 | `Ingame` widget（nametag、HpFloater、目标标记、overhead） | `IcedRenderer` 新增自定义 primitive，渲染时用 `view_projection_mat` 投影 + 视锥剔除（复用 `ui/mod.rs` maintain_internal 中 `ingame_locals` 逻辑） | 高 |
| 物品槽 Slot | `slot` widget（hotbar/bag/crafting 共用，rarity 框 + 悬停） | Container+Image+Text 组合 widget，独立模块 + 测试 | 中 |
| 物品 tooltip | `item_tooltip`（品质色名、属性、需求） | 扩展现有 tooltip 组件，支持富文本行 | 中 |
| 多色富文本 | `RichText`（聊天高亮、overhead） | 自定义 widget 或 Row 组合 | 中 |
| 描边文字 | `OutlinedText`（头顶名字） | 自定义 primitive（描边字形）或双层文字 | 低 |
| 拖拽换位 | slot 内建 drag 逻辑 | 无内置，mouse_detector + overlay + 状态自建 | 高 |
| 次要控件 | ImageSlider、RadioList、ToggleButton、ImageFrame | slider/checkbox 已有；ImageFrame 可组合 | 低 |

依赖：`iced = iced_native`（Imberflur/iced fork，tag `veloren-winit-0.28`）；`conrod_core`（veloren/conrod fork，branch `copypasta_0.7`）。

---

## 决策记录

- **D1 迁移策略**：模块逐个合入，conrod 并存到 Phase 4。不设 feature flag —— 靠合入顺序保证主分支始终可玩（AGENTS：不拿未完成的复杂度换工作产品）。
- **D2 世界空间元素**：迁入 iced 渲染器（投影在 renderer 层），不留 conrod 残余层；否则"统一框架"不成立。
- **D3 egui 不动**：`egui-ui` 保持开发者调试层（实验 shader、调试形状、聊天命令），独立于本迁移。
- **D4 主题单源不变**：迁移期间 `theme.rs` 继续服务两框架；语义色（HP/品质/聊天）冻结，不借迁移重设计。
- **D5 不迁移期间重写**：纯视觉统一（背景图、按钮素材）不纳入本计划，那是 `ui-development-plan.md` 的领域。

---

## 阶段划分

### Phase 0 — 地基（缺什么补什么）

1. 先落库当前未提交改动（物品拾取、block damage、wasm/ 等，与 UI 无关），避免迁移提交纠缠。
2. 移植缺口 widget 到 iced，每个独立合入并带 `#[cfg(test)]`：
   - Slot（含 rarity 框、悬停、数量）
   - ItemTooltip 富文本化
   - RichText 多色 widget
   - OutlinedText 描边 primitive
   - ImageSlider / RadioList / ToggleButton / ImageFrame 组合件
   - 拖拽原语（mouse_detector + overlay + 状态）
   - 世界投影 primitive（D2）
3. 验证 iced 字体槽在 HUD 侧的中文渲染、多语言长文本布局。
4. **手柄输入**：确认 GameInput 如何转 iced 事件（`ui/ice/winit.rs` 的键盘/鼠标路径），菜单对手柄的支持现状需如实记录为风险项。
5. 验收：缺口 widget 全部合入；`cargo test -p lemoncraft-voxygen --locked`、clippy 绿；游戏可玩性无回退。

### Phase 1 — 首个端到端（定模式）

- 迁最小模块：`change_notification`（78 行）→ `popup`（225）→ `overitem`（244）。
- 建立 `HudMsg` 枚举 + 每模块 `fn view(&self) -> Element<'_, HudMsg>` 模式（镜像菜单 `Menu`/message 模式）。
- 与 conrod HUD 并存：session 同时维护 `Ui`（conrod）与 `IcedUi`，已迁模块走 iced。
- 验收：三个轻量模块在游戏中可见、可交互；截图对比无视觉回退；主分支可玩。

### Phase 2 — HUD 主体（按依赖序，每模块独立合入）

1. hotbar / skillbar（Slot、冷却、按键提示）
2. bag / slot_grid / slots（**拖拽最难，Phase 0 拖拽原语验收后动工**）
3. crafting（Slot + 富 tooltip 最重）
4. map / minimap（image Rotation、target 标记）
5. chat（RichText、text_input、i18n）
6. buffs / loot_scroller / quest / tutorial / subtitles / buttons
7. group / social / trade
8. diary / esc_menu / settings_window
9. controller_icons / item_imgs / util 收尾

验收：每模块合入时截图为证（默认/hover/pressed/disabled/长文本/缩放状态，见 `docs/ui-qa-checklist.md` 模板）；`cargo fmt --all`、clippy `-D warnings`、锁定依赖编译通过。

### Phase 3 — 世界空间元素

- overhead（nametag/名字）、HpFloater/伤害浮字、准星、目标标记、交互提示。
- 渲染器层投影 + 视锥剔除，不占 iced 布局树。
- 验收：头顶名字与浮字在第三人称/第一人称、镜头移动、远距剔除下表现与 conrod 一致。

### Phase 4 — 拆除 conrod

- 删除：conrod 依赖、`ui/mod.rs`、`ui/widgets/`、`ui/cache.rs`（conrod 侧）、`fonts.rs` 的 conrod 通道、`to_conrod`、conrod 专用 proc macro（`#[conrod(common_builder)]`）。
- 保留共享：`theme.rs`（只留 iced/vek 通道）、`scale.rs`、`graphic/`、`img_ids`。
- 更新：`AGENTS.md` 架构段落、`ui-development-plan.md` 长期计划段、QA 清单（HUD iced 专项）。
- 验收：`cargo tree` 无 conrod；全量 clippy/test/fmt 绿；HUD 主流程（战斗、拾取、背包、制作、聊天、设置、地图、任务）实机走查一遍。

---

## 风险与对策

| 风险 | 对策 |
|------|------|
| iced fork 古老（iced_native 0.4 时代，winit 0.28）：无内置拖拽、布局能力有限 | 缺口原语 Phase 0 全部自建并测试；不升级 iced 版本（升级是新工程，另行立项） |
| 世界投影与 iced 缓存交互（primitive 携带世界坐标） | 投影在 renderer draw 阶段，widget 层只传占位 rect + 世界坐标，与 `Ingame` 行为对齐 |
| 拖拽手感与 conrod 不一致（HUD 高频操作） | Phase 0 拖拽原语单测覆盖连续 hover/drop 路径；实机对比 |
| 手柄输入在 iced 侧缺失 | **P0-9 已确认**：`ui/ice/winit.rs` 的 `window_event` 只转换鼠标/键盘/触摸，无手柄事件；手柄走 `game_input` → HUD 模块直接轮询（conrod 侧同样如此）。迁移后 HUD 模块继续直接读 `GameInput` 状态，或由 session 将手柄动作合成 iced 事件（Phase 2 按需定） |
| 性能（HUD 每帧重建 Element） | 菜单已证 iced 缓存可用；HUD 高频更新面（HP 条）在 Phase 2 专项验证 |
| 24k 行分 10+ 次合入期间的视觉漂移 | 每模块合入带截图；QA 清单逐项勾选 |
| 字体/中文渲染 | **P0-8 已确认**：`fonts.rs` 已有 `IcedFonts` 通道（universal/alkhemi/cyri → `IcedUi::add_font`），`test_font_manifests` 已把全部语言字体（含 zh-Hans Alibaba PuHuiTi）加载进 glyph_brush 验证；语言切换走 `clear_fonts`。渲染侧中文已在菜单生产使用 |

---

## DoD（全计划完成定义）

1. `cargo tree` 无 conrod 依赖；`ui/mod.rs` 与 `ui/widgets/` 删除。
2. HUD 全部模块为 `Element<HudMsg>`，与菜单同一 widget 体系、同一 `theme.rs` 单源。
3. 世界空间元素（头顶名/浮字/目标标记）经 iced 渲染器投影，行为与迁移前一致。
4. 全量验证：`cargo fmt --all`、两条 clippy 线、`cargo test` 绿。
5. 实机走查：战斗、拾取、背包拖拽、制作、聊天、设置、地图、任务无功能回退。
6. 文档同步：`AGENTS.md`、`ui-development-plan.md`、QA 清单更新完成。

---

## 进度日志

| 日期 | 事项 |
|------|------|
| 2026-08-11 | 计划落盘；Phase 0 未启动 |
| 2026-08-12 | **Phase 0 交付（一次性）**：渲染器 WorldPos primitive + `view_projection_mat` 线程（3 处菜单调用点更新）；新 widget 全量落位（slot/drag/item_tooltip/rich_text/outlined_text/image_frame/toggle_button/radio_list/image_slider/world_anchor，共 10 个新模块 + 单测）；P0-8/P0-9 调查结论写入风险表 |
| | **P0 备注**：`server/src/sys/metrics.rs`（用户 WIP）`#[cfg]` 置于 `use` 组/元组内非法语法，最小修复（该资源本就无条件注册，cfg 多余） |
| | **P0 待办**：P0-10 全量验证（fmt/clippy/test）尚未执行（编码阶段按用户要求禁止 cargo check） |
