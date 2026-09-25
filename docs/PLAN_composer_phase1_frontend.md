# PLAN — `/composer` Phase 1：畫布最小可用

## 這份文檔是什麼（2026-08-07）

**Phase 1 的實作規格**：`/composer` 分頁、圖編輯器四個邊、生長模型、picker 過濾、
座標引擎、右欄檢視器、執行與結果。

前置：`PLAN_composer_phase0_backend.md` 必須先完成並驗收——本階段的 picker 與右欄
**完全由 `list_graph_methods` 驅動**，前端不硬編任何 step 名字。

**進入條件**：Phase 0 驗收 §8 第 4 條（串接圖自動吃上游）通過，且 review 確認
「圖的深度只有 2 層」可以接受（主計畫 §3）。

---

## 1. 檔案地圖

```
src/routes/composer/+page.svelte                         分頁殼（PageContent ＋ GraphEditor）

src/lib/components/features/composer/
├── index.ts                          barrel
├── GraphEditor.svelte                編輯器本體（~700 行）：四個邊 ＋ 所有畫面狀態
├── graph-controller.svelte.ts        模型層（~450 行）：nodes/edges/params/生長/驗證/執行
└── graph/
    ├── layout.ts                     座標引擎（純函式，~200 行）
    ├── layers.ts                     拓樸分層（~40 行）
    ├── node-options.ts               picker 過濾純函式（~120 行）
    ├── NodeCard.svelte               單顆卡片（~120 行）
    ├── NodeInspector.svelte          右欄檢視器（~250 行）
    └── ParamField.svelte             依 ParamKind 畫一格參數（~100 行）
```

修改的既有檔案（純加法）：
`src/lib/nav.ts`（一列）、`src/lib/types.ts`（graph 型別鏡像）、
`src/lib/components/ui/icon-registry.ts`（`workflow` / `plus` / `play` / `trash` 等）、
`src/lib/events.ts`（`graph:progress` / `graph:complete` / `graph:error` 監聽）。

---

## 2. 分層界線（本案的一條線）

抄參考文檔 §4，那個界線是對的：**規則住 controller／純函式，畫面住元件**。

| 住哪 | 什麼 |
|---|---|
| `GraphEditor.svelte` | `picker`（選單開在哪一格）、`selected`（右欄畫誰）、pan/zoom/fit、浮層開關、右欄寬度、執行中的進度條。**只有畫面狀態** |
| `graph-controller.svelte.ts` | `nodes` / `edges` / `params`、四種生長 ＋ 邊上插入 ＋ 移除縫合、`issues`（`$derived`）、`toWire()` / `fromWire()`、執行（invoke ＋ 事件） |
| `graph/*.ts` 純函式 | 分層（`layers.ts`＝圖的性質）、排版（`layout.ts`＝擺哪裡）、picker 過濾（`node-options.ts`＝什麼接得上）。三支檔三條界線 |

repo 慣例同步遵守：`lib/` 全域零 `$effect`——計時器與生命週期（`$effect` ＋ `onDestroy`）
全在元件。Svelte 5 runes（`$state` / `$derived` / `$props`），不用 legacy store。

---

## 3. 畫布資料模型

```ts
// 畫布側（controller 私有）
interface CanvasNode {
  id: string;          // 'n0', 'n1' … mint 遞增
  kind: 'source' | 'method' | 'sink';
  ref: string;         // 'file' | 'extract_frames_to_disk' | 'save_report'
  label: string;       // 顯示名，從 manifest 反查（不上線，見下）
}
interface CanvasEdge { from: string; to: string }
type ParamsMap = Record<string, Record<string, unknown>>;   // nodeId → { key: value }
```

- **`label` 不上線**：wire 節點沒有顯示名，讀回來從 `ref` 反查 manifest。方法改名後圖
  自動跟上；round-trip 保證的是**節點與邊等價**，不是位元組等價。（抄參考文檔 §3）
