# REFERENCE — `/composer` 方法 manifest 草稿

## 這份文檔是什麼（2026-08-07）

**埠宣告盤點**：把 media-core 現有 `PipelineStep` 的「讀什麼槽／寫什麼槽／有哪些參數」
從程式碼裡挖出來，寫成資料。這是 `/composer` 整案的地基——picker 過濾、⚠ 驗證、
埠型別檢查全部靠它。

內容以 **2026-08-07 的 `media-core` v0.4.0（submodule commit `7e25dde`）原始碼**為準，
每一列都是讀過 `execute()` 本體確認的，不是從 `commands/pipeline.rs` 的呼叫端推的。

**這份是 review 的第一優先**——它決定圖畫出來長什麼樣。

---

## 1. 埠型別＝`MediaContext` 的欄位

```rust
// media-core/src/pipeline/context.rs
pub struct MediaContext {
    pub source: MediaSource,                            // File(PathBuf) | Stream(String) | Camera(i32)
    pub metadata: Option<MediaMetadata>,
    pub analysis: Option<AnalysisReport>,               // { motion_events, similarity_groups, image_comparison }
    pub extracted_frames: Vec<FrameData>,               // 記憶體中的 base64 幀
    pub hls_result: Option<HLSResult>,
    pub annotation_result: Option<AnnotationResult>,
    pub video_process_result: Option<VideoProcessResult>,
    pub process_result: Option<ProcessFilesResult>,
    resources: HashMap<String, Box<dyn Any + Send + Sync>>,   // private，lazy VideoCapture 快取
}
```

八個公開欄位＝八種埠。`resources` 是私有的 lazy 資源快取（`get_opencv_capture`），
**不是埠**，不進 manifest。

### 1a. `MediaSource` 的三個變體不等價 ⚠️

```rust
pub fn as_path_str(&self) -> Option<&str> {
    match self { MediaSource::File(p) => p.to_str(), _ => None }   // Stream / Camera 都回 None
}
```

**`as_path_str()` 對 `Stream` 和 `Camera` 都回 `None`。** 所有呼叫它的 step 在非 File
來源下會直接 `ConfigurationError`。實際能力表：

| 來源 | 能接的 step | 為什麼 |
|---|---|---|
| **`File`** | 全部 11 顆 | `as_path_str()` 有值；`get_opencv_capture()` 也走 `from_file` |
| **`Stream`** | **只有 `CaptureFrame`** | 只有它走 `get_opencv_capture()`（內部 `VideoCapture::from_file(url, CAP_ANY)`），其餘全部卡在 `as_path_str()` |
| **`Camera`** | **只有 `CaptureFrame`** | 同上（`VideoCapture::new(id, CAP_ANY)`） |

⇒ **picker 規則**：來源節點選了 Stream / Camera，下游只上架 `CaptureFrame` 一項。
這不是缺陷是事實，但 UI 要誠實講出來（空清單要給理由，不是靜默）。

---

## 2. manifest 草稿——11 顆 `MediaContext` step

欄位語意：

- `reads` — 執行前必須存在的槽。`source` 後面括號是**接受的來源變體**
- `writes` — 執行後寫進哪個槽
- `write_mode` — `replace`（整份覆寫）／`merge`（併進既有）／`append`（推進陣列）
- `params` — 可調參數（key 是 wire 上的名字，camelCase）
- `ctor` — 建構式簽章（registry 要照這個組）

---

### 2.1 `extract_metadata` — 擷取中繼資料

| | |
|---|---|
| **struct** | `metadata::pipeline::ExtractMetadata` |
| **ctor** | `ExtractMetadata::new(include_thumbnail: bool)` |
| **reads** | `source` (File) |
| **writes** | `metadata` — `replace` |
| **params** | `includeThumbnail: bool = false` |

---

### 2.2 `detect_motion` — 移動偵測

| | |
|---|---|
| **struct** | `analysis::pipeline::DetectMotion` |
| **ctor** | `DetectMotion::new().algorithm(_).threshold(_).min_area(_).output_dir(_)` |
| **reads** | `source` (File) |
| **writes** | `analysis` — ⚠️ **`replace`** |
| **params** | `algorithm: enum = frame_diff` \| `mog2` \| `knn` \| `optical_flow`<br>`threshold: f64 = 25.0`<br>`minArea: i32 = 500`<br>`outputDir: path = "output/motion_debug"` |

