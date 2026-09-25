# PLAN — 把 media-core 重組成 monorepo（併入 media-app，拆掉 submodule）

## 這份文檔是什麼（2026-09-26）

**搬遷計畫**：以 **media-core repo 為容器**，把 media-app 的完整歷史併進來，重組成
「Cargo workspace ＋ Tauri app」的 monorepo。完成後 **media-core 就是唯一的開發 repo**，
media-app 封存不再使用。

這是 PLAN 不是 REFERENCE。§3 的 M1–M5 已拍板，**M6、M7 仍待拍板**（步驟暫照建議排）。
搬完後應改寫成 as-built，並把 `.claude/CLAUDE.md` 更新成新結構。

本文檔寫在 media-app 的 `docs/`，會在階段 0 跟 composer 一起 commit；搬完後它會出現在
新 repo 的根目錄 `docs/`。

**分工：**

| 角色 | 負責 |
|---|---|
| 你 | 所有 `git commit`、`git merge`（merge 本身會產生 commit）、`git push`、GitHub 設定、刪除本機目錄、開新的 Claude Code session |
| Claude | 搬檔、改設定檔、改寫歷史（在暫存目錄）、跑驗證、回報結果；每個 commit 前停下來給你 commit message |

> 若 Claude 在某個目錄跑 git 被權限擋下（這個 session 在 submodule 內被擋過），改由你執行，Claude 提供指令。

---

## 1. 一句話定位

**media-core repo 從「一個 crate」長成「一個 workspace」**：core 和其他函式庫並排在
`crates/`，Tauri app 住在 `app/`，改 core 就是一般的一個 commit，不再有 submodule 和 upstream。

---

## 2. 為什麼要做：現況盤點

submodule 適合「多人、core 有多個使用者、兩邊要各自發版」。目前是一個人寫兩邊、core 只有
app 一個使用者，它只剩成本。以下全部是 2026-09-26 實查結果：

| 現象 | 證據 |
|---|---|
| core 的版本指標 8 個月沒動 | media-app 從第一個 commit（2026-01-18）起 gitlink 一直是 `7e25dde`（＝ tag `v0.4.0` ＝ remote `main`）；core 在 GitHub 最後一次 push 也是 01-18 |
| 唯一一次該 upstream 的修改沒回去 | submodule 工作目錄裡 `opencv = "0.93.5"` → `0.98` 只存在本機（mtime 2026-06-07，和 app／annotation 在 `b52fd3c` 升級同一天），不在任何 commit、任何 remote 分支 |
| 核心級程式碼在 app 裡增生 | `audio-core` 開在 app；core 已有 `annotation` 模組，app 又另長一個 `annotation` crate；dedup 的 pHash（`src-tauri/src/hasher/perceptual.rs`，img_hash）與 core 的 `analysis/hashing/hasher.rs`（opencv）是兩套；composer 的 manifest／registry 放 app 側（拍板 D1 的理由正是「core 是 submodule」） |
| core 沒有其他使用者 | `~/Desktop/code` 底下只有 app 引用；core 自己的 `examples/media_core_app` 還指向舊 repo 名 `rtsp_stream_extractor` |
| 容易忽略的髒狀態 | `git status --short` 只用小寫 ` m` 表示 submodule 內有修改 |

**與搬遷直接有關的現況：**

