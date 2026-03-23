**IMPORTANT**
ユーザーへの問いは必ず`askQuestions`で行う

**GLOBAL_CONSTRAINTS**
- library/ にドメイン知識（resolver, mapping解釈, doc_kind別例外）を絶対に入れない
- ドメイン判断は pipelines/ に閉じ込める
- nightly.py は判断しない（pipeline選択・実行のみ）
- 永続は storage/ のみ。workspace/ は消してよい
- HTTP取得は ETag 条件付きGET（If-None-Match）最優先、304なら以降全スキップ
- 保存パス規約は library/output/write.py だけが知る
- 実行は必ず uv run ...
- 提出前ゲート: uv run ruff format . / uv run ruff check . --fix / uv run ty check .
- 外部入力境界（HTTP/CSV/FS/CLI）は型注釈を厚めに
- 作業記録・エージェント連携はタスクチケットベースで行う

## 作業チケット

- 一連の作業は作業チケットを作成して管理する
- 作業チケットは`/tasks/<YYMMDD>.<作業名>.md`として作成する
- 作業チケットは簡潔に。おおむね150行以内で。
- 全作業完了時は作業チケットを`/tasks/done`に移動する
  - done移動時にはレビューレポートが出力済みであることを確認する
- 作業チケットのテンプレートは以下
- **「ユーザー指示」欄に記載がある場合は最優先指示として扱い、他の方針・計画より優先して従う**

```markdown
# チケット: <作業名>

## ユーザー指示
<!-- 通常は空欄。記載があれば最優先指示として扱う -->

## 1. 概要と方針

(概要と方針を1-3文で簡潔に記載)

## 2. 仕様
(この作業による仕様・仕様変更を記載)

## 3. シーケンス図
(mermaidシーケンス図を使用。重要な処理を明示。適宜rectによるフェーズ分け)

## 4. 設計
(設計の詳細を記載。必要に応じてクラス図やフローチャートを追加)

## 5. 考慮事項
（特に注意が必要な点や考慮すべき点、チェックすべき影響範囲などを記載）

## 6. 実装・テスト計画と進捗
（実装項目の細目を以下のタスクの形で記載。実装後にチェックをつけていく）
- [ ] <未完了タスク名>
- [x] <完了タスク名>

## 7. 品質要件チェック
レビュー: <レビューレポートのリンク>

(設計・実装完了後に確認する品質要件を列挙し、確認済みならチェックをつける)

## 8. まとめと改善提案
(作業完了後に振り返りを記載。次回同じ仕事を最初からより高品質に行うための改善提案があれば記載)

```

## 積み残し管理（issues.md）

- 今サイクルで対処しなかった「次にやるべきこと」は `/tasks/issues.md` に蓄積する
- **追記タイミング**: s.legend チェック後・reviewer 条件付き承認後・タスクチケット残件時
- **タスクチケット化したら**: issues.md の該当 Issue を「完了済み」テーブルに移動する
- issues.md は削除しない（永続的な積み残しリスト）

## エージェント連携

| エージェント | 役割 | 呼び出しタイミング | ツール |
|------------|------|------------------|--------|
| `s.pm` | タスク全体の調整 | 複数ステップのタスク開始時 | askQuestions, runInTerminal, getTerminalOutput, read, createFile, editFiles, search, agent, todo |
| `s.user-docs` | ユーザー文書作成進行 | ユーザー文書作成時 | askQuestions, read, createFile, editFiles, search, agent, todo |
| `s.architect` | 設計・方針決定 | 新機能・設計変更が必要な場合 | read, createFile, editFiles, search, agent |
| `s.researcher` | コンテキスト収集・調査 | 大規模調査・複数サブシステム分析時 | read, search, agent, fetch |
| `s.coder` | 実装・テスト | 設計完了後の実装作業 | read, edit, search, runInTerminal, getTerminalOutput, agent |
| `s.dev-reviewer` | コードレビュー | 実装完了後の品質確認 | read, createFile, editFiles, search, runInTerminal, getTerminalOutput |
| `s.legend` | 直感型リスク検知 | **s.pmが直接呼び出す**：①architect設計完了後（実装前）②reviewer差し戻し重大指摘時 | read, agent |

**並列実行ガイドライン**:
- 独立したタスクは並列実行を推奨（最大10並列）
- pmが並列実行を調整し、サブエージェントに並列実行対象を指示
- タスクチケットで並列実行対象を明示
- 複数のresearcherインスタンスを独立したサブシステム調査に並列利用可能
- 明確に分離された機能のcoderは並列実行可能
- 独立したフェーズのreviewerは並列実行可能

**引き継ぎのポイント**
- 各エージェントは作業完了時、次エージェントに必要な情報を明示する
- 不明点があれば自己判断せずユーザーに確認する
- タスクチケットを共通の情報源として活用する

## 設計ガイドライン
設計を行う場合は、`s.architect.agent.md`を参照

## 実装ガイドライン
実装を行う場合は`s.coder.agent.md`を参照

## テストガイドライン
テストを行う場合は`s.coder.agent.md`を参照

## レビューガイドライン
レビューを行う場合は`s.dev-reviewer.agent.md`を参照
