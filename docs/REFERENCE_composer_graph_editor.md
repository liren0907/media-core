# REFERENCE — /composer 圖編輯器（GraphEditor）實作全覽

## 這份文檔是什麼（2026-08-05）

**as-built 參考文檔**：把模式二圖編輯器的實作細節整理成一份可查閱的全覽——檔案地圖、
資料模型、分層界線、每個子系統的規則與刻意不做的事。不是 PLAN（沒有待辦）、不是
DISCUSSION（沒有未拍板的選項）；內容以 2026-08-05 的原始碼為準，與 code 不合時以
code 為準、回頭改這份。

歷史脈絡（為什麼長成這樣、哪些判斷被推翻過）在各計畫檔：
`done/PLAN_composer_graph_editor.md`（五階段主案）、`CHECKLIST_composer_layout_residual.md`、
`docs/dev/done/PLAN_node_knobs.md`、`done/PLAN_node_dictionary.md`、
`done/01_PLAN_composer_autosave.md`、`PLAN_room_graph_runtime.md`（執行面）。

---

## 1. 一句話定位

**`/composer` 的圖編輯器（`GraphEditor.svelte`）是模式二 GraphRecipe 的原生編輯表面**：
node + edge 的 DAG 畫布，畫的圖以圖的樣子存進後端 `graph_recipes` 表，`/console` 選了
它就照圖開錄（`rooms/graph.rs`）。它與模式一的 lane 組裝面（`ComposerCanvasPanel` ＋
`ComposerControlPanel`）**並存於同一頁、共用方法庫、互不寫入**——模式一那組全程零 diff
是本案鐵律，圖編輯器只透過 lower／lift 兩支純函式與 lane 世界互通（匯出／匯入）。

命名沿革：本檔一路叫「沙盒」（`ComposerCanvasSandbox.svelte`），Phase 1-A 改名
`GraphEditor.svelte`；**板面 block key 仍是 `composer.sandbox`**——那個字串存在使用者
的版面 localStorage 裡，改它會讓調過版面的人少一塊面板。

## 2. 檔案地圖

### 前端（`src/lib/components/views/lab/composer/`）

| 檔案 | 行數級 | 角色 |
|---|---|---|
| `GraphEditor.svelte` | ~2360 | 圖編輯器本體：四個邊（工具列／畫布／右欄檢視器／狀態列）＋所有**畫面狀態**（picker、selected、pan/zoom、收合、搜尋、浮層、自動存檔計時器） |
| `graph-controller.svelte.ts` | ~960 | **模型層**（Phase 0 整套搬出）：nodes/edges/params、四種生長＋移除縫合、型別守衛 `optionsAt`、常駐驗證 `issues`、存／載／自動存檔規則、開著的文件模型 |
| `graph-recipe.ts` | ~210 | 畫布圖 ↔ wire 圖的翻譯（同構、不壓縮）；`toWireGraph`／`fromWireGraph`／`methodLabelOf`／`graphNodeLabel`／`nextSeq` |
| `lower-graph.ts` | ~190 | 圖 → 模式一 lane 清單（lossy，匯出路徑）；畫布節點型別 `GraphNode`／`GraphEdge`／`NodeKind` 的家 |
| `lift-graph.ts` | ~160 | lane 清單 → 圖（匯入路徑；五類整份拒載） |
| `ComposerSavedPanel.svelte` | ~420 | 已保存清單：一顆 `.seg` 切模式一（`WorkflowRecipe`）／模式二（`GraphRecipe`）兩張表；`onLoad` 注入式載入 |
| `ComposerCanvasPanel.svelte` | ~850 | **模式一** lane 畫布（不是圖編輯器；零 diff 保留） |
| `ComposerControlPanel.svelte` | ~350 | 模式一控制台 |
| `ComposerRunPanel.svelte` | ~200 | 模式一批次 DAG 執行面（SSE 進度） |

### 前端（`graph/` 子目錄——圖編輯器自己的純函式與子元件）

| 檔案 | 行數級 | 角色 |
|---|---|---|
| `graph/layout.ts` | ~235 | **座標引擎**（Phase 2-1）：純函式 `layoutGraph` 算出每顆 x/y/w/h ＋ bezier 邊幾何；卡片尺寸常數的唯一真相 |
| `graph/layers.ts` | ~41 | 拓樸分層 `computeLayers`（memo DFS ＋ 環守衛退 0）；controller 與收合視圖共用 |
| `graph/collapse.ts` | ~145 | 群組收合（Phase 3-3）：`chainLengths`／`collapseView`——純顯示，不進存檔 |
| `graph/NodeInspector.svelte` | ~290 | 右欄檢視器：選中節點的設定（模型大小／修正鏈／窗口／字典／旋鈕） |
| `graph/node-settings.ts` | ~82 | 「這顆有哪些設定」的純判準（問埠型別與能力宣告，不問 method id）＋卡面摘要 `summaryOf` |
| `graph/MethodShelfOverlay.svelte` | ~190 | ⌘K 方法目錄浮層（browse-only，不判 verdict） |
| `graph/method-shelf.ts` | ~100 | 方法庫分區純函式 `shelfSections`／`filterSections`（從 `ComposerMethodLibrary` 抄一份、刻意不共用） |

### 前端（周邊）

