# 「私屿」私人空间具体交互功能实施方案

**文档版本**：V1.0
**修订日期**：2026年04月
**状态**：实施指南
**关联文档**：@personal_space_module.md (底座设计), @player_engine_implementation.md (播放器设计)

---

## 一、 功能概述

目前本项目的底座模型、SQLite 持久化（如 `liked_artists`，`liked_albums` 和 `play_history` 关系表）以及 Dioxus 的服务层及状态流（如 `PersonalService` 和 `PersonalProvider`）均已准备就绪。然而，这部分功能尚未嵌入实际的用户视图操作及播放回调生命周期中。

此方案的执行目标是将已经完备的后方核心服务与前方的用户界面按钮、播放切歌行为彻底对接合流。

---

## 二、 核心功能实施规划

### 2.1 播放记录 (Play History) 切歌埋点捕获

需要在播放状态发生改变的时候，无感触发异步写入历史任务。

1. **调整范围：** 涉及顶层组件或播放状态逻辑，比如专门拦截播放切歌行为的服务。 
2. **实现方式：** 
   * 利用 Dioxus 钩子机制或者在底层 `PlayerProvider` 的业务代码同步更新状态流处，捕获 `player.current_track`。
   * 当检测到全新音轨对象并且发生了实际切歌，异步调用 `personal_provider.add_play_history(&track.id)`。由于 `PersonalProvider` 中写好了刷新数据的逻辑，因此会顺势调用 SQLite 保存，并同时推送数据展示至“私人空间”的最近播放列表。
3. **去重保障：** 我们可维护暂存量进行边界对比防抖拦截，避免对同一首歌曲因重复点击暂停/播放按钮带来的额外冗余入库。

### 2.2 收藏歌手 (Like Artist) 交互绑定

位于代码文件：`packages/ui/src/components/music_library/entity_detail.rs` (对应 `entity_type == "ARTIST"` 分支)。

1. **状态解析：**
   通过解析引入全局服务获取已喜欢列表，并映射布尔值表示该歌手是否被关注：
   ```rust
   let liked_artists = personal_provider.liked_artists();
   let is_artist_liked = liked_artists.contains(&artist_name);
   ```
2. **UI 侧动态交互渲染：**
   * _未关注_ 状态按钮表现：文本渲染为“关注”。
   * _已关注_ 状态按钮表现：应用样式反转设计（如黑底白字），文本渲染为“已关注”，提升视觉分辨度。
3. **闭包事件 (`onclick`)：**
   通过 `spawn(async move { ... })` ，点击时进行逆向推导，若判断当前为 `is_artist_liked` 真，则发出并等待 `unlike_artist`；反之则发送 `like_artist` 接口行为。

### 2.3 收藏专辑 (Like Album) 交互绑定

同样位于文件：`packages/ui/src/components/music_library/entity_detail.rs` (对应 `entity_type == "ALBUM"` 状态栏分支)。

1. **状态解析：**
   判断依据建立在 `album.id` 之上：
   ```rust
   let liked_albums = personal_provider.liked_albums();
   let is_album_liked = liked_albums.iter().any(|a| a.id == album.id);
   ```
2. **UI 侧心形 Icon 动态反馈渲染：**
   替换原本展示用途的单纯 Heart 按钮。
   * 当_未喜欢_ 时：选用 `LdHeart` 线框图标。
   * 当_已喜欢_ 时：可替换为高亮红色填充效果等更直接的心智对应反馈。
3. **事件注入：**
   收藏专辑操作由于存在列表视图映射，必须确保接口所需完整关联参数的封存，`like_album` 需要透传 `album_id, title, artist, cover_path` 等资源。

---

## 三、 测试与验收准则 (Verification Plan)

所有代码改造合并前，需确保满足以下四项人肉全链路联调基带测试：

| 测试节点标号 | 前置操作场景 | UI 与数据层期望反馈结果 |
| :---: | :--- | :--- |
| **TC-01** | 全新曲目发起首次点击播放 | 打开 `私人空间` 可以顺利在 `最近播放` 栏看到该记录已经排于列表顶部位置。 |
| **TC-02** | 连续快进触发多张专辑或播放列表曲目的随意乱切 | 每首播放的实际歌曲都完好地顺位加入播放记录表且防抖异常没造成堵塞。 |
| **TC-03** | 点进指定的歌手资料实体页，进行实体关注操作 | 能够返回并在 `私人空间` 首页板块见到这张被挂上对应名称与头像标签的歌手封面图卡。 |
| **TC-04** | 进入已被点赞的专辑主页面，反选取消掉红心操作 | 随着状态反转变灰为空心；退回空间之后，收藏专辑列表中这张老面孔成功抽离消失并回归空状态展现模式。 |
