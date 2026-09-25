# PLAN — `/composer` 圖編輯器主計畫

## 這份文檔是什麼（2026-08-07）

**主計畫**：把 `REFERENCE_composer_graph_editor.md`（別的專案 pvdt 的 as-built 文檔）
的構想，落到 media-app 的實際條件上。定範圍、拍板決策、切階段、列出刻意不做的事。

這是 PLAN 不是 REFERENCE——裡面有待辦、有未定案、有「做到一半可能推翻」的判斷。
實作完成後應該回頭改寫成 as-built。

**配套文檔（建議照這個順序 review）：**

| # | 文檔 | 為什麼要先看 |
|---|---|---|
| 1 | `REFERENCE_composer_method_manifest.md` | **最重要**。14 顆 step 的埠宣告草稿。它決定圖畫出來長什麼樣，也揭露了一個壞消息（見 §3） |
| 2 | 本檔 | 範圍、決策、階段 |
| 3 | `PLAN_composer_phase0_backend.md` | Phase 0 後端實作細節 |
| 4 | `PLAN_composer_phase1_frontend.md` | Phase 1 前端實作細節 |

---

## 1. 一句話定位

**`/composer` 是一個新分頁，用 DAG 畫布把 media-core 的 `PipelineStep` 組成一張可存、
可驗、可執行的處理圖。** 節點＝來源／方法／出口，邊＝「資料槽依賴」，執行＝拓樸排序
後逐顆跑在共享的 `MediaContext` 黑板上。

它**不取代** `/processing`、`/analysis`、`/camera` 那幾個既有分頁——那些是「單一操作
的專用面」，`/composer` 是「把多個操作接起來的組裝面」。既有分頁**零 diff**。

---

## 2. 為什麼是現在、為什麼合適

media-core 已經有 `pipeline/` 模組（327 行）：`PipelineStep` trait、`Pipeline<T>` 執行器、
`MediaContext` 黑板。目前有 **14 顆 step 實作**，全部靠 `commands/pipeline.rs`（839 行）
用寫死的方式一支 command 組一條線。

關鍵觀察：**`MediaContext` 的欄位就是現成的埠型別。**

```rust
pub struct MediaContext {
    pub source: MediaSource,                        // File / Stream / Camera
    pub metadata: Option<MediaMetadata>,
    pub analysis: Option<AnalysisReport>,
    pub extracted_frames: Vec<FrameData>,
    pub hls_result: Option<HLSResult>,
    pub annotation_result: Option<AnnotationResult>,
    pub video_process_result: Option<VideoProcessResult>,
    pub process_result: Option<ProcessFilesResult>,
    ...
}
```

每顆 step 其實已經在宣告自己的埠了，只是宣告在**程式碼裡**而不是**資料裡**。把它搬進
資料，picker 過濾、⚠ 驗證、埠型別檢查就全部有了判準——而且**判準來自宣告不來自
step id**（新 step 零改動）。

這就是本案的地基。**Phase 0 做的就是這件事，不是畫面。**

---

## 3. ⚠️ manifest 草稿揭露的問題（review 時請重點看這節）

把 14 顆 step 的埠宣告寫出來之後，發現一件事：

> **現況下 10 顆 MediaContext step 裡，有 8 顆是「讀 `source` → 寫自己的槽」。
> 沒有任何一顆 step 讀另一顆 step 寫的槽。**

也就是說，**照現況直接畫圖，畫出來會是一朵扇形**：

```
        ┌─→ ExtractMetadata
        ├─→ DetectMotion
File ───┼─→ ExtractFrames
        ├─→ ConvertToHLS
        └─→ ExtractFramesToDisk        ← 全部平行，沒有任何串接
```

一朵扇形不需要圖編輯器，一個 checkbox 清單就夠了。**畫布會是過度設計。**

### 解法：讓「手填路徑」可以來自上游

真正的機會在於——好幾顆 step 的建構參數是**手填的路徑**，而那些路徑完全可以是上游
的產出：

| 上游 | 產出 | 下游 | 現況 | 改成 |
|---|---|---|---|---|
| `ExtractFramesToDisk` | `video_process_result.output_dir` | `GroupSimilarImages` | `input_dir` 手填 | 留空＝吃上游 |
| `ExtractFramesToDisk` | 同上 | `AnnotateVideo` | `frames_dir` 手填 | 留空＝吃上游 |
| ~~`ExtractFramesToDisk`~~ | ~~同上~~ | ~~`ProcessFiles`~~ | ❌ **做不到**，見下 | — |