- **`params` 另放一張 map**，不塞進 `CanvasNode`——刪節點時順手丟掉該 id 的 params
  （id 會被重用，留著會悄悄套到別顆上）。
- **`nextSeq` 取「最大 n{k} ＋ 1」而非節點數**——刪過節點的圖不連號。
- **座標不進 wire**：存結構不存位置，每次載入重排（決定性）。沒有手動拖曳所以無損失。

---

## 4. 生長模型（沒有手動拉線）

邊不是拖出來的，是**由插入動作生成的**。前四種抄參考文檔 §5：

| 入口 | 語意 | 邊變換 |
|---|---|---|
| `root` | 空畫布放第一顆（一定是 source） | 無邊 |
| `after`（後＋） | 串接插入下游 | X 的每條出邊 X→S 改 N→S，加 X→N |
| `before`（前＋） | 串接插入上游 | X 的每條入邊 P→X 改 P→N，加 N→X |
| `below`（下＋） | 平行分支（吃同一份輸入） | X 的每個上游 P 加 P→N |
| 邊上＋ | 插在兩顆之間 | 移除 from→to，接 from→N→to |

- 前／後＋**只在自由端**出現（有鄰居那側交給邊上＋）
- **source 不長前＋**（來源沒有輸入）；**sink 不長後＋**（資料到此離開圖）
- `removeNode`＝邊上插入的反操作：上游 P × 下游 S 縫成 P→S（去重、不自環），
  順手丟掉該節點的 params

### 第五種：埠上＋（我們獨有，參考文檔沒有）

參考文檔的四種生長都維持入度 ≤ 1 ⇒ 圖是森林，所以他們把 fan-in 上架成停用列。
**我們相反**——`save_report` 同時讀 `metadata` 與 `analysis`，是真的需要兩條入邊。

作法：**不是拖線，是埠上的＋**。

- 節點卡的左緣，依 manifest 的 `reads` 畫出 N 個輸入埠位
- 已被某條入邊佔用的埠位畫實心點；**未滿足的埠位畫一顆小＋**
- 點那顆＋ ⇒ 浮出「圖上哪些既有節點寫過這個槽」的清單 ⇒ 選一個 ⇒ 加一條邊
- 清單空 ⇒ 誠實給理由（「圖上還沒有人產生『分析報告』」），不是靜默

`edges` 是清單不是集合，**順序有意義**：同一顆節點的入邊順序＝它 `reads` 的順序。
`toWire` 因此不排序、不去重。

---

## 5. picker 過濾（`node-options.ts`）

比參考文檔的 `availAt` 簡單很多——我們的供給模型是**槽**不是埠型別鏈：

```ts
// 「跑到這一點為止，哪些槽已經有人寫過」
function suppliedAt(point: InsertPoint, g: GraphModel, meta: MethodMeta[]): Set<Slot>
```

三區規則：

- **source 區**：只在「新節點不會有入邊」的格子上架（空畫布 / `before` 到頭）。
  一張圖只能一顆 source ⇒ 已經有了就整區不上架，理由寫「一張圖只能有一個來源」
- **method 區**：`meta.reads` 的每個槽，要嘛 `suppliedAt` 有、要嘛該 step 對應的
  param 是 `fallback_from` 以外的必填路徑（使用者自己填）。
  ⚠️ **來源變體要對**：source 是 `stream` / `camera` 時，只有 `capture_frame` 的
  `accepts` 收得住（見 manifest §1a）——這一條會讓整個清單只剩一項，UI 必須把理由
  講出來
- **sink 區**：只在「不會有下游」的格子上架

其他規則：

- **G7 前置**：manifest 裡已經在圖上出現過的 `ref` 直接**上架成停用列**
  （`blocked: '一張圖同一個方法只能有一顆'`），不藏起來——藏起來沒人記得放回來
- **排序**：能直接吃到「剛好上一顆寫的槽」的排前面（例如 `extract_frames_to_disk`
  之後，`group_similar_images` / `annotate_video` / `process_files` 要排在最前），
  其餘照 manifest 序。排序不動篩選