| 檔案 | 角色 |
|---|---|
| `src/routes/composer/+page.svelte` | `/composer` 頁：subset board（四塊：`composer.control`／`canvas`／`sandbox`／`run`，預設顯示三塊，元件選盤增減）；只起 lab controller，不是 11-controller catalog board |
| `src/lib/controllers/lab.svelte.ts` | `graphs` 清單的唯一擁有者：`refreshGraphs`／`applyGraph`（就地換一列，不重抓全表）／`renameGraph`／`removeGraph`；**刻意沒有 load-into-canvas**（畫布狀態是沙盒私有） |
| `src/lib/api.ts`（graph recipe 鏡像段） | `GraphNodeKind`／`GraphNodeParams`／`GraphRecipeNode`／`GraphRecipeEdge`／`GraphRecipe` 型別 ＋ `getGraphRecipes`／`createGraphRecipe`／`updateGraphRecipe`／`deleteGraphRecipe` |
| `src/lib/components/views/lab/kit/MethodKnobs.svelte` | registry-driven 旋鈕原子（只畫方法宣告的那幾顆；值＋callback 進出，誰的旋鈕由呼叫端決定）——三個消費者：NodeInspector、ComposerCanvasPanel、PipelinePanel |

### 後端

| 檔案 | 角色 |
|---|---|
| `src-tauri/crates/pvdt-core/src/graph_recipe.rs` | 儲存格式與驗證器：`GraphRecipe`／`GraphNode`／`GraphEdge`／`NodeParams`、`source_outputs`／`sink_input`（圖自己的埠表，不 import `Runner`）、`topo_order`（pub，runtime 同一份 spawn 序）、`validate_graph_integrity`（寫入閘）／`validate_graph`（讀取判官，G0–G6） |
| `src-tauri/crates/pvdt-server/src/api/domain/graph_recipes.rs` | HTTP CRUD：`GET/POST /api/dev/lab/graph-recipes`、`PATCH/DELETE /api/dev/lab/graph-recipes/{id}`；`view()` 讀取時重驗 → `valid`/`reason`/`warnings` |
| `src-tauri/crates/pvdt-core/src/store/pipeline.rs` | Store CRUD：`create_graph_recipe`／`graph_recipes`（`ORDER BY created_at DESC`）／`graph_recipe`／`update_graph_recipe`／`delete_graph_recipe` |
| `src-tauri/crates/pvdt-core/src/store/types.rs` | `graph_recipes` 表（schema **v17** 加入；純加法，`workflow_recipes` 原樣不動） |
| `src-tauri/crates/pvdt-server/src/api/rooms/graph.rs` | **執行面消費者**（模式二 runtime）：`room_graph_plan`（先 `validate_graph` 再能力表）、`RoomGraphPlan`、`GRAPH_SLOT` marker |
| `src-tauri/crates/pvdt-server/src/api/console/pipeline.rs` | `/console` 執行模式選單的後端：每張圖跑一次 `room_graph_plan`（「列得出來 ⇒ 選得下去」）、`select_graph` PATCH 前重驗 |

## 3. 資料模型——三層一條線

```
畫布模型（lower-graph.ts）      wire / 後端（graph_recipe.rs ↔ api.ts 鏡像）
GraphNode{id,kind,ref,label}    GraphRecipeNode{node_id,kind,node_ref,params?}
  kind: 'modality'|'method'|      kind: 'source'|'method'|'sink'
        'sink'                    （開放字串，不用 Rust enum——新種類零 stored-shape break）
GraphEdge{from,to}              GraphRecipeEdge{from,to}
params 另放一張 NodeParamsMap    params 內嵌在節點上（Option<NodeParams>）
```

- **kind 翻譯只差一個字**：畫布叫 `'modality'`、wire 叫 `'source'`。翻譯層是
  `graph-recipe.ts`（`toWireKind`／`toCanvasKind`）。不直接改畫布型別的理由：
  `GraphNode` 住在 `lower-graph.ts`，那是模式一降級路徑的一部分，零 diff 鐵律。
- **label 不上線**：wire 節點沒有顯示名，讀回來從 `node_ref` 反查方法庫
  （`methodLabelOf`：有家族寫家族名——與插入時同一套規則，round-trip 卡面才不變）。
  方法改名後圖自動跟上；round-trip 保證的是**節點與邊等價**，不是位元組等價。
- **params 在畫布側刻意不塞進 `GraphNode`**：旁邊擺一張 `NodeParamsMap`
  （node id → `GraphNodeParams`），`lower-graph.ts` 才能全程不動。
- **`edges` 是清單不是集合，順序有意義**：fan-in 節點的第 k 條入邊餵它的第 k 個輸入
  （`MethodMeta::inputs`）。`toWireGraph` 因此不排序、不去重。
- **id**：畫布節點 `n0`…`n{k}`（`mint` 遞增；`nextSeq` 取「最大 n{k}＋1」而非節點數，
  因為刪過節點的圖不連號）；後端圖 id 是 `grf_` 前綴 ULID（`GraphRecipeId`）。

### `NodeParams` 七欄位（具名弱型別，零 bump 契約）

每欄 `Option<T>` ＋ serde `default`＋`skip_serializing_if`：**缺欄位＝沒設**，加欄位
免 schema bump。前端 `hasParams` 對應地把「全空／空陣列」的 params 整個不上線——
沒有參數的圖存出來一個位元組都不差。