> ⚠️ **2026-08-07 實作時推翻**：原本以為 `ProcessFiles` 也能吃上游，**是錯的**。它的建構式
> 只有 `new(output_dir)`，輸入固定取自 `context.source.as_path_str()`，struct 上沒有任何
> 輸入欄位可注入 ⇒ registry 無從覆寫。要讓它吃上游得先有 `RebindSource`（換掉黑板的
> source），那是 Phase 2 候選。**路徑繼承鏈只有兩條，不是三條。**

改完之後這兩條鏈是真的：

```
File → ExtractFramesToDisk(out=A) → GroupSimilarImages(in=A, out=B)     影片抽幀 → 相似分群
File → ExtractFramesToDisk(out=A) → AnnotateVideo(frames=A, out=v.mp4)  影片抽幀 → 標註重組
```

加上 `save_report`（Phase 0 已實作）讀 `metadata` ＋ `analysis`，圖的實際深度是 **3 層**：

```
File ─┬→ ExtractMetadata ──────────────────────┐
      ├→ DetectMotion ────────────────────────┤→ SaveReport
      └→ ExtractFramesToDisk → GroupSimilarImages ┘
```

**這帶來一個必須修正的設計判斷。** 原本的想法是「圖 → 拓樸排序 → 塞進現有的
`Pipeline::execute`，executor 完全不用改」。但只要下游 step 的**建構參數**依賴上游的
**執行結果**，就不能先把所有 step 建好再跑——`Pipeline` 是先 `add_node` 收集完
`Vec<Box<dyn Step>>` 才 `execute`。

⇒ **Phase 0 需要在 app 側自己寫一個小 graph runner（約 80 行）**：逐顆按拓樸序
「先看 context 解析參數 → 建 step → execute」。`media-core` 依然**零 diff**，
`PipelineStep` / `MediaContext` 原樣使用。細節見 `PLAN_composer_phase0_backend.md` §5。

### 還可以再解鎖（列入 Phase 2 候選，不進 v1）

在 **app 側自己寫 `PipelineStep` 實作**（trait 是 pub 的，不用改 media-core）補幾顆
「銜接型」節點，圖的深度會再上一層：

- **`SaveReport`**（讀 `metadata` ＋ `analysis` → 寫 JSON 檔）＝真正的**出口節點**。
  現在 `metadata` / `analysis` 兩個槽**沒有任何消費者**，補了這顆它們才有下游。
- **`RebindSource`**（讀上游的 `output_path` → 換掉 `context.source`）＝把「A 的產出
  當 B 的輸入來源」一般化。它會讓現在所有死路（HLS 產物、標註產物）全部接得下去。
  威力很大但語意有點 hacky（黑板中途換 source），**先記帳不做**。

---

## 4. 五個拍板決策

| # | 決策 | 拍板 | 理由 |
|---|---|---|---|
| D1 | manifest / registry 放哪 | **app 側 `src-tauri/src/graph/`** | `media-core` 是 git submodule（v0.4.0，另一個 repo），目前工作流一直排除它。放 app 側＝media-core 全程零 diff、不用跨 repo 升版 |
| D2 | recipe 存哪 | **`{app_data_dir}/composer/graphs/*.json`** | SurrealDB 有（dedup 在用）但 schema 是 dedup 專用。JSON 檔最簡單、可手改、可版控、debug 時 `cat` 得出來。Phase 2 真的需要查詢再搬 DB |
| D3 | 分頁路徑 | **`/composer`** | 沿用參考文檔的詞彙，溝通成本最低。`/pipeline` 會跟既有 `commands/pipeline.rs` 的「單一 pipeline」語意打架 |
| D4 | 圖框架 | **不引進，手刻** | `package.json` 目前沒有 svelte-flow / xyflow / dagre / elk。svelte-flow 自帶 state model 會跟 Svelte 5 runes 打架。參考文檔的 `layout.ts` 是 235 行純函式，抄得動 |
| D5 | 執行模型 | **拓樸序 + 逐顆建構的 app 側 runner** | 見 §3。不是真並行——分岔在畫面上是分岔，執行是拓樸線性化。**這是刻意的**，因為資料本來就在黑板上不在線上 |

---