- 兩區皆空要給誠實理由（`emptyReason`），不是空白選單

---

## 6. 座標與渲染

### 6a. 佈局引擎 `layout.ts`

**先算座標、照數字擺，不量 DOM**（抄參考文檔 §8a——那個決定省掉一整套
ResizeObserver / measure / graphSig）。前提是**卡片尺寸定死**：

```ts
export const NODE_W     = 220;   // 比參考文檔的 200 寬一點：我們的 label 是中文＋英文 ref
export const NODE_H     = 32;    // 純標頭
export const NODE_H_SUM = 52;    // 帶一行摘要
export const LAYER_GAP  = 48;
export const ROW_GAP    = 28;
export const PAD        = 28;    // 外圈的＋浮在卡片外，貼邊會被裁
```

**這支檔是尺寸唯一真相**，元件以 inline style 寫進卡片，CSS 不准有第二份數字。

指派規則：層 → x（`computeLayers` 拓樸深度）；y 跟著上游走——每顆算 `anchor`
（上游中心的平均，**排序用**）與 `want`（讓自己中心對上 anchor，**擺放用**），
同層照 anchor 由上往下推開避免重疊。

> ⚠️ **`anchor` 與 `want` 必須分開**：want 含自己半高，高矮不同的兄弟會不同分，
> 排序會把分岔上下對調。對**中心**不對上緣。取整避免 .5px 糊字。
> （這是參考文檔記過的真實踩雷，直接沿用結論）

`routeEdges`：出邊錨點沿上游卡**右緣**均分、入邊沿下游卡**左緣**均分，各照對面中心
排序（fan-out / fan-in 不自己交叉）；bezier 控制點水平拉出
（`dx = max(10, |Δx| * 0.4)`）；回傳邊中點 `mx/my` 給邊上＋定位。

介面留乾淨——呼叫端只拿 `{ boxes, edges, width, height }`，換演算法零改動。
**已知不做**：減少交叉。

### 6b. 視角

- 畫布＝`overflow:hidden` 的洞 ＋ 被 transform 的座標平面；卡片與 SVG 連線同層天生同步
- 縮放夾 `0.5–1.5`；滾輪：⌘/Ctrl（或觸控板捏合）＝以游標為圓心縮放，單純滾＝平移；
  `passive:false` 才擋得住頁面捲動
- ⚠️ **先判斷再擋**：事件從 picker 浮層發出就整個放行——否則選單捲不動、整張圖跟著滑走
- 拖曳平移只認**空白處起手**
- `fitView`（⌘9）／`resetView`（⌘0）／⌘±；打字中不接手

**Phase 1 不做**：小地圖、搜尋、⚠ 導航、群組收合（全部 Phase 3）。

### 6c. ⚠️ Tailwind v4 的動態 class 陷阱

節點卡要依 `category` 上色（analysis / extract / convert / annotate / process）。

> **Tailwind v4 掃不到執行期組出來的 class 名。** repo 已經踩過一次：
> `SparklineBar.svelte` 用 `` `${color}/20` `` 組出 `bg-[#137fec]/20`，那條 utility
> 從來沒被產生過，全 app 的暗色歷史長條是**透明的**。

⇒ `category → class` 必須是一張**完整字面值**的對照表：

```ts
const CATEGORY_CLASS = {
  analysis: 'border-violet-500 bg-violet-500/10 text-violet-600 dark:text-violet-400',
  extract:  'border-sky-500 bg-sky-500/10 text-sky-600 dark:text-sky-400',
  // …每一條都是完整、可被靜態掃到的字串
} as const;
```

不准 `` `border-${c}-500` ``。

---

## 7. 右欄檢視器（`NodeInspector.svelte`）

- **每一顆都選得中**（source / method / sink 都是）。沒東西可調就誠實說「這顆沒有
  參數可調」——只有部分卡點得動是猜謎