| 欄位 | 誰讀 | 語意 |
|---|---|---|
| `window_lines: u32` | 收集器 | 攢滿幾句往下丟 |
| `window_ms: u64` | 收集器 | 攢滿幾毫秒往下丟（與上者先到先算） |
| `chain_pipeline: String` | 修正節點 | 指向**已存** Tier-2 pipeline（`pl_…`）——指過去不抄進來；被指的鏈刪了＝執行時降級直通，不是圖的錯 |
| `dictionaries: Vec<String>` | `dict_vocab` 方法 | 熱詞字典 id 清單；**空／缺＝用房間自己那份**；存 id 不存內容（同 `chain_pipeline` 的理由） |
| `top_k: u32` | 宣告 `top_k` 的方法 | LLM 校正一次送幾筆詞彙（0＝不設上限）；**缺＝manifest 預設** |
| `window: u32` | 宣告 `window` 的方法 | LLM 一次吃幾句（0＝整段）——**跟 `window_lines/_ms` 是兩件事**，永不同節點；欄位名照 knob id 原樣，免翻譯表 |
| `threshold: f64` | 宣告 `threshold` 的方法 | 模糊匹配門檻 0..1 |

配套存取器：`dictionary_ids()`（trim＋去重）、`opts_over(defaults)`（**疊不取代**；
三顆全沒設回 `None`，呼叫端走與欄位出現前逐位元組相同的路）、`is_empty()`。
**`NodeParams` 沒有 `Eq`**（`threshold` 浮點，`NaN != NaN`），連帶 `GraphNode`／
`PlannedNode`／`RoomGraphPlan` 都只有 `PartialEq`。

`GraphNode::params()` 是唯一准許的讀法（`None` 與全空讀起來一樣），不准讀原始欄位。

### 前端翻譯層的守門（`graph-recipe.ts`）

- `KNOWN_PARAMS` 白名單：wire 帶了畫布這版不認得的參數欄位 ⇒ **整份拒載**（載進來
  再存回去會抹掉它＝沉默資料損失）。後端加欄位時前端會誠實擋一次。
- 不認得的 `kind` ⇒ 整份拒載。空的圖**不是**拒載理由（自動存檔時代「刪光節點」是
  合法編輯狀態，必須載得回來）。

## 4. 分層界線（誰住哪裡）

貫穿全案的一條線：**規則住 controller／純函式，畫面住元件**。

- `GraphEditor.svelte` 只留畫面狀態：`picker`（選單開在哪一格）、`selected`（右欄畫誰）、
  `collapsedHeads`（收合）、pan/zoom/fit、搜尋 `query`、三張浮層開關（開啟▾／⌘K／儲存▾）、
  右欄寬度（可拖、localStorage `pvdt/composer-side:v1`）、小地圖開關
  （`pvdt/composer-minimap:v1`，**預設開**所以判 `!== '0'`）、自動存檔**計時器**。
- `graph-controller.svelte.ts` 是模型：生長／移除簽名**刻意不提 picker**
  （`insertAtNode`／`insertOnEdge`／`removeNode` 是純圖操作）；`optionsAt(point)` 收斂成
  一支函式（鏈頭 picker 是畫面狀態，不搬進來）；載入函式**回傳成功與否**，元件在唯一
  呼叫點收畫面（`doLoad`／`doNew` 收 picker/selected/收合/搜尋）。
- 純函式家族各管一段：分層（`layers.ts`＝圖的性質）與排版（`layout.ts`＝擺哪裡）是
  兩支檔、兩條界線；收合（`collapse.ts`）畫另一張圖；翻譯（`graph-recipe.ts`）只做形狀
  轉換不驗證。
- repo 慣例同步遵守：`controllers/`／`stores/` 全域零 `$effect`——計時器與生命週期
  （`$effect`＋`onDestroy`）全在元件。

## 5. 圖的生長模型（沒有手動拉線）

邊不是拖出來的，是**由插入動作生成的**。四種節點生長（`Dir`）＋邊上 split：

| 入口 | 語意 | 邊變換 |
|---|---|---|
| `root` | 空畫布放第一顆（`nodeId=null`） | 無邊 |
| `after`（後＋） | 串接插入下游 | X 的每條出邊 X→S 改 N→S，加 X→N |
| `before`（前＋） | 串接插入上游 | X 的每條入邊 P→X 改 P→N，加 N→X |
| `below`（下＋） | 平行分支（吃同一份輸入） | X 的每個上游 P 加 P→N；X 無上游則 N 是新來源 |
| 邊上＋ | 插在兩顆之間 | 移除 from→to，接 from→N→to |

- 前／後＋**只在自由端**出現（有鄰居那側交給邊上＋）；模態不長前＋（來源沒有輸入）、
  出口不長後＋（資料到此離開圖）；出口保留下＋＝**開第二個出口**的動作。
- `removeNode` 是邊上插入的反操作：上游 P × 下游 S 縫成 P→S（去重、不自環），
  順手丟掉該節點的 params（id 會被重用，留著會悄悄套到別顆上）。
