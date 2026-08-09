command-adminify-desc = 临时授予玩家受限的管理员权限，或移除当前权限（若未授予）
command-alias-desc = 更改您的别名
command-area_add-desc = 新增一个建造区域
command-area_list-desc = 列出所有建造区域
command-area_remove-desc = 移除指定建造区域
command-aura-desc = 创建一个光环
command-body-desc = s将您的角色变成不同种族
command-set_body_type-desc = 设置性别：女性或男性。
command-set_body_type-not_found =
    这不是有效的体型。
    请尝试以下选项之一：
    { $options }
command-set_body_type-no_body = 无法设置体型，因为目标没有身体。
command-set_body_type-not_character = 仅能对在线玩家角色永久设置体型。
command-buff-desc = 给玩家施加增益效果
command-build-desc = 开关建筑模式
command-battlemode-desc =
    设置你的战斗模式为：
    + pvp（玩家对战）
    + pve（玩家对战环境）
    如不带参数调用，将显示当前战斗模式。
command-battlemode_force-desc = 直接更改战斗模式，无需验证
command-campfire-desc = 生成一个篝火
command-help-template = { $usage }{ $description }
command-help-list =
    { $client-commands }
    { $server-commands }

    此外，您可以使用以下快捷键:
    { $additional-shortcuts }
command-airship-desc = 生成一艘空中飞船
command-ban-desc = 根据给定的用户名，对玩家进行禁用操作，持续时间由参数指定(如果提供)。传递true以覆盖并修改现有禁令。
command-ban-ip-desc = 封禁拥有指定用户名的玩家，期限为指定时长(若已提供)。与常规封禁不同，此操作还会额外封禁与该用户关联的IP地址。传递true可覆盖选项，则可将现有封禁状态进行更改。
command-clear_persisted_terrain-desc = 清除附近已存在的地形
command-create_location-desc = 在当前位置创建一个定位
command-death_effect-dest = 为目标实体添加一个死亡时效果
command-debug_column-desc = 打印有关某列的一些调试信息
command-debug_ways-desc = 打印有关列的存储方式的调试信息
command-delete_location-desc = 删除定位
command-destroy_tethers-desc = 摧毁所有与你相连的束缚
command-disconnect_all_players-desc = 断开与服务器上连接的所有玩家
command-dismount-desc = 如果你在骑乘，请先下马，或者卸载骑在你身上的任何东西
command-dropall-desc = 把你所有的物品扔到地上
command-make_block-desc = 在你的位置生成一个具有颜色的方块
command-make_npc-desc =
    在你附近从配置中生成一个实体。
    使用 Tab 键获取示例或自动补全 。
