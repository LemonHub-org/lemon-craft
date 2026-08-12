# LemonCraft 高杠杆优化选项

> 状态：静态审计建议  
> 日期：2026-08-11  
> 范围：服务器主循环与 ECS、物理、AI、网络同步、客户端渲染、地形网格和构建效率

## 1. 结论

当前最值得优先投入的方向不是零散减少 `clone`、手写 SIMD 或局部数学微调，而是减少“每 tick / 每帧全量执行”的工作，并把空间查询、GPU 上传和网络消息改为增量或批处理。

建议按以下顺序推进：

1. 消除无效观测开销、拆除服务器上的客户端插值系统。
2. 建立可重复的端到端性能基线。
3. 复用粒子 GPU 缓冲，使用已有 Blocks of Interest 加速地形网格生成。
4. 将区域维护、实体清理和物理空间索引增量化。
5. 对 NPC AI 分级降频，并批量化实体同步协议。
6. 精简客户端默认特性和遗留 UI 依赖，改善开发循环。

本报告来自静态结构审计。由于游戏资产仍是未获取的 Git LFS 占位符，本轮没有运行完整客户端场景，因此收益等级是根据执行频率、算法结构和资源生命周期判断的，不代表已测得的加速比。

## 2. 优先级总览

| 优先级 | 优化项 | 主要目标 | 预期收益 | 成本 | 风险 |
| --- | --- | --- | --- | --- | --- |
| P0 | 禁止非 Tracy 构建求值 `plot!` 参数 | 服务器 CPU | 中高 | 很低 | 很低 |
| P0 | 服务器不注册插值系统 | ECS 关键路径 | 中 | 很低 | 很低 |
| P0 | ECS/Prometheus 指标降采样 | 服务器 CPU | 中 | 低 | 低 |
| P0 | 建立场景化性能基线 | 优化可信度 | 间接但很高 | 中 | 很低 |
| P1 | RegionMap 与实体清理增量化 | 大实体量扩展性 | 高 | 中 | 中 |
| P1 | 物理空间索引持久化与复用 | 物理吞吐 | 很高 | 中高 | 中 |
| P1 | NPC AI 分级降频 | NPC 数量上限 | 很高 | 中高 | 中高 |
| P1 | 粒子 GPU 缓冲复用与发射 LOD | 客户端帧时间 | 很高 | 中 | 中 |
| P1 | 实体同步消息批处理 | 多人及高视距 | 高 | 中高 | 中高 |
| P2 | BlocksOfInterest 加速地形网格 | 区块流送卡顿 | 中高 | 中 | 中 |
| P2 | UI 与 Cargo 特性瘦身 | 编译时间、磁盘 | 很高 | 中高 | 中 |

## 3. P0：低风险先手收益

### 3.1 禁止非 Tracy 构建执行观测表达式

证据：

- `common/base/src/lib.rs` 中禁用 Tracy 时的 `plot!` 宏仍通过 `let _: f64 = $value` 求值参数。
- `server/src/sys/metrics.rs` 每 tick 调用该宏统计实体、待生成区块和已加载区块。
- 实体数量虽然已经每 100 tick 更新一次 Prometheus gauge，却又被 `plot!` 每 tick 全量统计一次。

建议：

- 禁用 Tracy 时，只进行不求值的类型检查；或者将调用整体放进编译期 Tracy 条件。
- 避免用运行时 `if false` 掩盖昂贵表达式，优先保证 release 和 dev 构建都不会执行它。
- 添加单元测试或小型计数器测试，确认禁用宏不会触发带副作用的表达式。

验收：

- 非 Tracy 构建中，`plot!` 参数没有运行时调用。
- 空服和 1,000/10,000 实体场景的 `metrics` 系统耗时明显下降。
- Tracy 构建仍保留原有 plot 数据。

### 3.2 从服务器调度图中移除插值系统

证据：

- `server/src/lib.rs` 通过 `add_local_systems` 注册公共系统。
- `common/systems/src/lib.rs` 在该函数中无条件注册 `interpolation::Sys`，并已有“不要在服务器运行插值”的 TODO。
- 插值系统声明对 `Pos`、`Vel` 和 `Ori` 的写访问，同时是物理系统的显式依赖，会延长服务器调度关键路径。

建议：

- 将系统集合拆为 `add_shared_systems`、`add_client_systems` 和服务器专用集合。
- 服务器不注册插值系统，也不让物理系统依赖不存在的客户端阶段。
- 保留物理测试所需的显式测试注册函数，避免测试环境意外改变语义。