- **四種生長都保持入度 ≤ 1 ⇒ 圖是一片森林**。fan-in（裁決 `arbitrate-*`）型別上
  接得上但畫布長不出來——選單把多輸入方法上架成**停用列**（`blocked: '尚未支援匯流'`），
  不藏（藏起來沒人記得放回來；匯流做完刪掉那個分支就自動回到正常列）。
- `swapModel(id, memberId)` 換同家族模型尺寸＝只改 `ref`，label 保持家族名。

## 6. 型別守衛與驗證（三道，一道比一道嚴）

### 6a. picker 過濾（預防）——`optionsAt`

- `slotOf(point)` 把五個入口收斂成 `{up, down}` 一個問題。
- `availAt`：每顆節點「跑到它為止」可用的埠型別集合——**累加、不消耗**（memo DFS＋
  環守衛）。模態的供給＝`SOURCE_PORTS` 該 port 各臂起點的**聯集**（寬鬆側；逐臂定論
  在後端）。
- 三區規則：模態只上架在「新節點不會有入邊」的格子；method 要 `availHere` 含它的
  input、且插在中間時下游要吃的埠留得住；出口只在「不會有下游」的格子且上游供得出
  它收的埠。
- **排序**：接得上直接上游（`supplyOf` 那顆自己交出的埠）的排前面，只靠更上游埠才
  合格的排後面，各自維持 manifest 序——即時圖中段三種埠同時可用會一次上架 17 項，
  不排序的話常用的字幕窗口寫回會摺到看不見。排序不動篩選。
- 家族（`m.group`）收合成一項、ref＝首位合格成員真 id；兩區皆空給誠實理由
  （`emptyReason`）。

### 6b. 常駐圖驗證（不變量）——`issues`

圖一變就重算（`$derived`），所以**不經過 picker 的變動**（移除縫合等）造出的壞圖
一樣抓得到。三張表：

- `byNode`：卡上 ⚠（壞邊的理由掛在**下游**那顆——「餵給我的線不對」是它的問題）；
- `byEdge`：該畫虛線的邊，key＝`edgeKey(from,to)`＝`` `${from}→${to}` ``（**字串格式
  住 controller、兩邊共用**——各拼各的抓不到，症狀是虛線默默不出現）；
- `edgeOf`：⚠ 清單那列→是哪條邊（3-2 導航用）。

判定用**嚴格 wire 語意**：只看直接上游 `supplyOf`（模態＝起點埠、method＝輸出、
sink＝空），不看祖先。**比 picker 的累加式嚴，落差是刻意的**：累加式允許「跳過中間
直接吃更上游的埠」，但後端執行期是單值序列，那種圖跑起來會炸——執行期對，所以畫面
誠實標 ⚠。方法庫還沒載入時**不下判斷**（否則一瞬間滿畫布假警報）。

「還沒接出口」（`openEnds`）**刻意不進 issues**——不是壞圖、是還沒畫完；狀態列一行
安靜提示。

### 6c. 後端驗證（裁判）——`graph_recipe.rs`

**兩層不是兩份**（`validate_graph` 先呼叫 `validate_graph_integrity` 再往下）：

- **`validate_graph_integrity`＝寫入閘**，只問「這份資料讀不讀得回來」：node_id 唯一
  非空、node_ref 非空、kind 認得、邊指向存在節點、不自環、不重複。**刻意不查**
  node_ref 在不在方法庫（那是引用層——方法庫刪了一支 method，圖也要能存「換掉那顆」
  的修法）。
- **`validate_graph`＝讀取判官**（G 系列，刻意不叫模式一的 R——同名不同物）：
  - G0 引用層（來源表／方法庫／出口表查得到）
  - G1 邊完整性【integrity 層】
  - G2 無環（`topo_order` 排不出來＝有環；Kahn、`edges` 序穩定、`pub` 給 runtime 共用）
  - G3 埠型別接得上（嚴格 wire；method 入邊數必須**等於** arity；sink 允許多條同型別
    入邊＝多條鏈寫同一面牆）
  - G4 source 入度 0／sink 出度 0
  - G5 無孤兒節點
  - G6 **能力回報（只警告不擋）**：模態混用（`source_runner`／`sink_runner` 的
    即時／批次表）、一張圖兩支麥克風。`GraphVerdict{warnings}`。
  - 參數層：指定了字典但方法不吃字典（`method_meta(..).dict_vocab`）⇒ Err；設了未
    宣告的旋鈕（`MethodMeta::params`）⇒ Err。**判準問能力宣告不問 method id**——
    新方法零改動。
- **G2–G5 不是存檔的閘**（自動存檔案拍板）：畫到一半存得下去；「跑不動」由四個讀取
  面講——`view()` 的 `valid`/`reason`、已保存清單的 ✗、`/console` 選單 disabled、
  `rooms::graph` 開錄前重驗。

## 7. 儲存體系

### 7a. 開著的文件模型（Phase 1-A）

- `openDoc: {id,label}|null`＝現在開著哪一份已保存的圖（null＝新畫的或 workflow 匯入，
  存檔只能是新增）。
- `baseline`＝載入／存檔成功那一刻整份內容的序列化字串；`current`＝
  `JSON.stringify({label, ...toWireGraph(...)})`（決定性、不排序）。
- `dirty`＝**逐欄比對**（`current !== baseline`；沒基準線時＝畫布上有東西）——畫一顆
  再刪掉回到原樣就不髒；名稱欄也算內容。