command-dummy-desc = 生成一个训练假人
command-explosion-desc = 让地面爆炸
command-faction-desc = 向您的派系发送讯息
command-give_item-desc = 给自己一些物品，使用tab键获取示例或自动完成。
command-gizmos-desc = 管理小工具订阅。
command-gizmos_range-desc = 更改小工具订阅的范围。
command-goto-desc = 传送到某个位置
command-goto-rand = 传送到随机位置
command-group-desc = 向您的群组发送讯息
command-group_invite-desc = 邀请玩家加入群组
command-group_kick-desc = 从群组中移除玩家
command-group_leave-desc = 离开当前群组
command-group_promote-desc = 提升某玩家为群组领导者
command-health-desc = 设置您当前的生命值
command-into_npc-desc = 将自己转换为NPC，请谨慎使用!
command-join_faction-desc = 加入/离开指定的派系
command-jump-desc = 偏移您当前的位置
command-kick-desc = 踢出某个名称的玩家
command-kill-desc = 自杀
command-kill_npcs-desc = 杀死NPC
command-kit-desc = 将一组物品放入您的物品栏。
command-lantern-desc = 更改您的灯笼强度和颜色
command-light-desc = 生成具有光线的实体
command-lightning-desc = 在当前位置放出闪电
command-location-desc = 传送到某个地点
command-outcome-desc = 创建一个结果
command-permit_build-desc = 给予玩家在某范围内建造的权限
command-players-desc = 列出当前在线的玩家
command-portal-desc = 生成一个传送门
command-region-desc = 向您的区域内所有人发送讯息
command-reload_chunks-desc = 重新加载服务器上的区块
command-repair_equipment-desc = 修复所有以装备的物品
command-reset_recipes-desc = 重置您的配方书
command-respawn-desc = 传送到您的路径点
command-revoke_build-desc = 撤销玩家的建筑区域权限
command-revoke_build_all-desc = 撤销玩家所有区域的建筑权限
command-safezone-desc = 创建一个安全区域
command-say-desc = 向所有听的到的人发送讯息
command-scale-desc = 调整您的角色大小
command-server_physics-desc = 设置/取消账户的服务器物理授权
command-set_motd-desc = 设置服务器描述
command-tell-desc = 向另一个玩家发送讯息
command-tether-desc = 将另一个实体系在您身上
command-time-desc = 设置一天中的时间
command-time_scale-desc = 设置时间的缩放比例
command-make_sprite-desc = 在你的位置创建一个精灵。要定义精灵属性，请使用 RON 语法指定一个 StructureSprite。
command-make_volume-desc = 创建一个空间体积（实验性功能）
command-motd-desc = 查看服务器描述
command-mount-desc = 骑乘一个实体
command-object-desc = 生成一个物体
command-poise-desc = 设置你当前的姿态
command-remove_lights-desc = 移除所有由玩家生成的光源
command-set-waypoint-desc = 将你的航点设置为当前位置。
command-ship-desc = 生成一艘船
command-site-desc = 传送到一个地点
command-skill_point-desc = 为某个技能树分配技能点
command-skill_preset-desc = 赋予你的角色所需的技能。
command-spawn-desc = 生成一个测试实体
command-spot-desc = 查找并传送到最近的特定类型地点。
command-sudo-desc = 以另一个实体的身份运行命令

## 翻译补充：缺失的命令 key

command-tp-desc = 传送到另一个实体
command-rtsim_chunk-desc = 显示 rtsim 当前区块的信息
command-rtsim_info-desc = 显示 rtsim NPC 的信息
command-rtsim_npc-desc = 列出符合查询条件的 rtsim NPC（例如：simulated,merchant），按距离排序
command-rtsim_purge-desc = 下次启动时清除 rtsim 数据
command-rtsim_tp-desc = 传送到一个 rtsim NPC
command-unban-desc = 解除指定玩家的封禁。若存在关联的 IP 封禁也会一并解除。
command-unban-ip-desc = 仅解除指定玩家的 IP 封禁。
command-version-desc = 显示服务器版本
command-weather_zone-desc = 创建一个天气区域
command-whitelist-desc = 添加/移除白名单中的用户名
command-wiring-desc = 创建电路元件
command-world-desc = 向服务器上所有玩家发送消息
command-wiki-desc = 打开 wiki 或搜索一个主题
command-reset_tutorial-desc = 将游戏内教程重置为初始状态
command-reset_tutorial-success = 已重置教程状态。
command-naga-desc = 切换初始着色器处理中是否使用 naga（不持久化）
players-list-header = { $count ->
  [1] { $count } 名玩家在线
    { $player_list }
  *[other] { $count } 名玩家在线
    { $player_list }
}
command-clear-desc = 清空聊天中的所有消息。影响所有聊天标签页。
command-experimental_shader-desc = 切换一个实验性着色器。
command-help-desc = 显示命令的相关信息
command-mute-desc = 屏蔽某个玩家的聊天消息。
command-unmute-desc = 取消对 'mute' 命令屏蔽的玩家的屏蔽。
command-waypoint-desc = 显示当前重生点的位置
command-preprocess-target-error = '@' 后应为 { $expected_list }，实际为 { $target }
command-preprocess-not-looking-at-valid-target = 未注视有效的目标
command-preprocess-not-selected-valid-target = 未选中有效的目标
command-preprocess-not-valid-viewpoint-entity = 未从有效的视角实体进行观察
command-preprocess-not-riding-valid-entity = 未骑乘有效的实体
command-preprocess-not-valid-rider = 没有有效的骑乘者
command-preprocess-no-player-entity = 没有玩家实体
command-invalid-command-message =
  找不到名为 { $invalid-command } 的命令。
  您是想输入以下命令之一吗？
  { $most-similar-command }
  { $commands-with-same-prefix }

  输入 /help 可查看所有命令的列表。