> ⚠️ **它是覆寫**：`context.analysis = Some(report)`（`analysis/pipeline.rs:96`），
> 會清掉 `group_similar_images` / `compare_images` 已經寫進去的 `similarity_groups`
> 與 `image_comparison`。validate 的 G6 要對「同槽多寫入者且其中有 replace」出警告。

---

### 2.3 `group_similar_images` — 相似影像分群

| | |
|---|---|
| **struct** | `analysis::pipeline::GroupSimilarImages` |
| **ctor** | `GroupSimilarImages::new(input_dir, output_dir).method(_).process_mode(_).min_category_size(_).group_similar(_).{histogram,phash,feature}_threshold(_)` |
| **reads** | ⚠️ **不讀任何槽**（自帶 `input_dir`） → 見 §4「可吃上游」 |
| **writes** | `analysis.similarity_groups` — `merge` |
| **params** | `inputDir: path`（**留空＝吃上游 `video_process_result.output_dir`**）<br>`outputDir: path`（必填）<br>`method: enum = histogram` \| `feature_matching` \| `perceptual_hash`<br>`processMode: enum = single` \| `parallel`<br>`minCategorySize: i32`<br>`groupSimilar: bool`<br>`threshold: f64`（依 `method` 分派到三個 setter 之一） |

> **`threshold` 的分派**：`method=histogram` → `histogram_threshold()`；
> `method=feature_matching` → `feature_threshold()`；否則 → `phash_threshold()`。
> 這個分派邏輯 `commands/pipeline.rs:194-199` 已經有一份，registry 照抄。

---

### 2.4 `compare_images` — 兩張圖比對

| | |
|---|---|
| **struct** | `analysis::pipeline::CompareImages` |
| **ctor** | `CompareImages::new(image1, image2).method(_).threshold(_)` |
| **reads** | ⚠️ **不讀任何槽**（自帶兩個路徑） |
| **writes** | `analysis.image_comparison` — `merge` |
| **params** | `image1: path`（必填）<br>`image2: path`（必填）<br>`method: enum = perceptual_hash`（**注意：這顆的預設與 2.3 不同**）<br>`threshold: f64 = 0.95` |

---

### 2.5 `extract_frames` — 抽幀到記憶體

| | |
|---|---|
| **struct** | `streaming::pipeline::ExtractFrames` |
| **ctor** | `ExtractFrames::new(strategy).with_scale(_).with_mode(_)` |
| **reads** | `source` (File) |
| **writes** | `extracted_frames` — `replace` |
| **params** | `strategy: enum = every_nth` \| `first_n` \| `range` \| `key_frames`<br>`strategyParam: usize`（`every_nth` / `first_n` 的 N，預設 30 / 10）<br>`rangeStart: usize` / `rangeEnd: usize`（`range` 專用）<br>`scaleFactor: f64`（0.0–1.0，不設＝原尺寸）<br>`frameMode: enum = seek` \| `sequential` ⚠️ 見下 |

> ⚠️ **`SamplingStrategy::Custom(Vec<usize>)` v1 不上架**（要一個陣列輸入 UI，不值得）。
> ⚠️ **`frameMode` 對應 `streaming::ExtractionMode { Seek, Sequential }`，
> 跟 2.6 的 `extractionMode` 是完全不同的 enum，只是同名。** 見 §5。

---

### 2.6 `extract_frames_to_disk` — 抽幀到磁碟

| | |
|---|---|
| **struct** | `video_process::pipeline::ExtractFramesToDisk` |
| **ctor** | `ExtractFramesToDisk::new(output_dir).interval(_).mode(_).save_mode(_)` |
| **reads** | `source` (File) |
| **writes** | `video_process_result` — `replace` |
| **params** | `outputDir: path`（必填）<br>`interval: usize = 30`（內部 `.max(1)`）<br>`extractionMode: enum = opencv_interval` \| `opencv_sequential` \| `ffmpeg` \| `ffmpeg_interval` \| `parallel` ⚠️<br>`saveMode: enum = single_directory` \| `multiple_directory` |

> **這是目前唯一有意義的上游節點**（見 §4）。它的 `output_dir` 是三條真鏈的起點。

---

### 2.7 `convert_to_hls` — 轉 HLS