## 5. 階段路線

每一階段自己能跑，不是全做完才有價值。

### Phase 0 — 後端骨架（無畫面）

`src-tauri/src/graph/`：`manifest.rs`（14 顆 step 宣告）＋ `registry.rs`（字串→step 工廠）
＋ `validate.rs`（G1–G5 ＋ topo）＋ `runner.rs`（逐顆建構的執行器）＋
`commands/graph.rs`（3 支 command）。

- **規模**：約 700–1000 行純 Rust
- **驗收**：手寫一份 JSON 圖（`File → ExtractFramesToDisk → GroupSimilarImages`）
  丟給 `execute_graph`，跑得出結果。**此時前端一行都沒改。**
- **這階段的真正價值**：如果 manifest 寫下去發現埠型別比預期更貧乏、或 runner 的
  參數解析比預期難，我們會在花 1500 行畫布**之前**知道。
- 細節：`PLAN_composer_phase0_backend.md`

### Phase 1 — 畫布最小可用

新分頁 `/composer`：四個邊（工具列／畫布／右欄檢視器／狀態列）、四種生長
＋邊上插入、`layout.ts` 座標引擎、picker 依 manifest 過濾、右欄調參數、一顆「執行」。

- **規模**：約 1200–1500 行（本案主體）
- **驗收**：畫一張圖 → 按執行 → 看到結果
- **刻意不做**：小地圖、搜尋、群組收合、自動存檔
- 細節：`PLAN_composer_phase1_frontend.md`

### Phase 2 — 存載 ＋ 驗證顯示

recipe 持久化（D2 的 JSON 檔）、已保存清單面板、常駐 ⚠（`issues` 三張表）、狀態列提示。
候選：§3 的 `SaveReport` 出口節點。

- **規模**：約 500 行

### Phase 3 —（選配）大圖可讀性

小地圖、畫布搜尋、⚠ 導航、群組收合。參考文檔 §9 整段。

**先不排期。** 圖只有 5–8 顆節點時這些是純負擔；等真的畫出 20 顆的圖再說。

### Phase 4 — gallery catalog

依 repo 慣例（`gallery/pages/composer/+page.svelte` ＋ `sections.ts` 條目 ＋
`CompositionTree`），與 `/audio` 同一套做法。

---

## 6. 刻意不抄（從參考文檔剔除的東西）

| 參考文檔的東西 | 我們 | 理由 |
|---|---|---|
| `lower-graph.ts` / `lift-graph.ts`（模式一互通） | **整組不要** | 那是為了與他們既有的 lane 世界共存。我們沒有模式一，`/processing` 保持原樣不碰 |
| `chain_pipeline` / `dictionaries` / `top_k` / `window_lines` / `window_ms` | **不要** | 純 ASR 領域參數。我們的參數是 `threshold` / `algorithm` / `interval` / `strategy` / `segmentDuration` / `profile` / `saveMode` |
| HTTP CRUD（`/api/dev/lab/graph-recipes`）、tier 權限 | **不要** | 我們是 Tauri `invoke()`，沒有 server。CRUD 對應成 3–5 支 command |
| `/console` 執行模式選單、`rooms/graph.rs`、`GRAPH_SLOT` marker | **不要** | 沒有「房間」概念，也沒有即時 runtime 消費者 |
| DELETE 409 守衛 | **不要** | 那是為了保護 `/console` 的選取。我們沒有選取狀態 |
| 自動存檔（1 秒 debounce ＋ 三條守衛 ＋ `autosaveStop`） | **Phase 2 之後再說** | 他們是因為砍掉儲存鈕才**必須**做。我們留一顆儲存鈕就好，省掉一整段複雜度 |
| 「不引進圖框架」的判斷 | ✅ **這條要抄** | 見 D4 |
| fan-in 上架成停用列（`尚未支援匯流`） | ❌ **反過來，我們支援** | 他們的模型是逐邊傳值所以匯流難。我們是黑板 + 槽依賴，一顆 step 讀兩個槽＝兩條入邊，天生就通 |

---

## 7. 已知風險與記帳