| 項目 | 狀態 |
|---|---|
| GitHub 公開狀態 | 兩個 repo 都是 public ⇒ 合併不會外流任何東西 |
| 大小 | media-core 約 277 KB；media-app 約 64.6 MB（主要是 vendored whisper-rs）⇒ 合併後約 65 MB |
| media-app 分支 | 本機 `dev`（＝ `feat/graph-editor` 目前的 HEAD `3991a9d`）有 2 個 commit 沒 push：`ac1083f`、`3991a9d`。`main` ＝ `origin/main` ＝ `86f5ec4`，是 `dev` 的祖先（可 fast-forward） |
| media-app 未 commit | composer Phase 0＋1 共 20 個檔、本文檔 |
| 本機 media-core clone（`~/Desktop/code/media-core`） | 停在過期的 `dev`；最後 fetch 2026-02。本機分支 `dev`／`feature-pipeline`／`main` 全部已在 GitHub（`feature-pipeline` 落後 main 9、領先 0），無 stash |
| 本機 media-core 未追蹤檔 | `data/`（**1.5 GB**，core 的 tests 用 `data/test.mp4` 相對路徑）、`references/`（627 MB）、`config/`、`.vscode/`、`Cargo.lock`（被 core 的 `.gitignore` 忽略）；`docs/` 底下有 4 個被忽略的子目錄 |
| `.claude/` | media-app 的 `.claude/`（`CLAUDE.md`、`launch.json`、`settings.local.json`）被 `.gitignore` 排除，**不在 git 裡**，不會隨歷史過去 |
| Claude memory | 以資料夾路徑為 key：`-Users-admin-Desktop-code-media-app`；`-Users-admin-Desktop-code-media-core` 目前不存在 |
| 工具 | `git-filter-repo` 已安裝（`/opt/homebrew/bin/git-filter-repo`） |

---

## 3. 決策

| # | 題目 | 決定 | 理由 | 狀態 |
|---|---|---|---|---|
| M1 | 要不要合併 | **合併** | §2 每一條都指向同一件事：upstream 的摩擦讓核心程式碼長在 app，重複實作會越來越多 | ✅ |
| M2 | 誰當容器 | **保留 media-core repo，把 media-app 歷史併進來；media-app 封存** | media-app 先全部 merge、push 後就不再變動，改寫它的歷史和改寫 core 一樣安全；保留 media-core 則 repo 名稱、網址、`v0.1.0`–`v0.4.0` tag 全部原封不動，符合「core 為主」。media-app 封存後舊 commit 編號仍查得到 | ✅（改判，原建議是保留 media-app） |
| M3 | 版面 | **virtual workspace 根 ＋ `crates/` ＋ `app/`**（§4） | 若 core 留在根目錄，它的 `Cargo.toml` 就是 workspace 根，`[profile.release] panic = "abort"` 會套用到 app（R3）；core 的 12 個 bin 也會和 `app/` 擠在同一層 | ✅ |
| M4 | 時機 | **composer 原樣收尾、推上去之後才搬** | 未 commit 的檔案和大搬家混在一起，出事很難拆；composer review 的 14 項發現搬完後在新 repo 修 | ✅ |
| M5 | media-app 的 `main` | **fast-forward 追上 `dev`，兩條都 push** | 封存前兩條分支停在同一點，封存的就是完整最終狀態 | ✅ |
| M6 | core 的 `[profile.*]` | 建議：**從 core 的 `Cargo.toml` 刪掉，不提到 workspace 根；在 core README 記一筆原因**。不採「提到根目錄」 | 這些設定現在本來就沒生效（依賴的 profile 一律被忽略），刪掉只是明文化；若提到根目錄，`panic = "abort"` 會讓 `commands/graph.rs` 的 `handle.join()` 接不到 panic，整個 app 閃退 | ⏳ 待拍板 |
| M7 | 根目錄要不要放代理用的 `package.json` | 建議：**不放**，前端指令一律在 `app/` 下執行。不採「放一個只轉發 `yarn --cwd app …` 的 `package.json`」 | 根目錄保持「只有 Cargo workspace ＋ docs」；之後嫌麻煩再加，成本很低 | ⏳ 待拍板 |

---

## 4. 目標版面