- 點卡片＝選中（`selected` 存 **id 不存物件**）
- 右欄可拖寬（預設 260、下限 220、上限＝容器寬 − `CANVAS_MIN_W` 340；**夾在 derived
  不寫回**——窄視窗暫時讓步、拉回來偏好自己回來），寬度存 localStorage
- 內容**完全由 manifest 的 `params` 驅動**，`ParamField.svelte` 依 `kind` 分派：

| `ParamKind` | 元件 |
|---|---|
| `bool` | `ToggleSwitch`（既有） |
| `int` / `float` | `FormField` type=number（帶 `min`/`max`） |
| `string` | `FormField` type=text |
| `enum` | `<select>`（選項來自 `options`） |
| `path` / `dir` | `FilePicker` / `DirPicker`（`features/analysis` 已有，抽到 `ui/` 共用） |

- **`fallbackFrom` 有值的參數**：placeholder 寫「留空＝使用上游的輸出目錄」，
  且上游真的接上了就在下面加一行灰字顯示會取到什麼
- 清成空值＝整個 key 拿掉（回到「沒設過」）；**不取整**（threshold 0.9 取整變 0）

### 卡面唯讀摘要

一行、定高（`nowrap` ＋ `ellipsis`）——`params` 不在重排觸發裡，行高一變線就歪。
內容：**設過才出聲**（預設就是 manifest 那組，逐顆掛「（預設）」只是吃掉卡片）。

---

## 8. 執行與結果

- 工具列一顆 `RunButton`（既有元件）。按下去：`validate_graph` → 有 `reason` 就擋並
  顯示 `ErrorAlert`；只有 `warnings` 就照跑但在狀態列列出來
- 執行中：`graph:progress` 事件 → 當前節點卡加高亮 class ＋ 狀態列進度
  ⚠️ 高亮用 **class ＋ setTimeout**，**不用 CSS transition/animation**——HMR 與
  Browser pane 隱藏都會讓動畫停格（repo 已知陷阱）
- 完成：右欄下方或底部面板顯示 `GraphRunResult`（`metadata` / `analysis` /
  各種 `*_result` 的摘要卡）
- 取消：執行中 RunButton 變「取消」，呼叫 `cancel_graph_run`

**Phase 1 的結果呈現先做最簡**：一張 `Card` 列 JSON 摘要即可，不做圖表、不做縮圖預覽。

---

## 9. Phase 1 刻意不做

| 項目 | 去哪 |
|---|---|
| 存載、已保存清單、`openDoc` / `dirty` / 覆蓋確認 | Phase 2 |
| 自動存檔 | 不做（主計畫 §6：我們留儲存鈕，省掉整段複雜度） |
| 小地圖、畫布搜尋、⚠ 導航、群組收合 | Phase 3（且不排期） |
| ⌘K 方法目錄浮層 | Phase 3。picker 已經夠用了 |
| 匯出／匯入（lower / lift） | 不做（沒有模式一） |
| compact（<860px）版面 | 誠實放棄：不畫編輯器，一句話 ＋ 提示改用桌面寬度 |
| i18n | 全 app 尚未 i18n（`svelte-i18n` 在 package.json 但 `src/` 沒用），單頁翻譯會不一致 |
| gallery catalog | Phase 4 |

---

## 9b. 實作時偏離本文檔的地方（2026-08-07，以 code 為準）