验收：

- 服务器 ECS 指标中不再出现 `interpolation`。
- 客户端远端实体平滑行为不变。
- 物理、客户端预测和服务器测试全部通过。

### 3.3 降低观测系统自身开销

证据：

- `common/ecs/src/system.rs::gen_stats` 汇集全部系统时间点，并在每个时间点再次遍历全部系统，结构上接近 O(S²)。
- `server/src/sys/metrics.rs` 每 tick 调用它，并同步更新多组 Prometheus labels、gauges、counters 和 histograms。

建议：

- 普通运行每 10～30 tick 聚合一次系统指标。
- 检测到慢 tick 或显式启用诊断模式时，临时切换到逐 tick 采样。
- 保留总 tick histogram 的逐 tick 记录，因为其成本低且用于发现尖峰。
- 评估把 `CpuTimeline` 名称改为静态键，避免每 tick 克隆和散列 `String`。

验收：

- 指标采样周期可配置且有清晰默认值。
- Prometheus 数据仍足以定位慢系统。
- 指标系统的 p95 耗时下降，慢 tick 捕获能力不退化。

## 4. 性能基线与回归门禁

现有 Criterion 基准主要覆盖 loot、颜色转换、chonk、网络协议、地形 meshing 和部分 worldgen；关键运行时路径缺少场景基准，尤其是物理、AI、RegionMap、实体同步和粒子上传。

建议增加以下固定场景：

### 4.1 服务器场景

| 场景 | 变量 | 记录指标 |
| --- | --- | --- |
| 空服基线 | Tracy 开/关、metrics 开/关 | tick p50/p95/p99、各 ECS 系统耗时 |
| 大实体量 | 1k、5k、10k 静止/移动实体 | RegionMap、physics、cleanup 耗时 |
| 密集碰撞 | 稀疏、集中、不同 collider 尺寸 | 碰撞检查数、碰撞命中率、physics p99 |
| NPC 负载 | 1、32、128、256 bots | agent 耗时、tick p99、行为响应时间 |
| 高视距同步 | 多客户端移动穿越区域 | 消息数、字节数、序列化时间、队列延迟 |

### 4.2 客户端场景

| 场景 | 变量 | 记录指标 |
| --- | --- | --- |
| 粒子压力 | 0、1k、10k、50k 粒子 | CPU maintain、GPU upload、frame p99 |
| 区块流送 | 固定路线和视距 | mesh 队列延迟、上传时间、帧尖峰 |
| 城镇/战斗 | 固定相机轨迹 | figure、lights、shadows、particles 耗时 |

### 4.3 门禁策略

- Criterion 保存基线并报告超过 5%～10% 的显著回退。
- 端到端 smoke benchmark 使用较宽阈值，重点阻止数量级回退。
- 性能结果至少记录机器、线程数、profile、feature 集合和 git revision。
- 服务器吞吐优化以 tick p99 为主，不只看平均耗时。

## 5. P1：服务器扩展性

### 5.1 RegionMap 与实体清理增量化

证据：

- `common/src/region.rs::RegionMap::tick` 遍历全部区域及其实体，即使大部分实体静止。
- 源码已经标注按速度决定检查频率、按实体 ID 错峰以及分散清理工作的 TODO。
- `server/src/lib.rs` 每 tick 另行扫描 Anchor 链和所有无 `Presence` 实体，寻找需要删除的 NPC。

建议分三步实施：

1. 对静止实体按 ID 分片，每 30～100 tick 检查一次；移动实体仍逐 tick 检查。
2. 使用 `Pos`、`Presence`、`Anchor` 和实体生命周期事件维护 dirty bitset。
3. 将区块卸载产生的实体清理直接放入删除队列，避免随后再次全局扫描。

注意事项：

- 瞬移和服务器事件直接修改位置时必须立即标脏。
- 实体删除、失去 `Pos`、切换 `sync_me` 都必须覆盖。
- 应保留低频完整校验模式，用于检测增量索引漂移。

验收：

- 10k 静止实体下 RegionMap 成本接近活动实体数量，而不是总实体数量。
- 瞬移、骑乘、区块卸载和 Presence 切换测试无回归。
- 可选完整校验在测试构建中验证索引一致性。

### 5.2 物理空间索引持久化

证据：

- `common/systems/src/phys/mod.rs` 每 tick 更新全部 `PreviousPhysCache`。
- `construct_spatial_grid` 每 tick创建新的碰撞网格，并已有持久化、预分配和并行化 TODO。
- 物理结束后又清空并重建供其他系统使用的 `CachedSpatialGrid`。