```
media-core/                      ← repo 根（GitHub liren0907/media-core；本機 ~/Desktop/code/media-core）
  Cargo.toml                     ← 只有 [workspace]；opencv 等共用版本寫在這
  Cargo.lock                     ← 沿用 media-app 的 src-tauri/Cargo.lock
  README.md                      ← 新寫：monorepo 總覽與怎麼跑
  .gitignore                     ← 新寫：只放 repo 層級的規則
  crates/
    media-core/                  ← 原本在根目錄的 core（含自己的 .gitignore、docs/、tests/、data/）
    annotation/                  ← 來自 media-app 的 src-tauri/crates/
    audio-core/                  ← 同上
    whisper-rs/                  ← 同上（含 sys/，是 path 依賴，自動成為 workspace 成員）
  app/                           ← 原本的 media-app（Tauri）
    package.json  yarn.lock  svelte.config.js  vite.config.js  tsconfig.json  components.json
    README.md  .gitignore
    src/  static/  scripts/
    src-tauri/                   ← Tauri 後端 crate（package 名 media-app）
  docs/                          ← 來自 media-app 的 docs/（含本文檔）
  .claude/                       ← 不在 git；從 media-app 手動複製後改寫
```

### 每樣東西從哪來、怎麼搬

| 來源 | 目的地 | 怎麼搬 |
|---|---|---|
| core 根目錄的追蹤檔：`src/` `tests/` `examples/` `script/` `docs/` `Cargo.toml` `README.md` `.gitignore` | `crates/media-core/` | `git mv`（C1）；`docs/` 是整個目錄搬，被忽略的子目錄會一起走 |
| core 本機未追蹤：`config/` `data/` `references/` | `crates/media-core/` 同名位置 | 一般 `mv`（C1）；搬進去後仍被 core 的 `.gitignore` 忽略；tests 的 `data/test.mp4` 相對路徑才對得上 |
| core 本機未追蹤 `Cargo.lock` | 刪除 | 由 app 的 `Cargo.lock` 取代（C1 先刪，否則 C3 的 `git mv` 會撞到） |
| media-app 全部歷史 | `app/` | 在暫存目錄用 `git filter-repo` 改寫（階段 2） |
| media-app 的 `src-tauri/crates/{annotation,audio-core,whisper-rs}` | `crates/` | 同上，改寫時直接放到位，`git log crates/annotation` 看得到完整歷史 |
| media-app 的 `docs/` | 根目錄 `docs/` | 同上 |
| media-app 的 submodule 指標（歷史裡共 3 個路徑）與 `.gitmodules` | 移除 | 同上，從每個歷史 commit 裡拿掉 |
| media-app 的 `src-tauri/Cargo.lock` | 根目錄 `Cargo.lock` | `git mv`（C3），沿用既有鎖定版本 |
| media-app 的 `.claude/` | 根目錄 `.claude/` | 手動複製後改寫（C4） |
| Claude memory | `~/.claude/projects/-Users-admin-Desktop-code-media-core/memory/` | 階段 0 先複製，C4 後改寫內容 |
| media-app 本機未追蹤 `data/` `deprecated/` `reference/` | 你決定 | 手動；不搬也不影響建置 |

### 根目錄 `Cargo.toml` 草稿

```toml
[workspace]
resolver = "2"                   # virtual workspace 預設是 "1"，必須明寫才維持現行行為
members = ["crates/*", "app/src-tauri"]
exclude = ["crates/media-core/examples/media_core_app"]   # 指向舊 repo 名，本來就壞

[workspace.dependencies]
opencv = { version = "0.98", default-features = false, features = ["features2d", "highgui", "imgcodecs", "imgproc", "video", "videoio"] }
```

用到 opencv 的 3 個 crate（`crates/media-core`、`crates/annotation`、`app/src-tauri`）改成
`opencv.workspace = true`。**opencv 0.98 的升級就在這裡正式進入 core 的 commit**——
submodule 工作目錄裡那份未 commit 的修改不需要另外保存。不放任何 `[profile.*]`（M6）。

### 根目錄 `.gitignore` 草稿

```gitignore
# OS / editor
.DS_Store
.vscode/
.idea/

# Rust（workspace 的 target 在根目錄）
/target/

# Claude Code
.claude/
```

`node_modules/`、`.svelte-kit/`、`build/` 等前端規則由 `app/.gitignore` 負責；core 的媒體檔、
`data/` 等規則由 `crates/media-core/.gitignore` 負責（巢狀 `.gitignore` 只作用在自己目錄以下）。

---

## 5. 步驟

### 階段 0：media-app 最後一次操作（在 `~/Desktop/code/media-app`）