| 文檔原本寫 | 實際做法 | 為什麼 |
|---|---|---|
| `issues` 是 controller 的 `$derived`，前端自己算三張表 | **完全不算**，改成 debounce 200ms 呼叫後端 `validate_graph`，結果只放狀態列 | 驗證規則只留一份（Rust），前後端不可能漂。代價是 async ⇒ 由元件的 `$effect` 驅動。per-node ⚠ 本來就排在 Phase 2 |
| 第五種生長「埠上＋」畫在卡片左緣 | 做在**右欄檢視器的 Inputs 區**，未滿足的埠給一個「Connect an input…」下拉 | 卡片左緣塞不下多個埠位（卡片高度定死 34/54px），而且下拉能直接列出「誰產得出這個槽」＋自動排除會成環的候選 |
| `FilePicker` / `DirPicker` 要從 `features/analysis` 抽到 `ui/` | **不用抽**——它們本來就在 `$lib/components/form/`。另外也沒有重用它們，改寫 `ParamField.svelte` | 那兩顆是 `bind:value` 介面，不適合 manifest 驅動的 `onchange(value)` 回呼 |
| 畫布用 `flex-1` 撐滿高度 | `min-h-[60vh]` | `<main>` 與 `PageContent` 之間還有一層**非 flex** 的 `overflow-y-auto` 包裝，`flex-1` 到那裡就斷了、塌成內容高度（實測 551px）。結果面板出現時整頁照常捲動 |
| `NODE_H = 28` / `NODE_H_SUM = 45` / `LAYER_GAP = 40` / `PAD = 26` | `34` / `54` / `56` / `30` | 抄參考專案的數字對我們的中英混排字級偏擠，實際調過 |
| — | `listen()` 包 try/catch | 在 Tauri 殼外它會**同步 throw**，而 `$effect` 裡未捕捉的 throw 會讓整頁空白。丟掉即時進度遠比丟掉整個編輯器好 |
| — | icon-registry 補 `zoom_in` / `zoom_out` | 原本沒有 |

## 10. 驗收 — 2026-08-07 狀態

**已驗證**（`yarn run check` ＋ 瀏覽器預覽，`invoke` 用真實 manifest JSON 打樁）：
`0 ERRORS / 20 WARNINGS`（baseline 未增加）；picker 分區／排序／amber「需要填參數」列；
root ＋ after 生長；邊與邊上＋、下＋；卡片摘要行與兩種卡高；右欄 manifest 驅動、
Inputs 區的 ← 供給者與 Disconnect；「From upstream」placeholder；fit／reset／zoom；
**light ＋ dark 兩份都確認過**。

**仍待 `yarn tauri dev` 實測**（打樁的 `invoke` 蓋不到）：真正按 Run 執行、`graph:progress`
事件的節點高亮與進度、Cancel、後端 `validate_graph` 的真實 reason 顯示在狀態列、
camera 來源只剩 `capture_frame` 的過濾、重複 method 的停用列。

原始驗收清單：

1. `yarn run check` — **0 errors**（baseline 20 warnings，不得增加）
2. 空畫布 → 放 `file` 來源 → 後＋ 放 `extract_frames_to_disk` → 後＋ 放
   `group_similar_images`（`inputDir` 留空）→ 執行 → 拿到分群結果
3. fan-out：來源下＋ 再開一支 `extract_metadata`，兩支都跑到
4. 埠上＋：放 `save_report`，把 `metadata` 與 `analysis` 兩條入邊接上
5. 來源改成 `camera` ⇒ picker 只剩 `capture_frame`，且空清單有理由
6. 同一個 `ref` 第二次出現 ⇒ 停用列 ＋ 理由
7. pan / zoom / fitView / resetView；浮層開著時滾輪不會讓整張圖滑走
8. light ＋ dark 兩份 preview 截圖（含節點分類色——確認 §6c 的字面值表真的生效）

---

## 11. 拍板紀錄

1. ~~埠上＋要不要進 Phase 1？~~ → **進了**，但形式改成右欄的「Connect an input…」下拉（見 §9b）。
2. ~~`FilePicker` / `DirPicker` 抽到 `ui/`~~ → **不需要**，它們本來就在 `$lib/components/form/`。
   本階段對既有檔案**全部是加法**：`nav.ts`（+1 列）、`types.ts`（+graph 型別段）、
   `icon-registry.ts`（+2 顆圖示）。
3. ~~結果呈現位置~~ → **底部獨立 Panel**，如建議。