| 風險 | 影響 | 對策 |
|---|---|---|
| **同一顆 method 在一張圖裡放兩次會互蓋** | 黑板一個槽一份，第二顆的寫入蓋掉第一顆 | v1 **直接擋**：同一 `method_ref` 一張圖最多一顆（validate 的 G7）。「真正逐邊傳值」列入刻意不做 |
| **`DetectMotion` 是覆寫不是合併** | 它 `context.analysis = Some(report)`，會清掉 `GroupSimilarImages` 寫的 `similarity_groups` | manifest 標 `writes_mode: replace`；validate 警告「同槽多寫入者且其中一顆是 replace」 |
| **圖的深度只有 2 層** | 見 §3。畫布的價值打折 | 誠實記帳。Phase 0 做完就看得出來值不值得往 Phase 1 走——**這是 Phase 0 先做的主因** |
| **`streaming::ExtractionMode` 與 `video_process::ExtractionMode` 同名不同物** | registry 的參數映射會寫錯 | manifest 用不同的 param key（`frameMode` vs `extractionMode`），文檔標紅 |
| **兩顆 benchmark step 吃 `BenchmarkContext` 不是 `MediaContext`** | 泛型不同，進不了同一張圖 | v1 排除。要納入得先統一 context，屬 media-core 的工作 |
| 規模 | 參考文檔背後是 4000+ 行前端 ＋ 一整套後端 | 我們做到 Phase 2 約 **2500–3000 行**，realistically 好幾個 session。不是一個下午的分頁 |

---

## 8. 檔案地圖（規劃中，尚未存在）

### 後端（Phase 0）

| 檔案 | 角色 |
|---|---|
| `src-tauri/src/graph/mod.rs` | 模組出口 |
| `src-tauri/src/graph/types.rs` | wire 格式：`GraphRecipe` / `GraphNode` / `GraphEdge` / `NodeParams` |
| `src-tauri/src/graph/manifest.rs` | 方法宣告表（14 顆 step 的埠 ＋ 參數 ＋ 家族） |
| `src-tauri/src/graph/registry.rs` | `node_ref` ＋ params ＋ context → `Box<dyn PipelineStep<MediaContext>>` |
| `src-tauri/src/graph/validate.rs` | `topo_order` ＋ G1–G7 |
| `src-tauri/src/graph/runner.rs` | 逐顆建構 ＋ 執行 ＋ 進度事件 |
| `src-tauri/src/graph/store.rs` | JSON 檔 CRUD（Phase 2） |
| `src-tauri/src/commands/graph.rs` | Tauri command 層 |

### 前端（Phase 1）

| 檔案 | 角色 |
|---|---|
| `src/routes/composer/+page.svelte` | 分頁殼 |
| `src/lib/components/features/composer/GraphEditor.svelte` | 編輯器本體（畫面狀態） |
| `src/lib/components/features/composer/graph-controller.svelte.ts` | 模型層（nodes/edges/params/生長/驗證） |
| `src/lib/components/features/composer/graph/layout.ts` | 座標引擎（純函式） |
| `src/lib/components/features/composer/graph/layers.ts` | 拓樸分層 |
| `src/lib/components/features/composer/graph/NodeInspector.svelte` | 右欄檢視器 |
| `src/lib/components/features/composer/graph/node-options.ts` | picker 過濾純函式 |
| `src/lib/components/features/composer/index.ts` | barrel |

### 需要**修改**的既有檔案（全部是加法）

`src/lib/nav.ts`（加一列）、`src/lib/types.ts`（graph 型別）、
`src/lib/components/ui/icon-registry.ts`（幾顆圖示）、`src-tauri/src/main.rs`（註冊 command）、
`src-tauri/src/commands/mod.rs`（`pub mod graph;`）、
`src/lib/components/features/gallery/sections.ts`（Phase 4）。

**`src-tauri/crates/media-core/` 全程零 diff。**

---

## 9. 待確認事項（review 時請回答）

1. **§3 的壞消息接受嗎？** 圖的深度只有 2 層、要靠「留空＝吃上游」才串得起來。
   如果覺得這樣的圖價值不夠，Phase 0 做完可以喊停，成本只有 ~900 行後端（而且那份
   manifest ＋ registry 本身對 `commands/pipeline.rs` 的重構也有價值）。
2. **五個決策 D1–D5 有沒有要改的？**
3. **Phase 0 做完是否要先停下來看結果**，再決定走不走 Phase 1？（建議：要）
4. **`SaveReport` 出口節點要不要提前到 Phase 0？** 它會讓 `metadata` / `analysis`
   兩個槽有消費者，圖立刻多一層深度。成本約 80 行。