| # | 誰 | 做什麼 |
|---|---|---|
| 0.1 | 你 | commit composer Phase 0＋1 與本文檔（兩個 commit；staging 仍排除 submodule：`git add -A -- . ':(exclude)src-tauri/crates/media-core'`） |
| 0.2 | 你 | `git switch dev && git merge --ff-only feat/graph-editor` → `git switch main && git merge --ff-only dev` → `git push origin dev main` |
| 0.3 | 你 | 確認 submodule 裡沒有本機獨有的 commit：`git -C src-tauri/crates/media-core log --branches --not --remotes` 應為空 |
| 0.4 | Claude | 把 memory 複製到 `-Users-admin-Desktop-code-media-core/memory/`，並在裡面加一筆「搬遷進行中，計畫在 `~/Desktop/code/media-app/docs/PLAN_monorepo_migration.md`」 |
| 0.5 | 你 | 在 `~/Desktop/code/media-core` 開一個新的 Claude Code session；之後的階段都在那裡做 |

✅ 關卡：`origin/main` ＝ `origin/dev` ＝ 本機 `feat/graph-editor` 的 HEAD。**從這一刻起 media-app 不再有任何新 commit。**

### 階段 1：準備 media-core 本機 clone（在 `~/Desktop/code/media-core`）

| # | 誰 | 做什麼 |
|---|---|---|
| 1.1 | 你 | `git fetch origin` → `git switch main` → `git pull --ff-only` |
| 1.2 | 你 | `git status` 應只有被忽略的未追蹤檔（`data/` 等），沒有修改 |
| 1.3 | 你 | `git tag pre-monorepo`（只留本機，回滾用）→ `git switch -c chore/monorepo` |

### 階段 2：在暫存目錄改寫 media-app 的歷史（Claude；不碰任何正式 repo）

分三次跑，每次的改名規則互不重疊，不依賴 filter-repo 的套用順序：

| # | 做什麼 |
|---|---|
| 2.1 | 全新 clone：`git clone https://github.com/liren0907/media-app.git <scratchpad>/app-import`（filter-repo 第一次跑必須是全新 clone） |
| 2.2 | 拿掉 submodule 痕跡：`git filter-repo --path src-tauri/crates/media-core --path src-tauri/crates/media_core --path src-tauri/crates/media_core_bak --path .gitmodules --invert-paths`（歷史裡共有 3 個 gitlink 路徑：`media-core` 29 個 commit、03-25 改名前的 `media_core` 7 個、早期殘留的 `media_core_bak` 6 個；已確認沒有任何 commit 會因此變空） |
| 2.3 | 整包搬進 `app/`：`git filter-repo --force --to-subdirectory-filter app` |
| 2.4 | 函式庫與文件拉到最上層：`git filter-repo --force --path-rename app/src-tauri/crates/annotation/:crates/annotation/ --path-rename app/src-tauri/crates/audio-core/:crates/audio-core/ --path-rename app/src-tauri/crates/whisper-rs/:crates/whisper-rs/ --path-rename app/docs/:docs/` |

✅ 關卡：
1. `git ls-tree --name-only HEAD` 只有 `app`、`crates`、`docs`
2. `git ls-tree --name-only HEAD crates/` 只有 `annotation`、`audio-core`、`whisper-rs`
3. 整段歷史沒有任何 gitlink：`git rev-list --all | xargs -I{} git ls-tree -r {} | grep -c '^160000'` 為 0；`git log --all --oneline -- .gitmodules app/.gitmodules` 為空
4. `git log --oneline -- crates/whisper-rs` 看得到它最早進 repo 的那個 commit
5. commit 數和 media-app `main` 相同（`git rev-list --count HEAD`）

> 暫存目錄是 session 級的。2.1 到 C2 的 fetch 請在同一個 session 內完成；真的遺失就重跑階段 2（可重現）。

### 階段 3：在 `chore/monorepo` 上重組（4 個 commit，每個都要過關卡）

**C1 — core 搬進 `crates/media-core/`**