| | |
|---|---|
| **struct** | `hls::pipeline::ConvertToHLS` |
| **ctor** | `ConvertToHLS::new(output_dir).segment_duration(_).playlist_name(_).force_keyframes(_).profile(_).level(_)` |
| **reads** | `source` (File) |
| **writes** | `hls_result` — `replace` |
| **params** | `outputDir: path`（必填）<br>`segmentDuration: u32 = 5`<br>`playlistName: string = "playlist.m3u8"`<br>`forceKeyframes: bool = true`<br>`profile: string = "baseline"`<br>`level: string = "3.0"` |

> **需要外部 ffmpeg**（`hls/converter.rs:57`）。manifest 標 `requires_ffmpeg: true`，
> UI 上掛一個小標記。

---

### 2.8 `capture_frame` — 擷取單幀

| | |
|---|---|
| **struct** | `camera::pipeline::CaptureFrame`（unit struct，無 builder） |
| **reads** | `source` (**File / Stream / Camera 都可以**——唯一一顆) |
| **writes** | `extracted_frames` — ⚠️ **`append`**（`push`，不覆寫） |
| **params** | 無 |

---

### 2.9 `annotate_frame` — 單張圖標註

| | |
|---|---|
| **struct** | `annotation::pipeline::AnnotateFrame` |
| **ctor** | `AnnotateFrame::new(input, output)` **或** `::with_context_source(output)` |
| **reads** | `source` (File)（`inputPath` 留空時） |
| **writes** | `annotation_result` — `replace` |
| **params** | `inputPath: path`（**留空＝用 `context.source`**，走 `with_context_source`）<br>`outputPath: path`（必填）<br>`annotationType: enum = filename` \| `timestamp`（`Custom(String)` v1 不上架）<br>`textPosition: enum = top_left` \| `top_right` \| `bottom_left` \| `bottom_right` \| `center` |

---

### 2.10 `annotate_video` — 影格目錄標註成影片

| | |
|---|---|
| **struct** | `annotation::pipeline::AnnotateVideo` |
| **ctor** | `AnnotateVideo::new(frames_dir, output)` **或** `::with_context_source(output)` |
| **reads** | `source` (File／目錄)（`framesDir` 留空時） → 見 §4「可吃上游」 |
| **writes** | `annotation_result` — `replace` |
| **params** | `framesDir: path`（**留空＝吃上游 `video_process_result.output_dir`，再留空＝用 `context.source`**）<br>`outputPath: path`（必填）<br>`annotationType` / `textPosition`（同 2.9）<br>`fps: i32 = 30`<br>`sourceFps: f64 = 30.0` |

> `execute()` 會用 `input_path.is_dir()` 自動判定 `FrameDir` vs `Video`
> （`annotation/pipeline.rs:200-216`）——所以吃上游目錄天生就通。

---

### 2.11 `process_files` — 檔案處理

| | |
|---|---|
| **struct** | `process::pipeline::ProcessFiles` |
| **ctor** | `ProcessFiles::new(output_dir).mode(_).overwrite(_)` |
| **reads** | `source` (File) → 見 §4 |
| **writes** | `process_result` — `replace` |
| **params** | `outputDir: path`（必填）<br>`processingMode: enum = single_file` \| `batch_files` \| `directory_process` \| `stream_process`<br>`overwrite: bool = false` |

---

## 3. 排除在 v1 之外的 step

| step | 為什麼排除 |
|---|---|
| `benchmark::pipeline::BenchmarkMetadataExtraction` | 實作的是 `PipelineStep<BenchmarkContext>`，**泛型參數不同**，進不了同一張圖 |
| `benchmark::pipeline::BenchmarkFrameExtraction` | 同上 |

要納入得先讓 `BenchmarkContext` 與 `MediaContext` 統一，那是 media-core 的工作（跨 repo）。
**v1 明確不做。**

---

## 4. ⚠️ 最重要的一節：現況的圖有多深

把 §2 的 `reads` / `writes` 攤開對照：

| 槽 | 誰寫 | 誰讀 |
|---|---|---|
| `source` | （來源節點） | 8 顆 step |
| `metadata` | `extract_metadata` | **沒有人** |
| `analysis` | `detect_motion` / `group_similar_images` / `compare_images` | **沒有人** |
| `extracted_frames` | `extract_frames` / `capture_frame` | **沒有人** |
| `hls_result` | `convert_to_hls` | **沒有人** |
| `annotation_result` | `annotate_frame` / `annotate_video` | **沒有人** |
| `video_process_result` | `extract_frames_to_disk` | **沒有人** |
| `process_result` | `process_files` | **沒有人** |