- `openGone`＝開著那份已從 `lab.graphRecipes` 消失（清單住 lab、寫入後都會重抓所以
  看得見；清單過期也不出事——後端 404 原樣浮上來）。
- 儲存＝更新開著那份（`updateGraphRecipe`）；`saveAsNew`＝一律新增、**存完開著的換成
  新那張**（否則按兩次另存會得到三張）；`newGraph()`＝回到剛進頁面的狀態（1-A 修了
  「存檔長重複」、它補鏡像面「沒有離開」）。

### 7b. 自動存檔（Phase 1）

- 「儲存」鈕**已消失**；工具列剩「另存新檔」（唯一開分岔的路）＋儲存▾ 裡的
  「匯出成 workflow」。
- 計時器在元件：`$effect` 依賴 `g.snapshot`（內容指紋）**而不是 `g.dirty`**——連打字
  時 dirty 恆 true 不會重排計時器，存下去的會是第一個字那一刻的內容。停手
  `AUTOSAVE_MS = 1000` 毫秒後叫 `g.autosave()`。
- `autosave()` 三條守衛：①開著那份被刪 ⇒ **停下來不自動新增**（自動另存＝把使用者
  剛刪的圖復活）；②失敗就停（admin tier token 會過期，無聲重試只會刷 401）；
  ③沒改動／正在存就跳過。停下來的理由進 `autosaveStop`，只有**手動**存檔（名稱欄
  Enter）或換文件會重新掛上。
- 空畫布不會自動長出圖（沒基準線時 dirty＝「畫布上有東西」，第一顆節點落下才第一次
  寫入）。沒取名用 `UNTITLED = '未命名的圖'`（後端仍要求 label 非空）。
- `onDestroy` 沖一次最後的改動；**關整個分頁仍會掉最後一秒**（`beforeunload` 的
  非同步 PATCH 不保證送出、`sendBeacon` 不划算）——記過帳的已知代價。
- `persist()` 成功後 `lab.applyGraph(saved)` **就地換清單那一列，不重抓全表**
  （自動存檔一秒一次，每次 GET 全表是效能債）；成功訊息帶 `· HH:MM` 時間戳
  （報事件不是時鐘，不 tick）。
- 覆蓋確認：載入／新建**只在 `g.dirty` 時**跳 `ConfirmModal`（防呆不打字），乾淨畫布
  直接做。確認框的第二句話**跟著 `autosaveStop` 換**（正常＝「等一秒自動存檔補上」；
  停了＝「用另存新檔」——兩種都不能叫人按那顆已經不存在的儲存鈕）。

### 7c. 後端 CRUD 契約

- 端點（dev tier）：`GET/POST /api/dev/lab/graph-recipes`、
  `PATCH/DELETE /api/dev/lab/graph-recipes/{id}`。
- 契約沿用 pipelines／workflow_recipes 那套低風險形：**last-write-wins、無樂觀鎖**、
  snake_case、400 帶驗證器中文原句（前端 `Error.message` 原樣顯示，後端才是裁判）。
- **寫入只驗 integrity**；讀取 `view()` 每次重跑 `validate_graph`（方法庫可能改過）→
  `valid`/`reason`/`warnings` 誠實降級，不用外鍵擋刪除。
- PATCH 的 `nodes`/`edges` 是**整份取代**（畫布一律送整張圖）。
- **DELETE 的唯一守衛**：被 `/console` 選著的圖（`console_selection(GRAPH_SLOT)`）
  回 409——marker 指空之後 `/console` 會壞在「看不出是壞掉」的地方（選單值還在、
  清單沒那個 id ⇒ 整顆選單空白）。改圖不擋（跑不動是看得懂的降級）。
- Store：`graph_recipes` 表（schema v17 純加法）＋ `store/pipeline.rs` 五支 CRUD。

## 8. 座標與渲染（Phase 2）

### 8a. 佈局引擎 `layout.ts`

**先算座標、照數字擺**，不量 DOM——2-1 整組拆掉 `measure()`／`nodeEls`／`graphSig`／
ResizeObserver。前提是**卡片尺寸定死**：`NODE_W = 200`；高度只有兩種
（`NODE_H = 28` 純標頭、`NODE_H_SUM = 45` 帶一行摘要），由常數算出、元件以 inline
style 寫進卡片——**這支檔是尺寸唯一真相**，CSS 沒有第二份數字。`LAYER_GAP = 40`、
`ROW_GAP = 28`、`PAD = 26`（外圈的＋浮在卡片外，貼邊會被裁）。

指派規則：層 → x（`computeLayers` 拓樸深度）；y **跟著上游走**——每顆算
`anchor`（上游中心的平均，排序用）與 `want`（讓自己中心對上 anchor，擺放用），同層
照 anchor 由上往下推開避免重疊。`anchor`／`want` **必須分開**：want 含自己半高，
高矮不同的兄弟會不同分，排序會把分岔上下對調（真踩過：設個參數多行摘要 ⇒ 兩支分岔
互換位置）。對**中心**不對上緣（兩種卡高，對上緣會畫出微斜的線）。取整避免 .5px 糊字。

`routeEdges`：出邊錨點沿上游卡**右緣**均分、入邊沿下游卡**左緣**均分，各照對面中心
排序（fan-out/fan-in 不自己交叉）；bezier 控制點水平拉出（`dx = max(10, |Δx|·0.4)`）；
回傳邊中點 `mx/my` 給邊上＋定位。