| # | 誰 | 做什麼 |
|---|---|---|
| 3.1 | Claude | `mkdir -p crates/media-core` → `git mv .gitignore Cargo.toml README.md docs examples script src tests crates/media-core/` |
| 3.2 | Claude | `mv config data references crates/media-core/`（未追蹤，約 2.1 GB，同一顆磁碟上只是改名，很快）；刪掉根目錄未追蹤的 `Cargo.lock` |
| 3.3 | 你 | commit — 建議訊息：`refactor(repo): move media-core crate into crates/media-core` |

✅ 關卡：`git status` 乾淨；`git log --follow --oneline crates/media-core/src/lib.rs` 看得到 2025-04 起的歷史；`crates/media-core/data/test.mp4` 存在。
（這一步不跑 `cargo`：core 此時還是 opencv 0.93.5，在這台機器上本來就可能編不過。）

**C2 — 併入 media-app 歷史**

| # | 誰 | 做什麼 |
|---|---|---|
| 3.4 | 你 | `git remote add app-import <scratchpad>/app-import` → `git fetch app-import` → `git merge --allow-unrelated-histories app-import/main` — 建議把 merge 訊息改成：`chore(repo): import media-app history into app/` |

✅ 關卡：根目錄有 `app/`、`crates/`（4 個 crate）、`docs/`；`git log --oneline -- app/src-tauri/src/graph` 看得到 composer 的 commit；`git blame app/src/lib/nav.ts` 顯示原作者與原日期。
（這一步也還不能編：`app/src-tauri/Cargo.toml` 的 path 依賴還指著舊位置，C3 修。）

**C3 — Cargo workspace**

| # | 誰 | 做什麼 |
|---|---|---|
| 3.5 | Claude | 新增根目錄 `Cargo.toml`（§4 草稿）；`git mv app/src-tauri/Cargo.lock Cargo.lock` |
| 3.6 | Claude | `app/src-tauri/Cargo.toml` 的 4 個 path 依賴改成 `../../crates/…` |
| 3.7 | Claude | 3 個 crate 的 opencv 改 `opencv.workspace = true`；刪掉 `crates/media-core/Cargo.toml` 的 `[profile.release]`、`[profile.dev]`（M6） |
| 3.8 | Claude | 新增根目錄 `.gitignore`（§4 草稿） |
| 3.9 | 你 | commit — 建議訊息：`build(workspace): add cargo workspace with shared lockfile and opencv 0.98` |

✅ 關卡（依序，前一個過了才跑下一個）：
1. `cargo check -p media-app` 0 errors
2. `cargo test -p media-app --bin media-app graph::` 26 passed
3. `cargo build --workspace` ——**全案最可能出事的一步**（R1、R2）
4. `Cargo.lock` 的 diff 只應該是「新增 core bins／tests 需要的條目」，既有套件版本不變
5. （選做）`cargo test -p media_core`：用 `crates/media-core/data/test.mp4`，失敗先記帳不擋搬遷

> 第一次在根目錄 build 會全部重編（target 目錄換位置，opencv 綁定生成要好幾分鐘）。

**C4 — app 的路徑、工具、文件**

| # | 誰 | 做什麼 |
|---|---|---|
| 3.10 | Claude | `app/package.json` 的 `TAURI_DEV_WATCHER_IGNORE=src-tauri/crates/**`：路徑已失效，實測後刪除或改寫（R4） |
| 3.11 | Claude | `app/.gitignore` 刪掉已失效的 `src-tauri/crates/media-core/.git/`、`/src-tauri/target/` 兩行 |
| 3.12 | Claude | 新寫根目錄 `README.md`（monorepo 總覽、目錄說明、怎麼跑）；`crates/media-core/README.md` 加一段「profile 設定已移除」的說明（M6） |
| 3.13 | Claude | 不在 git 的設定：從 media-app 複製 `.claude/` 到根目錄後改寫——`launch.json` 改成 `yarn --cwd app dev --port 1530 --strictPort`；`CLAUDE.md` 改成新結構與新指令；`settings.local.json` 刪掉寫死舊路徑的 allow 規則 |
| 3.14 | 你 | 在 `app/` 下 `yarn install` |
| 3.15 | 你 | commit — 建議訊息：`chore(app): fix paths and tooling after moving into app/` |