建议分层优化：

1. 先复用网格对象、HashMap 容量、单元格 Vec 和临时结果 Vec。
2. 给实体维护当前小网格/大网格 cell，只处理跨格、半径变化、新增和删除。
3. 评估碰撞查询网格与公共查询网格能否共享桶结构；若语义不同，至少共享更新事件和容量策略。
4. 添加空间网格占用、最大半径、候选数和真实碰撞数指标，用数据调整 cell size 与 radius cutoff。

验收：

- 稳态场景每 tick 不再随实体总数产生大量桶分配。
- 稀疏和密集碰撞场景结果与现有实现逐实体一致。
- physics p95/p99、分配次数和 collision checks 同时记录。

### 5.3 NPC AI 分级降频

证据：

- `server/src/sys/agent/mod.rs` 每 tick 对全部已加载 Agent 执行完整行为树。
- 每个 Agent 每次执行都会创建 RNG、事件发射器并进行目标、装备、路径和状态判断。

建议更新层级：

| 状态 | 建议频率 |
| --- | --- |
| 战斗中、受击、玩家极近 | 服务器 tick 频率 |
| 可见但非战斗 | 10 Hz |
| 远距离闲置 | 2～5 Hz |
| 关键脚本/Boss 阶段 | 显式覆盖为高频 |

控制器、物理和动画状态仍可按服务器 tick 执行；行为树使用累积 `dt`，不能简单丢弃时间。

验收：

- 128/256 NPC 场景 agent 系统时间近似按活跃 AI 数量扩展。
- 攻击响应、仇恨、逃跑、对话和路径行为保持可接受延迟。
- 固定种子回放能够解释降频引起的行为差异。

## 6. P1：客户端帧时间

### 6.1 粒子缓冲复用

证据：

- `voxygen/src/scene/particle.rs::maintain` 每帧调用所有粒子发射类别。
- `upload_particles` 每帧将全部实例重新收集为 `Vec<ParticleInstance>`，随后调用 `renderer.create_instances` 新建 GPU buffer。
- 源码已标注“优化 buffer writes”的 TODO。

建议：

- `ParticleMgr` 持有可增长的 instance buffer 和当前有效长度。
- 容量不足时按 1.5～2 倍扩容，平时使用 `queue.write_buffer` 更新有效前缀。
- 将 CPU 侧实例数组作为长期缓冲复用，避免每帧重新分配。
- heartbeat 为零时，在进入相关 ECS join 前直接返回。
- 按距离、视锥、粒子屏幕尺寸和类型设置发射预算。
- 烟雾、余烬等允许较低更新率；技能判定相关视觉效果保持高优先级。

验收：

- 稳态粒子场景不再每帧创建 GPU buffer。
- 粒子 CPU/GPU buffer 分配次数、上传字节数和丢弃数可观测。
- 10k/50k 粒子场景的 frame p99 改善，视觉密度变化受配置控制。

### 6.2 地形发光方块查询

证据：

- `voxygen/src/mesh/terrain.rs::generate_mesh` 已接收 `BlocksOfInterest`，但参数名为 `_boi`，当前没有使用。
- 网格生成会扫描完整区块及光照边界寻找发光方块，并有明确的性能 TODO。

建议：

- 使用当前区块和相邻区块的 `BlocksOfInterest::lights` 直接构造 glow sources。
- 对边界光源补充相邻区块查询，避免已有注释指出的缺口。
- 在 meshing benchmark 中加入发光方块稀疏、密集和无发光三组数据。

验收：

- 无发光/稀疏发光区块不再进行完整体素扫描。
- 新旧 glow/light map 在固定输入下逐像素或逐体素一致。
- 区块 mesh 生成 p95 和加载路线 frame p99 下降。

## 7. P1：网络同步批处理

证据：

- `server/src/sys/subscription.rs` 在客户端进入新区时，为每个实体单独发送 `CreateEntity`，源码已有批处理 TODO。
- `server/src/sys/entity_sync.rs` 可能针对一个客户端的多个订阅区域分别发送 `CompSync`，force update counter 也可能重复携带。

建议：

- 添加批量实体创建和删除消息，或扩展现有 sync package 表达初始化数据。
- 每个客户端每 tick 合并所有区域的 component updates，再进行一次 prepare/serialize。
- 保留最大消息尺寸，超过阈值时按字节预算切片，而不是按实体个数硬切。
- 为消息数、平均/最大 payload、压缩比、排队时间和丢弃数增加指标。

风险：