**座標不進 wire**：存結構不存位置，每次載入重排（決定性——同一份圖每次畫一樣）。
已知未做：減少交叉（同層序只由 anchor 決定，大圖仍會義大利麵；介面留乾淨——呼叫端
只拿 `{boxes, edges, width, height}`，換演算法零改動）。

### 8b. 視角（pan/zoom/fit）

- 畫布＝`overflow:hidden` 的洞（`.ed-canvas`）＋被 transform 的座標平面（`.ed-pan`）；
  卡片與 SVG 連線同層天生同步。
- 縮放夾 `0.5–1.5`；滾輪：⌘/Ctrl（或觸控板捏合）＝以游標為圓心縮放
  （`exp(-deltaY * 0.002)`），單純滾＝平移；`passive:false` 才擋得住頁面捲動。
  **先判斷再擋**：事件從 `.pick-pop`（＋的選單）發出就整個放行——否則選單捲不動、
  整張圖跟著滑走（踩過）。
- 拖曳平移只認**空白處起手**（`.node`／`.plus-anchor`／`.mini` 各有自己的點擊）。
- `fitView`（⌘9）／`resetView`（⌘0）／⌘±；快捷鍵刻意蓋掉瀏覽器縮放（桌面 app 畫布
  語意），打字中不接手。
- `centerOn(gx, gy, ratioY)` 是**唯一**的跳轉原語——小地圖點擊、搜尋、⚠ 導航三個
  消費者共用。

### 8c. 小地圖（2-4）

- 預設顯示、工具列鈕可關（門檻式自動顯隱被推翻——兩頭都會猜錯而使用者沒辦法）。
- 範圍＝**圖 ∪ 目前視窗**（react-flow MiniMap 同款），再撐成小地圖外框
  （`MINI_BOX = 132×92`）的長寬比＋等比留邊——修掉「視窗框被 viewBox 裁掉」與
  「內容吊在正中、縮放倍率跳動」兩個症狀。
- 視窗看得完整張圖時不畫框（框＝全部＝零資訊）；框外遮罩用單一 path＋
  `fill-rule="evenodd"` 挖洞。
- 點擊反推座標**用 `miniBox` 不用 `layout`**（viewBox 原點換了，不跟著換點下去會偏）。

## 9. 大圖可讀性（Phase 3）

- **3-1 畫布搜尋**：`query` 比對 label 與 ref、find-as-you-type 跳第一個命中、
  Enter/Shift+Enter 輪替；**搜完整的圖**（含收合成員，跳過去自動展開）。
  `NAV_MIN_NODES = 5` 以下不畫搜尋框（現在只剩它用這個門檻）。**跟 ⌘K 是兩件事**：
  ⌘K＝「有什麼可以放」，搜尋＝「我畫的那顆在哪」。
- **3-2 ⚠ 導航**：點清單一列到現場；**邊的問題順手把那條邊的＋打開**（那顆＋就是
  修法——插一顆轉接頭），終點是動作不是第四份解釋。原 Phase 4（「邊要能選中」）被
  吸收後整階刪除。落點偏上（`NAV_Y = 0.35`，選單往下開，正中央會被裁）。高亮用
  class＋setTimeout（1400ms）**不用 transition/animation**——HMR 與 Browser pane
  隱藏都會讓動畫停格（repo 已知陷阱）。
- **3-3 群組收合**（`collapse.ts`）：一段＝一條直鏈（目前這顆唯一出邊、下一顆唯一
  入邊才吞；一條規則同時擋分岔與匯流）。**純顯示**：狀態在元件（`collapsedHeads`
  陣列）、不進 controller、不進存檔、載入清空——收合前後存出的 recipe 逐條相同。
  三張映射表：`members`（收合卡→成員鏈序）、`repOf`（真節點→畫面上的代表）、
  `realOf`（畫出來的邊→真邊）。**幾何用代表、互動用真的**——邊上＋照畫出來的端點
  呼叫 `insertOnEdge` 會插到不存在的邊上（靜默 no-op）。收合卡：kind 寫頭的、
  摘要行寫「＋n 顆 › 尾名」、成員 ⚠ 冠名浮上來、後＋看**尾**、下＋不畫（從哪顆分
  出去含糊）、✕ 不給（要刪先展開）。巢狀由層序決勝（外圈贏）；失效自己好（鏈改斷
  ／頭被刪 ⇒ walk 走不出兩顆 ⇒ 自動不成立）。

## 10. 節點設定（右欄檢視器）

- Phase 1-B 把設定從卡上抽屜搬進右欄 `.ed-side`（**搬家不是重寫**，markup 逐字保留）。
  買到卡片高度恆定（座標化前置）；**每一顆都選得中**（模態／出口也是，右欄誠實說
  「沒東西可調」——只有部分卡點得動是猜謎）。點卡片＝選中（`selected`，以 id 查圖
  不存物件——swapModel 換物件後右欄才跟得上）。
- 右欄可拖寬（預設 240、下限 220、上限＝容器寬 − `CANVAS_MIN_W` 320；**夾在 derived
  不寫回**——窄視窗暫時讓步、拉回來偏好自己回來）；雙擊握把還原。