✅ 關卡：
1. `yarn --cwd app run check` ＝ 0 errors / 20 warnings（baseline 不變）
2. `cargo check -p media-app` 0 errors
3. 你在 `app/` 下跑 `yarn tauri dev`：app 能開；`/composer` 能 Run；`/audio`（whisper-rs 搬過）、`/annotator`（annotation 搬過）、dedup 各點一下
4. 改一個 `crates/media-core/src/` 的檔：`tauri dev` 會重編、且不會無限重編（R4）

### 階段 4：併回主線（在 media-core）

| # | 誰 | 做什麼 |
|---|---|---|
| 4.1 | 你 | `git switch main && git merge --ff-only chore/monorepo` → `git push origin main` |
| 4.2 | 你 | media-core 的 `dev` 是過期殘留（落後 7、領先 0）：fast-forward 到 `main` 再 push，之後照你在 media-app 的 dev → main 流程走 |
| 4.3 | 你 | `git remote remove app-import`；刪掉已合併的本機分支 `feature-pipeline`、`chore/monorepo` |

### 階段 5：收尾（不急，可以隔幾天）

| # | 誰 | 做什麼 |
|---|---|---|
| 5.1 | 你 | media-app 的 README 加一行「已併入 liren0907/media-core」，push 後在 GitHub 設 archive |
| 5.2 | Claude | 改寫新位置的 memory：路徑全部更新；「staging 排除 submodule」規則作廢；composer 的 14 項 review 帳本沿用 |
| 5.3 | Claude | `docs/` 裡寫死 `src-tauri/…` 路徑或提到 submodule 的文件（約 9 份）更新；`crates/media-core/docs/RELEASE_WORKFLOW.md` 檢查是否還適用；本文檔改寫成 as-built |
| 5.4 | 你 | 確認一切正常後刪本機 `~/Desktop/code/media-app`；舊的 memory 目錄可一併刪 |
| 5.5 | 你 | 刪掉 `pre-monorepo` tag（本機） |

---

## 6. 回滾

| 做到哪 | 怎麼回 |
|---|---|
| 階段 0 | 都是一般的 commit／merge／push，media-app 照常可用 |
| 階段 1–2 | media-core 只多了一條空分支和一個本機 tag；暫存目錄刪掉即可 |
| 階段 3（任何一個 commit，尚未 push） | 先把 `crates/media-core/{config,data,references}` `mv` 回根目錄（未追蹤檔不會跟著切分支）→ `git switch main` → `git branch -D chore/monorepo` → `git remote remove app-import` |
| 階段 4 之後 | media-core 的 `main` 已改變；最乾淨的退路是 media-app 本身——它完整保留、只是封存，GitHub 上可以隨時取消 archive 回去繼續開發 |

---

## 7. 風險