command-mute-cannot-mute-self = 您不能屏蔽自己
command-mute-success = 已成功屏蔽 { $player }
command-mute-no-player-found = 找不到名为 { $player } 的玩家
command-mute-already-muted = { $player } 已被屏蔽
command-mute-no-player-specified = 您必须指定一个玩家
command-unmute-cannot-unmute-self = 您不能取消屏蔽自己
command-unmute-success = 已成功取消对 { $player } 的屏蔽
command-unmute-no-muted-player-found = 找不到被屏蔽的玩家 { $player }
command-unmute-no-player-specified = 您必须指定要取消屏蔽的玩家
command-shader-backend = 当前着色器后端：{ $shader-backend }
command-experimental-shaders-list = { $shader-list }
command-experimental-shaders-not-found = 没有可用的实验性着色器
command-experimental-shaders-enabled = 已启用 { $shader }
command-experimental-shaders-disabled = 已禁用 { $shader }
command-experimental-shaders-not-supported = 此游戏版本不支持 { $shader }
command-experimental-shaders-not-a-shader = { $shader } 不是实验性着色器，使用该命令并附上任意参数可查看完整列表。
command-experimental-shaders-not-valid = 您必须指定一个有效的实验性着色器；不附加任何参数使用该命令可查看实验性着色器列表。
command-no-permission = 您没有使用 '/{ $command_name }' 的权限
command-position-unavailable = 无法获取 { $target } 的位置
command-player-role-unavailable = 无法获取 { $target } 的管理员角色
command-uid-unavailable = 无法获取 { $target } 的 UID
command-area-not-found = 找不到名为 '{ $area }' 的区域
command-player-not-found = 找不到玩家 '{ $player }'！
command-player-uuid-not-found = 找不到 UUID 为 '{ $uuid }' 的玩家！
command-username-uuid-unavailable = 无法确定用户名 { $username } 对应的 UUID
command-uuid-username-unavailable = 无法确定 UUID { $uuid } 对应的用户名
command-no-sudo = 冒充别人是不礼貌的
command-entity-dead = 实体 '{ $entity }' 已死亡！
command-error-write-settings = 设置文件写入磁盘失败，但已成功应用于内存。
  错误（存储）：{ $error }
  成功（内存）：{ $message }