- 判準（`node-settings.ts`）**問埠型別與能力宣告，不問 method id**：
  - `isChainNode`＝吃字幕吐字幕 ⇒ 修正鏈下拉（`chain_pipeline`，選項＝
    `lab.savedPipelines`；沒選存得下去但房間跑不動）；
  - `isCollector`＝吃字幕吐一批 ⇒ 兩格窗口（`window_lines`／`window_ms`，UI 以秒
    輸入、存毫秒；placeholder＝`DEFAULT_WINDOW_LINES 12`／`DEFAULT_WINDOW_MS 60s`
    ——**只是 placeholder，真正決定的是後端** `rooms::window`）；
  - `usesDictionary`＝`m.dict_vocab`（引擎申報；同埠的方法分不出來）⇒ 字典勾選清單
    （一列一本；順序＝勾選序＝載入器疊加序）；
  - `usesKnobs`＝`m.params.length > 0` ⇒ `MethodKnobs`（只畫宣告的那幾顆；預設值
    來自 `lab.knobDefaults`＝`GET /api/dev/lab/presets`，**不是** `lab.topK` 那組
    實驗室即時滑桿——那會漂）＋「回到預設」＝三 key 一起清；
  - `m.group` ⇒ 模型大小單選（一列一尺寸；換尺寸＝`swapModel` 改 ref、會存進圖）。
- 卡面唯讀摘要 `summaryOf`（B-4）：模型尺寸／「已接鏈／未接鏈」（不放鏈名——會吃掉
  整行）／窗口／「字典 n 本｜用房間的字典」（預設是看不出來的選擇，值得說）／旋鈕
  **設過才出聲**（預設就是 manifest 那組，逐顆掛「（預設）」只是吃掉卡片）。
  ⚠ 摘要行必須定高（nowrap＋ellipsis）——`params` 不在重排觸發裡，行高一變線就歪。
- `setParam` 清成空值＝整個 key 拿掉（回到「沒設過」）；**不取整**（threshold 0.9
  取整就變 0）；`toggleDictionary` 取消到全空同樣拿掉 key。

## 11. 與模式一互通（lower / lift）

- **`lowerGraph`（圖 → lane 清單）＝匯出**。lossy：模態→lane `input`、method→chain
  step、出口→lane `sink`（兩端蒸發成欄位）。切分規則：**X 有 sink 子節點、或 ≥2 個
  method 子節點 ⇒ 在 X 之後下刀**；sink 永不開新 lane。`lane_id` 從頭一顆 method 的
  畫布 id 導出（`lane-n1`）——邊是真圖邊、`lane:` 參照存檔那一刻才生成，改圖永不
  cascade。blockers 只擋「結構上不可能」（入度>1、來源直連出口、沒接出口…）與
  「這版後端明說不支援」（即時臂多步鏈、即時 lane 多出口）；**畫布 ⚠ 不在裡面**
  （呼叫端 `recipeBlockers = issueBlocker + lowered.blockers` 一起擋匯出鈕）。
- **`liftGraph`（lane 清單 → 圖）＝匯入**。五類**整份拒載**（半張圖存回去＝沒載進來
  的 lane 悄悄消失）：`chain.kind:'pipeline'`、`extra_inputs`（fan-in）、明示
  `trigger`、`enabled:false`、未知來源 port。同 port 的模態共用一顆；出口一條 lane
  一顆自己的（共用會入度 2 ⇒ 撞 lower 的 blocker）。丟掉的只有欄內順序（座標從沒
  被存過）。
- 匯入的 workflow **不算開著的文件**（`openDoc=null`，存下去必然是新的一張圖）。
- 存成圖那條路（原生、同構）**沒有** lower 的結構 blockers，也不再被 ⚠ 擋
  （`canSave = !saving` 而已）；匯出那條保留自己的閘（`canSaveRecipe`）。

## 12. 周邊表面

- **`/composer` 頁**：subset board，四塊 id（`composer.control`／`canvas`／`sandbox`
  ／`run`），預設顯示三塊、元件選盤增減（palette toggle 後照 Option B 規則：fit 模式
  下 `packHeight > BOARD_ROWS` 自動切 scroll）。只起 lab controller；草稿在
  controller 記憶體中，**本頁與 /develop 是兩份獨立草稿**（已保存的圖共用）。
  Header 的 dev reload 縫（`registerDevReload(lab.load)`）＝全部重抓。
- **`ComposerSavedPanel`**：一顆 `.seg` 切兩張表（預設停在**模式二**——圖是在長的
  那半）。模式二一行一條**邊**（邊清單是 wire 事實，不用推導不說謊）＋「未接線」
  節點另列；卡帶讀取時重驗的 `valid`/`reason`/G6 `warnings`。載入畫布靠**呼叫端注入
  `onLoad`**（清單不寫畫布——畫布狀態是沙盒私有）；GraphEditor 把它 `bare` 掛在
  開啟▾ 浮層裡（工具列 A-3 之前掛在畫布下方）。改名帶 kind（兩張表端點不同）；
  刪除走 `ConfirmModal`，「被主控台選用」由後端 409 擋、訊息原樣浮上來。