| # | 風險 | 在哪一步現形 | 對策 |
|---|---|---|---|
| R1 | core 的 12 個 bin 與 tests 從沒用 opencv 0.98 編過（app 只用到 core 的 lib） | C3 關卡 3 | 個別修；修不動的 bin 先在 core `Cargo.toml` 標 `required-features` 暫時關掉，記帳，不擋搬遷 |
| R2 | workspace 一起 build 時 Cargo 的 feature 合併結果會變；之前 bindgen × opencv 就是栽在這（`docs/AUDIO_ARCHITECTURE.md`） | C3 關卡 3 | whisper-rs 已拿掉 bindgen，風險低；若復發，比較 `cargo tree -e features` 前後差異 |
| R3 | core 的 `[profile.release]` 有 `panic = "abort"`、`lto = "fat"`；若成為 workspace 根的設定就會生效 | 若 core 留在根目錄或誤搬 profile | M3 讓 core 不在根目錄；M6 直接刪掉 |
| R4 | `TAURI_DEV_WATCHER_IGNORE=src-tauri/crates/**` 是為了避開 submodule 內 `.git` 造成的 watcher 迴圈；路徑搬走後失效，且 workspace 成員可能被 `tauri dev` 監看 | C4 關卡 4 | 巢狀 `.git` 已不存在，原因消失；實測「改 core 會重編、不會無限重編」後決定刪或改 |
| R5 | core 的 `.gitignore` 有 `*.png`、`*.jpg`、`assets/`、`build/`、`config/`…；若留在根目錄會**默默忽略 app 新增的圖檔**（例如 `src-tauri/icons/`） | C1 | 它跟著 core 一起搬進 `crates/media-core/`，只管 core 自己；根目錄另寫一份（§4） |
| R6 | core 的 bin 名（`annotation`、`metadata`、`process`…）和其他 package 的輸出檔名撞到 | C3 關卡 3 | Cargo 會出 filename collision warning；有就改 bin 名 |
| R7 | 暫存目錄被清掉，`app-import` 消失 | 階段 2 到 C2 之間 | 同一個 session 內做完；遺失就重跑階段 2 |
| R8 | 2.1 GB 未追蹤資料沒跟著搬，core tests 找不到 `data/test.mp4` | C1 | 3.2 明確搬移；C1 關卡檢查檔案存在 |
| R9 | filter-repo 的改名規則互相干擾 | 階段 2 | 拆成三次跑、每次規則不重疊；階段 2 關卡逐條驗 |
| R10 | 新資料夾的 Claude session 讀不到舊 memory | 0.5 開新 session 時 | 0.4 先複製 memory |

---

## 8. 搬完之後的日常

| 事情 | 以前 | 以後 |
|---|---|---|
| 工作資料夾 | `~/Desktop/code/media-app` | `~/Desktop/code/media-core` |
| 開發 app | 根目錄 `yarn tauri dev` | `app/` 下 `yarn tauri dev` |
| 型別檢查 | 根目錄 `yarn run check` | `yarn --cwd app run check` |
| Rust 檢查／測試 | `src-tauri/` 下 `cargo …` | 根目錄 `cargo … -p media-app`；core 單獨 `cargo test -p media_core` |
| 改 core | submodule 內 commit → push → app 側 bump 指標 → 再 commit | 一般的一個 commit |
| staging 規則 | `git add -A -- . ':(exclude)src-tauri/crates/media-core'` | 排除規則作廢 |
| 版本 tag | core 的 `v*` 只標 core | `v*` 繼續用，但標記的是整個 repo 的狀態 |

---

## 9. 刻意不做（搬遷只做搬遷）

- **不合併重複實作**：兩套 pHash、兩個 annotation 原封不動，搬完再逐項評估。
- **不移動 composer 的 `graph/`**：拍板 D1「manifest／registry 放 app 側」的前提（core 是 submodule）搬完後就不成立，但要不要搬進 core 另案討論。
- **不修 composer 的 14 項 review 發現**（M4）。
- **不動 edition**：core 維持 2024，其餘維持 2021。
- **不改 crate 版本號、不打新 tag**：要用 `v0.5.0` 標記 monorepo 起點可以，但不在本案範圍。
- **不發佈 crates.io**。
- **不修 core 的過期內容**：`examples/media_core_app` 只做 `exclude`；README 裡的 `rtsp_stream_extractor` 舊名記帳另案處理。

---

## 10. 拍板紀錄

| 日期 | # | 決定 |
|---|---|---|
| 2026-09-26 | M1 | 合併兩個 repo |
| 2026-09-26 | M2 | 以 media-core 為容器、media-app 封存（**改判**：原建議保留 media-app，因「media-app 先全部 merge 完再凍結」後原理由不成立） |
| 2026-09-26 | M3 | virtual workspace 根 ＋ `crates/` ＋ `app/` |
| 2026-09-26 | M4 | composer 原樣收尾、推上去後再搬 |
| 2026-09-26 | M5 | media-app 的 `main` fast-forward 追上 `dev` |
| — | M6 | 待拍板 |
| — | M7 | 待拍板 |
