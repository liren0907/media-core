# TODO（待確認）

## Rust 後端（`media_core` 子模組已完成）
- [ ] 依賴指向：`src-tauri/Cargo.toml` 的 `media_core` 改成 `crates/media_core`（開發用 path），發版改 git tag/commit。
- [ ] 確認 backend 只作為 `media_core` 的接口層，不修改子模組程式碼。

## 現有功能修正（保持 backend 僅作接口）
- [ ] 對齊 HLS 路徑與參數命名（`output_dir`/`config_path` 等）確保前端 invoke 成功。
- [ ] 調整 `get_video_info` 回傳欄位或前端解析，避免顯示空白/錯誤。
- [ ] 修正 inferencer 的 annotation 載入（去除重複 JSON parse）。
- [ ] 移除硬編絕對路徑（`/Users/admin/...`），改為使用對話框或相對路徑。
- [ ] 審視安全設定：CSP 關閉、asset scope 為 `**`，評估最小化。

## 文件與確認
- [ ] README 補充：子模組初始化、依賴切換（path ↔ git）、發版 pin 版本。
- [ ] 若有 CI，確認子模組抓取與 build 流程可重現。
- [ ] 若調整命令介面（參數命名/回傳格式），補充簡短使用說明。

