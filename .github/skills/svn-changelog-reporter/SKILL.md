---
name: svn-changelog-reporter
description: SVN変更履歴を設計観点で解析し、Markdownレポートを生成/追記。
---

# SVN Changelog Reporter

SVN変更履歴を設計的観点で解析し、**このブランチの累積サマリレポート** (`changelog.md`) を管理します。

## ワークフロー

1. **詳細レポート生成**
   ```powershell
   # srcディレクトリの変更履歴を解析（デフォルト）
   .github\skills\svn-changelog-reporter\scripts\analyze_svn_changes.ps1
   ```
   - 未記録リビジョンの詳細diff情報を `{Target}/detail_rXXXXX-rYYYYY.md` に生成
   - changelog.mdと詳細レポートはSVN管理単位のルート（src/直下、docs/直下など）に出力

2. **LLMによるサマリ追記**
   - `{Target}/detail_rXXXXX-rYYYYY.md` を精読して実際の変更内容を把握
   - メソッド、クラス、インタフェース変更を特定
   - 複数コミットを設計的テーマでグルーピング
   - 変更意図（Why）を分析
   - `{Target}/changelog.md` の**先頭に新しい週のセクションを追加**もしくは既に存在する週に追記
   - **リビジョン番号の連続性を確認**して取りこぼし防止

3. **フォーマット検証**（オプション）
   ```powershell
   # srcのchangelog.mdを検証（デフォルト）
   .github\skills\svn-changelog-reporter\scripts\validate_changelog_format.ps1
   ```
   - `{Target}/changelog.md` の見出し構造と必須セクションを検証
   - フォーマット違反を警告

4. **クリーンアップ**（サマリ作成後）
   ```powershell
   # srcの詳細レポートを削除（デフォルト）
   .github\skills\svn-changelog-reporter\scripts\cleanup_detail_reports.ps1
   ```
   - 不要になった詳細レポートファイルを削除

## サマリレポート形式

**ファイル**: `{Target}/changelog.md` （例: `src/changelog.md`, `docs/changelog.md`）  
**累積管理**: 新しい週のセクションを先頭に追記（新しい順）

**必須構造**:
```markdown
# SVN変更履歴サマリレポート（設計的観点）

## 2026-01-19(Mon) - 2026-01-25(Sun)

**3行サマリ**
{この週の重要な変更概要を簡潔に}

**変更一覧**
| revision | author | date       | category | message                        |
|----------|--------|------------|----------|--------------------------------|
| r12346   | user1  | 2026-01-25 | GUM      | Added logging interface        |
| r12345   | user1  | 2026-01-24 | APP      | Added logging interface        |

### {テーマ1} (rxxxx, rxxxx, ...)
**変更**: {この変更の概要}
**意図**: {この変更の設計意図。理由や背景}

### {テーマ2} (rxxxx, rxxxx, ...)
...（以下同様）

## 2026-01-12(Mon) - 2026-01-18(Sun)

...（以下同様）
```

**見出しレベル**:
- `#` 1つ: タイトルのみ
- `##` 2つ: 週の期間（`YYYY-MM-DD(Mon) - YYYY-MM-DD(Sun)`形式）
- `###` 3つ: テーマ（自由）

5. **ユーザーへ提示** → 口頭で簡潔に要約（ファイルをそのまま貼らない）

## 重要事項

- サマリレポートは**必ずLLMが詳細ログを解析して作成**すること
- サマリレポートは**累積的に管理**し、新しい週のセクションを先頭に追記
- **ファイル出力先**: SVN管理単位のルート（src/、docs/など）直下に出力
  - `src/changelog.md` ← srcの変更履歴
  - `src/detail_rXXXXX-rYYYYY.md` ← srcの詳細レポート
  - `docs/changelog.md` ← docsの変更履歴
  - `docs/detail_rXXXXX-rYYYYY.md` ← docsの詳細レポート
- スクリプトは詳細diff生成のみ担当（サマリはLLMが作成）
- **リビジョン取りこぼし防止**: 
  - スクリプトがchangelog.mdから最新リビジョンを抽出
  - 未記録リビジョンを特定し、ギャップがあれば警告
  - LLMは変更一覧テーブルで連続性を再確認
- サマリ形式の厳守: 見出しレベルは検証スクリプトで確認可能
- 目標: 週単位でセクション化、週ごとに100行程度となる設計的分析
- 機械的列挙ではなく洞察に富む分析を重視

## スクリプト責務

| スクリプト | 責務 | タイミング | 引数 |
|-----------|------|-----------|------|
| `analyze_svn_changes.ps1` | 詳細diffレポート生成 | サマリ追記前 | `-Target src` (デフォルト)  |
| `validate_changelog_format.ps1` | changelogフォーマット検証 | サマリ追記後（オプション） | `-Target src` (デフォルト)  |
| `cleanup_detail_reports.ps1` | 詳細レポート削除 | サマリ追記後 | `-Target src` (デフォルト)  |

## 改善内容（2026-01-30）

### 1. 複数ターゲット対応
- `-Target` パラメータで `src` または `docs` を指定可能
- 各SVN管理単位のルート直下にchangelog.mdと詳細レポートを出力
- 将来的な拡張性を確保

### 2. 文字コード問題の解決
- スクリプトをUTF-8（BOM なし）で統一
- `#requires -Version 5.1` とエンコーディング明示
- 日本語メッセージを英語化してパーサエラーを回避
- 出力ファイルもUTF-8（BOM なし）で保存

### 3. SVN検出ロジックの改善
- 現在地から上方向へ `.svn` フォルダを検索
- `svn info` の出力を厳密にパース
- フォールバック機制を強化
- ターゲットディレクトリベースの検出に対応

### 4. パス構築の修正
- `$designDir` 未定義エラーを解消
- SVNルート直下に出力するように変更
- changelog.mdの場所を `{Target}/changelog.md` に統一
- 詳細レポートも同じ場所に出力

### 5. エラーハンドリングの強化
- より明確なエラーメッセージ
- SVN未検出時の適切なガイダンス
- 処理ステップの詳細なログ出力