> **沒有任何一顆 step 讀另一顆 step 寫的槽。** 照現況直接畫，圖是一朵扇形，
> 深度永遠 1 層。

### 解法：把「手填路徑」變成「可以來自上游」

`group_similar_images.inputDir` 與 `annotate_video.framesDir`，在語意上完全可以是
`extract_frames_to_disk` 產出的目錄。做法是在 manifest 上多一個欄位：

```rust
// manifest.rs
pub struct ParamSpec {
    pub key: &'static str,
    pub kind: ParamKind,
    pub required: bool,
    /// 留空時可以從哪個槽的哪個欄位取值
    pub fallback_from: Option<SlotPath>,   // 例：SlotPath::VideoProcessOutputDir
}
```

`fallback_from` 有值的參數，在圖上**就是一個真的輸入埠**——picker 會把
`extract_frames_to_disk` 上架成它的合法上游，validate 會檢查那條邊。

### v1 真正接得起來的兩條路徑鏈

```
File → extract_frames_to_disk(out=A) → group_similar_images(in=A↑, out=B)   影片抽幀 → 相似分群
File → extract_frames_to_disk(out=A) → annotate_video(frames=A↑, out=v.mp4) 影片抽幀 → 標註成片
```

> ⚠️ **`process_files` 接不上**（2026-08-07 實作時推翻原本的判斷）。它的建構式只有
> `new(output_dir)`，輸入固定取自 `context.source.as_path_str()`——struct 上沒有任何
> 輸入欄位可以注入，registry 無從覆寫。要讓它吃上游得先有 `RebindSource`。

加上下面那顆 `save_report`（**Phase 0 已實作**），圖的實際深度是 **3 層**：

```
File ─┬→ extract_metadata ─────────────────────────┐
      ├→ detect_motion ───────────────────────────┤→ save_report
      └→ extract_frames_to_disk → group_similar_images ┘
```

再加上 fan-out（一個來源同時餵多顆）與 fan-in（`save_report` 收兩條入邊），圖是有
內容的。

### 補一顆就多一層（**Phase 0 已實作**，`src-tauri/src/graph/steps/save_report.rs`）

`metadata` / `analysis` 兩個槽沒有消費者，是最可惜的一點。在 **app 側**自己寫一顆
`PipelineStep<MediaContext>`（不用改 media-core，trait 是 pub 的）：

```rust
// src-tauri/src/graph/steps/save_report.rs
pub struct SaveReport { output_path: PathBuf, include: ReportParts }
// reads: metadata + analysis（至少一個）  writes: 無（純出口，寫檔）
```

有了它，`extract_metadata` 和 `detect_motion` 就有下游，圖上也終於有真正的**出口節點**
（參考文檔的 `sink` kind）。**已於 Phase 0 一起做掉**，順便驗證了「app 側能自訂 step、
media-core 零 diff」這條路走得通。

---

## 5. 踩雷清單（registry 實作時看這節）