command-error-while-evaluating-request = 验证请求时遇到错误：{ $error }
command-give-inventory-full = 玩家背包已满。已给予 { $given ->
  [1] 仅一件
  *[other] { $given }
} 件，共 { $total } 件物品。
command-give-inventory-success = 已将 { $total } 个 { $item } 加入背包。
command-invalid-item = 无效物品：{ $item }
command-invalid-block-kind = 无效方块类型：{ $kind }
command-nof-entities-at-least = 实体数量应至少为 1
command-nof-entities-less-than = 实体数量应小于 50
command-entity-load-failed = 加载实体配置失败：{ $config }
command-spawned-entities-config = 已根据配置生成 { $n } 个实体：{ $config }
command-invalid-sprite = 无效精灵类型：{ $kind }
command-time-parse-too-large = { $n } 无效，不能超过 16 位数字。
command-time-parse-negative = { $n } 无效，不能为负数。
command-time-backwards = { $t } 早于当前时间，时间不能倒退。
command-time-invalid = { $t } 不是有效的时间。
command-time-current = 当前时间是 { $t }
command-time-unknown = 时间未知
command-rtsim-purge-perms = 必须是真正的管理员（而非临时管理员）才能清除 rtsim 数据。
command-chunk-not-loaded = 区块 { $x }, { $y } 尚未加载
command-chunk-out-of-bounds = 区块 { $x }, { $y } 超出地图边界
command-spawned-entity = 已生成实体，ID：{ $id }
command-spawned-dummy = 已生成一个训练假人
command-spawned-airship = 已生成一艘飞艇
command-spawned-campfire = 已生成一个营火
command-spawned-safezone = 已生成一个安全区域
command-volume-size-incorrect = 尺寸必须在 1 到 127 之间。
command-volume-created = 已创建一个体积
command-permit-build-given = 您现在被允许在 '{ $area }' 内建造
command-permit-build-granted = 已授予在 '{ $area }' 内建造的权限
command-revoke-build-recv = 您在 '{ $area }' 内的建造权限已被撤销
command-revoke-build = 已撤销在 '{ $area }' 内的建造权限
command-revoke-build-all = 您的建造权限已被全部撤销。
command-revoked-all-build = 所有建造权限均已撤销。
command-no-buid-perms = 您没有建造权限。
command-set-build-mode-off = 已关闭建造模式。
command-set-build-mode-on-persistent = 已开启建造模式。实验性地形持久化已启用。服务器将尝试保存更改，但不保证成功。
command-set-build-mode-on-unpersistent = 已开启建造模式。区块卸载时更改将不会被保存。
command-set_motd-message-added = 服务器每日消息已设置为 { $message }
command-set_motd-message-removed = 已移除服务器每日消息
command-set_motd-message-not-set = 此语言没有设置每日消息
command-set-waypoint-result = 重生点已设置！
command-invalid-alignment = 无效阵营：{ $alignment }
command-kit-not-enough-slots = 背包没有足够的空格
command-lantern-unequiped = 请先装备一盏提灯
command-lantern-adjusted-strength = 您调整了火焰强度。
command-lantern-adjusted-strength-color = 您调整了火焰强度与颜色。
command-explosion-power-too-high = 爆炸威力不能超过 { $power }
command-explosion-power-too-low = 爆炸威力必须大于 { $power }
command-disconnectall-confirm = 请再次运行该命令并附上第二个参数 "confirm"，以确认您确实想断开服务器上所有玩家的连接
command-invalid-skill-group = { $group } 不是技能组！
command-unknown = 未知命令
command-disabled-by-settings = 该命令已在服务器设置中禁用
command-battlemode-intown = 您需要在城镇中才能切换战斗模式！
command-battlemode-cooldown = 冷却中。请在 { $cooldown } 秒后重试
command-battlemode-available-modes = 可用模式：pvp, pve
command-battlemode-same = 尝试设置相同的战斗模式
command-battlemode-updated = 新的战斗模式：{ $battlemode }
command-buff-unknown = 未知增益：{ $buff }
command-buff-data = 增益参数 '{ $buff }' 需要附加数据
command-buff-body-unknown = 未知体型规格：{ $spec }
command-skillpreset-load-error = 加载预设时出错
command-skillpreset-broken = 技能预设已损坏
command-skillpreset-missing = 预设不存在：{ $preset }
command-location-invalid = 位置名称 '{ $location }' 无效。名称只能包含小写 ASCII 字母和下划线
command-location-duplicate = 位置 '{ $location }' 已存在，请考虑先删除它
command-location-not-found = 位置 '{ $location }' 不存在
command-location-created = 已创建位置 '{ $location }'
command-location-deleted = 已删除位置 '{ $location }'
command-locations-empty = 当前没有已保存的位置
command-locations-list = 可用位置：{ $locations }
command-weather-valid-values = 有效值为 'clear'、'rain'、'wind' 和 'storm'。
command-scale-set = 已将缩放比例设置为 { $scale }
command-repaired-items = 已修复所有已装备的物品
command-repaired-inventory_items = 已修复所有物品
command-message-group-missing = 您正在使用队伍聊天，但您不属于任何队伍。请使用 /world 或 /region 切换聊天频道。
command-tell-to-yourself = 您不能 /tell 自己。
command-transform-invalid-presence = 无法在当前状态下变形
command-aura-invalid-buff-parameters = 光环的增益参数无效
command-aura-spawn = 已为实体附加新的光环
command-aura-spawn-new-entity = 已生成新的光环
command-reloaded-chunks = 已重新加载 { $reloaded } 个区块
command-server-no-experimental-terrain-persistence = 服务器编译时未启用地形持久化
command-experimental-terrain-persistence-disabled = 实验性地形持久化已禁用
command-adminify-assign-higher-than-own = 不能授予他人高于您自己永久角色的临时角色。
command-adminify-reassign-to-above = 不能为与您同级或更高级别的人重新分配角色。
command-adminify-cannot-find-player = 找不到玩家实体！
command-adminify-already-has-role = 该玩家已拥有此角色！
command-adminify-already-has-no-role = 该玩家本来就没有角色！
command-adminify-role-downgraded = 玩家 { $player } 的角色已降级为 { $role }
command-adminify-role-upgraded = 玩家 { $player } 的角色已提升为 { $role }
command-adminify-removed-role = 已移除玩家 { $player } 的角色：{ $role }
command-ban-added = 已将 { $player } 加入封禁名单，原因：{ $reason }
command-ban-already-added = { $player } 已在封禁名单中
command-ban-ip-added = 已将 { $player } 加入常规封禁名单与 IP 封禁名单，原因：{ $reason }
command-ban-ip-queued = 已将 { $player } 加入常规封禁名单，并排入 IP 封禁队列，原因：{ $reason }
command-faction-join = 请先使用 /join_faction 加入一个阵营
command-group-join = 请先创建一个队伍
command-group_invite-invited-to-group = 已邀请 { $player } 加入队伍。
command-group_invite-invited-to-your-group = { $player } 已被邀请加入您的队伍。
command-into_npc-warning = 希望您不是在滥用这个！
command-kick-higher-role = 不能踢出角色级别高于您的玩家。
command-respawn-no-waypoint = 未设置重生点
command-site-not-found = 未找到该地点
command-sudo-higher-role = 不能对角色级别高于您的玩家使用 sudo。
command-sudo-no-permission-for-non-players = 您没有对非玩家实体使用 sudo 的权限。
command-time_scale-current = 当前时间倍率为 { $scale }。
command-time_scale-changed = 已将时间倍率设置为 { $scale }。
command-unban-successful = { $player } 已成功解封。
command-unban-ip-successful = 通过用户 "{ $player }" 关联的 IP 封禁已成功解除（该用户仍将被封禁）
command-unban-already-unbanned = { $player } 已被解封。
command-version-current = 服务器正在运行 { $version }
command-whitelist-added = 已加入白名单：{ $username }
command-whitelist-already-added = 已在白名单中：{ $username }！
command-whitelist-removed = 已移出白名单：{ $username }
command-whitelist-unlisted = 不在白名单中：{ $username }
command-whitelist-permission-denied = 没有移除用户的权限：{ $username }
command-outcome-variant_expected = 应为结果变体
command-outcome-expected_body_arg = 应为体型参数
command-outcome-expected_entity_arg = 应为实体参数
command-outcome-expected_skill_group_kind = 应为有效的 ron SkillGroupKind
command-outcome-expected_frontent_specifier = 应为前端标识符
command-outcome-expected_integer = 应为整数
command-outcome-expected_sprite_kind = 应为 SpriteKind
command-outcome-invalid_outcome = { $outcome } 不是有效的结果
command-death_effect-unknown = 未知死亡效果 { $effect }。
command-spot-spot_not_found = 在此世界中未找到任何此类小型场景。
command-spot-world_feature = 运行该命令需要启用 `worldgen` 功能。
command-cannot-send-message-hidden = 作为隐藏观察者无法发送消息。
command-destroyed-tethers = 所有绳索已摧毁！您现在自由了
command-destroyed-no-tethers = 您没有连接任何绳索
command-dismounted = 已下马
command-no-dismount = 您没有在骑乘，也没有被骑乘
command-client-has-no-socketaddr = 无法获取 { $target } 的 socket 地址（通过 mpsc 连接）
command-parse-duration-error = 无法解析时长：{ $error }
command-waypoint-result = 您当前的重生点位于 { $waypoint }；
command-waypoint-error = 找不到您的重生点。
command-player-info-unavailable = 无法获取 { $target } 的玩家信息
command-unimplemented-spawn-special = 生成特殊实体功能尚未实现
command-kit-inventory-unavailable = 无法获取背包
command-inventory-cant-fit-item = 物品无法放入背包
command-you-dont-exist = 您并不存在，因此无法使用此命令
command-entity-has-no-client = 玩家没有客户端组件：{ $target }