- **`MethodShelfOverlay`（⌘K）**：browse-only 方法目錄；分區＝埠簽章
  （`method-shelf.ts`，從 `ComposerMethodLibrary` **抄一份**——原檔在退場名單上、
  零 diff 鐵律，且這份**刻意不判 verdict**：浮層沒有插入點，「接不上」會是謊）。
  與開啟▾ 共用中段浮層版位、互斥（面板 body 有 overflow，自由 popover 會被裁半截）。
- **lab controller**：`graphs` 清單唯一擁有者（沙盒載入下拉與已保存面板讀同一份陣列，
  一邊刪另一邊看得見）；`refreshGraphs` 失敗保留舊清單不清空面板。

## 13. 執行面消費者（圖畫完之後誰在讀）

- **`rooms/graph.rs`（模式二 runtime）**：`room_graph_plan(graph)` 先重跑
  `validate_graph`（存的時候驗過，但可能是別版後端寫的行）再過房間能力表
  `plan_room_graph`——**圖的法則與 runner 能力分工**（同一張圖在批次 runner 跑不跑
  得動是另一個問題）。一個判官三個呼叫點：capture 開場、`/console` 選單顯示、選取
  PATCH。節點參數消費：`chain_pipeline` 解析成鏈、`window_lines/_ms` 進收集器、
  `dictionary_ids()` 開錄時從物件儲存載（載不到不擋錄音）、`opts_over` 疊旋鈕
  （批次鏈側收成 `harness/pipeline.rs::StepOverride{vocab, opts}`，
  `run_pipeline_with_overrides` 逐步套用）。**圖有問題絕不擋錄音**——每種失敗
  log 一句中文回 `None`，退回模式一。
- **`/console` 選單**（`ConsoleWorkflowPanel` ＋ `console/pipeline.rs`）：一顆
  `<select>` 兩個 optgroup＝兩種執行模式；值編碼 `g:<id>`／`r:<id>`／`none`。
  「模式」沒有自己的後端欄位——**`GRAPH_SLOT` marker 有值＝模式二**，同一張
  `console_settings` 表多一個 record key。slot builder 對每張圖跑一次
  `room_graph_plan`（「列得出來 ⇒ 選得下去」，跑不動的 disabled ＋原因寫進 option
  文字）；`select_graph` PATCH 前再驗一次。接線描述（`graph_wiring_summary`）由
  後端整句算好。
- 這條消費鏈也是 DELETE 409 守衛（§7c）存在的理由。

## 14. 刻意不做／已知留白（別當成 bug 修）

| 項目 | 狀態 |
|---|---|
| **fan-in（匯流）** | 畫布長不出入度 >1；選單上架成停用列 `尚未支援匯流`；lift/lower 兩邊都擋。後端格式裝得下（edges 序＝第 k 輸入），純前端成本，**⏸️ 拍板延後** |
| **availAt 收緊成嚴格 wire** | picker 寬鬆／驗證嚴格的落差刻意保留——收緊會改變「什麼接得上」，**⏸️ 先不做**（⚠ 比默默消失更好懂） |
| **節點座標入 wire** | 沒有手動拖曳所以無損失；真要手排再長 `layout` 欄位（零 bump） |
| **`kind:"pipeline"` method 節點** | 指向已存 Tier-2 pipeline 的節點是未來臂；`validate_graph` 現在當未知 method 拒絕（不默默存） |
| **收合存成可重用的東西** | 會動 wire，違反後端零 diff；收合永遠純顯示 |
| **關分頁掉最後一秒** | debounce 必然代價，記過帳 |
| **減少交叉的排版** | 大圖仍會交錯；`layout.ts` 介面已為換演算法留乾淨（可引演算法、不引框架——本案拍板「不引進圖框架」） |
| **compact（<860px）** | 誠實放棄：不畫編輯器，一句話＋已保存清單 |
| **顯示名不存** | 特性不是缺口（方法改名圖自動跟上） |
| **批次 graph runner** | 圖存得下批次源／批次出口（G6 只警告），但**沒有任何東西會離線走一遍圖**——現行唯一執行者是房間（即時）。三階落地計畫在 `05_PLAN_offline_graph_runner.md`（2026-08-06 排進佇列 04、同日推後成 05；歷任檔名 `DISCUSSION_review_compare_issues.md` → `04_DISCUSSION_batch_workspace.md`） |

## 15. 階段沿革速查（詳情在計畫檔）

| 階段 | 產出 |
|---|---|
| 0-1 ~ 0-4 | 圖模型／型別守衛／圖驗證／存載 整套從元件搬進 `graph-controller.svelte.ts` |
| 1-A | 四個邊的編輯器表面；開著的文件（openDoc/baseline/dirty）；開啟▾／⌘K 浮層；改名 `GraphEditor.svelte` |
| 1-B | 設定搬右欄檢視器（B-2 內容、B-3 拆抽屜換 `selected`、B-4 卡面摘要）；旋鈕定調 per-node、工具列假旋鈕整組刪除 |
| 2 | 2-1 座標化（`layout.ts`，拆量測整套）；2-2 pan/zoom；2-3 fit/100%/鍵盤；2-4 小地圖 |
| 3 | 3-1 畫布搜尋；3-2 ⚠ 導航（吸收並刪除原 Phase 4）；3-3 群組收合 |
| （另案） | 自動存檔（integrity/runnability 兩層閘拆分）；節點旋鈕；節點字典；`/composer` 版面重構 |