| 雷 | 說明 |
|---|---|
| ⚠️ **`SaveMode::default()` 是 `MultipleDirectory`** | **實作時踩到，會靜默毀掉整個繼承鏈。** media-core 的 struct 預設把幀寫進 `{outputDir}/{video_name}/` 子目錄，於是 ① 它自己的計數只掃頂層 ⇒ `frames_extracted` 恆為 0；② 更嚴重：下游繼承到的 `output_dir` 是**幀目錄的父目錄**，`group_similar_images` 會在空目錄裡找圖。（`commands/pipeline.rs:449-452` 的 `_ => SingleDirectory` 是 app 側自己補的預設，不是 media-core 的。）**解法**：`registry.rs` 對 `saveMode` 明確補上 `single_directory` 預設——這是唯一一個「manifest 預設由 registry 施加」的例外，因為繼承契約要求 `output_dir` 必須就是幀所在的目錄 |
| **兩個 `ExtractionMode`** | `streaming::ExtractionMode { Seek, Sequential }` 與 `video_process::ExtractionMode { OpenCVSequential, OpenCVInterval, FFmpeg, FFmpegInterval, Parallel }` **同名、不同 crate 路徑、完全不同的值域**。manifest 用不同 key（`frameMode` / `extractionMode`）隔開，registry 的 `use` 要寫全路徑 |
| **`GroupSimilarImages::new` 的泛型** | `new<P: AsRef<Path>>(input_dir: P, output_dir: P)` — **兩個參數同一個 `P`**，傳 `&str` 和 `PathBuf` 混用會編不過。全部統一成 `PathBuf` 或全部 `&str` |
| **`AnnotateFrame::new` / `AnnotateVideo::new` 同上** | `new<P: AsRef<Path>>(input: P, output: P)` 同一個 `P` |
| **`threshold` 的三向分派** | `group_similar_images` 的 `threshold` 依 `method` 送到不同 setter，不是單一欄位 |
| **`interval` 有下限** | `ExtractFramesToDisk::interval()` 內部 `.max(1)`，UI 傳 0 不會炸但會變 1。前端也擋一次比較誠實 |
| **`CaptureFrame` 是 append** | 一張圖放兩顆會累積兩幀而不是互蓋——它是唯一不受「同一 method 只能一顆」限制影響的 step。v1 為了規則單純還是一起擋，記帳 |
| **`DetectMotion` 的 `output_dir` 預設是相對路徑** | `"output/motion_debug"`，相對於 process cwd。在 bundled app 裡 cwd 不可預期。manifest 應該把它標成必填、或預設成 `{app_data_dir}/composer/motion_debug` |
| **`convert_to_hls` 需要外部 ffmpeg** | macOS 從 Finder 啟動的 GUI app PATH 不含 `/opt/homebrew/bin`。`yarn tauri dev` 正常、打包後會炸。manifest 標 `requires_ffmpeg`，這是既有問題不是本案引入的 |

---

## 6. manifest 的 wire 形狀（給前端）

`list_graph_methods` 回傳的形狀（camelCase）：

```ts
interface MethodMeta {
  id: string;                 // 'extract_frames_to_disk'
  label: string;              // '抽幀到磁碟'
  group?: string;             // 家族（同家族在 picker 收合成一項）
  category: string;           // 'analysis' | 'extract' | 'convert' | 'annotate' | 'process'
  reads: SlotRef[];           // [{ slot: 'source', accepts: ['file'] }]
  writes: SlotRef[];          // [{ slot: 'videoProcessResult', mode: 'replace' }]
  params: ParamSpec[];
  requiresFfmpeg?: boolean;
}

interface ParamSpec {
  key: string;                // 'extractionMode'
  kind: 'bool' | 'int' | 'float' | 'string' | 'path' | 'dir' | 'enum';
  label: string;
  default?: unknown;
  required?: boolean;
  options?: { value: string; label: string }[];   // kind === 'enum'
  min?: number; max?: number;                     // kind === 'int' | 'float'
  fallbackFrom?: string;      // 'videoProcessResult.outputDir' — 留空時吃上游
}
```

**前端不硬編任何 step 名字。** 右欄檢視器照 `params` 畫（`kind` → 元件），picker 照
`reads` / `writes` 過濾。新增 step ＝ 只動 `manifest.rs` ＋ `registry.rs`，前端零改動。
這是參考文檔 §10 的鐵律，這裡照抄。

**實作後的實際形狀**（以 `manifest.rs` 的 serde 為準，與上面的草稿有幾處差異）：

- `reads` 的元素多了 `optional: bool` 與 `satisfiedByParam?: string`。
  `satisfiedByParam` 指的是「這個參數有值就不需要上游供給」（`group_similar_images`
  的 `inputDir` 就是靠它）；`optional` 給 `save_report` 那種「有就寫、沒有就少寫一段」
  的埠。
- `fallbackFrom` 序列化成 enum 名（目前唯一值 `"videoProcessOutputDir"`），不是
  草稿寫的 `"videoProcessResult.outputDir"` 路徑字串。
- 多了 `kind`（`source` / `method` / `sink`）與 `description`；`params` 每格多了
  選填的 `hint`。
- **所有面向使用者的字串是英文**，與 repo 其餘部分一致（原草稿寫「中文原句」是照抄
  參考專案的慣例，那個專案本身是中文介面，我們不是）。