- 需要协议版本协商或客户端/服务器同步升级。
- 大包可能增加单次延迟，必须设置分片和发送预算。
- 不能破坏 Create/Delete 与组件更新之间的顺序语义。

验收：

- 固定高视距路线中，每 tick 消息数显著下降。
- 总字节数、序列化 CPU 和网络队列延迟不恶化。
- 乱序、重连、观战切换和区域边界往返测试通过。

## 8. P2：构建与开发效率

证据：

- `voxygen/Cargo.toml` 默认特性同时包含单人服务器/worldgen、热重载、Egui 和 `shaderc-from-source`。
- Voxygen 同时依赖 Conrod、旧 Iced 和 Egui，带来多套 UI、字体、窗口及旧版本传递依赖。
- 当前 `target/debug/build` 中观察到三个约 1.39 GB 的 shaderc 源码构建目录；它们可能来自不同 feature/profile 的历史构建，但说明该依赖对磁盘和构建缓存非常敏感。
- workspace 的 dev profile 对本地 crate 使用 O2，对第三方依赖使用 O3。

建议：

1. 增加明确的开发预设：
   - `dev-client-fast`：远程客户端、单一 UI、无 worldgen。
   - `dev-singleplayer`：完整本地服务器。
   - `dev-web`：WASM 所需最小依赖。
2. 将 `shaderc-from-source` 从普通开发默认路径移出，由环境预设或显式 alias 启用。
3. HUD 迁移完成后删除 Conrod/Iced 及其旧依赖链。
4. 增加低优化快速检查 profile；运行游戏仍使用现有 O2/no_overflow profile。
5. CI 按 crate/feature 矩阵共享缓存，避免单次任务启用所有互斥前端。

验收：

- 记录 clean build、增量修改和 link 时间。
- 记录 `target/` 大小及 shaderc 构建次数。
- 快速客户端预设不再编译 server/worldgen 和未使用 UI 栈。
- 发布和单人游戏功能不受影响。

## 9. 建议里程碑

### Milestone A：低风险清障

- 修复禁用 Tracy 时的 `plot!`。
- 拆分客户端/共享系统注册。
- ECS 指标降采样。
- 建立服务器空服和大实体量基线。

完成标准：改动具有微基准或端到端数据，并且不改变游戏语义。

### Milestone B：客户端尖峰

- 粒子 CPU/GPU 缓冲复用。
- 粒子发射预算与 heartbeat 快速路径。
- BlocksOfInterest 地形光源查询。

完成标准：固定路线和粒子压力场景的 frame p99 改善。

### Milestone C：服务器规模

- RegionMap 错峰和 dirty 更新。
- 事件驱动实体清理。
- 物理空间索引容量复用及增量维护。

完成标准：静止实体总量增长不再线性推高相应系统的每 tick 成本。

### Milestone D：人口与多人

- NPC AI 分级频率。
- 初始化和组件同步批处理。
- 协议兼容、回放和高视距压力测试。

完成标准：目标 NPC/玩家规模下 tick p99 和网络队列均在预算内。

## 10. 暂不优先的优化

除非 profiler 显示为主要热点，否则暂不优先：

- 大范围手写 SIMD。
- 全局替换哈希算法或容器。
- 为少量 `clone` 引入复杂生命周期。
- 仅优化平均帧率而不关注 p95/p99。
- 在没有场景基线时重写 ECS 或渲染架构。

这些方向可能最终有价值，但当前结构中，全量扫描、重复分配、更新频率和消息粒度具有更清晰的杠杆。

## 11. 关键代码索引

- `common/base/src/lib.rs`：Tracy/plot 宏。
- `common/ecs/src/system.rs`：ECS 系统时间线与 `gen_stats`。
- `common/systems/src/lib.rs`：公共系统注册和服务器插值 TODO。
- `common/systems/src/phys/mod.rs`：物理缓存和空间网格。
- `common/src/region.rs`：RegionMap 全量维护。
- `server/src/lib.rs`：服务器 tick、区域更新和实体清理。
- `server/src/sys/agent/mod.rs`：NPC 行为树主循环。
- `server/src/sys/subscription.rs`：进入区域时的实体初始化消息。
- `server/src/sys/entity_sync.rs`：每 tick 组件同步。
- `voxygen/src/scene/particle.rs`：粒子维护和实例上传。
- `voxygen/src/mesh/terrain.rs`：地形网格光源扫描。
- `voxygen/Cargo.toml`：客户端 UI、singleplayer 和 shaderc 特性。
- `Cargo.toml`：workspace profiles。
