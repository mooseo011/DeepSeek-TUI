# 🐳 DeepSeek TUI（mooseo011 フォーク）

> **DeepSeek V4 向けターミナルネイティブなコーディングエージェント。
> 新規追加の `/swarm` オーケストレーターがタスクを並列のワーカー
> サブエージェントに分配します。100 万トークンのコンテキストと
> プレフィックスキャッシュ最適化を踏襲し、MCP クライアント、
> サンドボックス、永続タスクキューを同梱した自己完結型 Rust バイナリ。**

[English README](README.md)
[简体中文 README](README.zh-CN.md)

> [!IMPORTANT]
> **これはフォークです。** リポジトリは
> [github.com/mooseo011/DeepSeek-TUI](https://github.com/mooseo011/DeepSeek-TUI)
> にあり、**npm / crates.io / Homebrew / Scoop / GHCR には公開されていません**。
> サポートされる唯一の導入手段は `git clone` + `cargo install --path`
> です。これらのレジストリから `deepseek-tui` を入れると上流の
> [`Hmbown/DeepSeek-TUI`](https://github.com/Hmbown/DeepSeek-TUI) バイナリ
> が落ちてくるだけで、**本フォークの `/swarm` は含まれません**。

## インストール（このフォークはソースビルド必須）

`deepseek` は 2 つの Rust バイナリ — ディスパッチャ（`deepseek`）と
コンパニオン TUI（`deepseek-tui`）— で構成されています。ディスパッチャは
ランタイムに `PATH` 上の `deepseek-tui` を呼び出すので、**両方**を
フォークのソースから入れてください。片方だけだと上流の古いランタイムを
踏み続けることになります。

```bash
# Linux ビルド依存（Debian/Ubuntu/RHEL）:
#   sudo apt-get install -y build-essential pkg-config libdbus-1-dev
#   sudo dnf install -y gcc make pkgconf-pkg-config dbus-devel

# 1. このフォークを clone（上流 Hmbown/DeepSeek-TUI ではありません）。
git clone https://github.com/mooseo011/DeepSeek-TUI.git
cd DeepSeek-TUI

# 2. 両バイナリをソースからビルド＆インストール。Rust 1.88+ が必要。
cargo install --path crates/cli --locked   # `deepseek` を提供
cargo install --path crates/tui --locked   # `deepseek-tui` を提供

# 3. 確認。
deepseek --version
```

フォークから今後の更新を取り込むには:

```bash
cd /path/to/DeepSeek-TUI
git pull
cargo install --path crates/cli --locked --force
cargo install --path crates/tui --locked --force
```

> npm / crates.io / Homebrew / Scoop / Docker から上流リリース版を
> インストールしていた場合、フォークの `cargo install --path ... --force`
> は `PATH` 上の `deepseek` と `deepseek-tui` をソースビルド版で
> 上書きします。`which deepseek` で実体を確認してください。

![DeepSeek TUI スクリーンショット](assets/screenshot.png)

<details>
<summary>上流 Hmbown のリリースバイナリが欲しい場合</summary>

本フォークの `/swarm` が不要であれば、npm / crates.io / Homebrew /
Docker / GitHub Releases から上流バイナリを入れることもできます。手順は
[上流 Hmbown/DeepSeek-TUI README](https://github.com/Hmbown/DeepSeek-TUI#install)
と [docs/INSTALL.md](docs/INSTALL.md) を参照してください。上流バイナリには
本フォークの変更は**含まれません**。

</details>

---

## DeepSeek TUI とは？

DeepSeek TUI は、ターミナル内で完結するコーディングエージェントです。DeepSeek のフロンティアモデルがあなたのワークスペースに直接アクセスできるようにし、ファイルの読み取り・編集、シェルコマンドの実行、Web 検索、Git 管理、サブエージェントの統制などを、すべて高速でキーボード駆動の TUI を通じて行えます。

**DeepSeek V4 向けに構築** (`deepseek-v4-pro` / `deepseek-v4-flash`)。100 万トークンのコンテキストウィンドウとネイティブの thinking-mode（思考連鎖）ストリーミングをサポートします。

### 主な機能

- **`/swarm` オーケストレーターモード**（*本フォークで追加*）— セッション
  フラグを切り替えると、各ユーザーターンが API 送信前に「バイト単位で
  安定したオーケストレーター指示書」で包まれます。アシスタントは
  タスクを分解し、1 ターン内で `agent_open` ワーカーを並列に派遣
  （安定セッション名 + `fork_context` / `resident_file` のキャッシュ
  指向デフォルト）、`agent_eval` で結果を集約、副作用を検証して
  1 つの回答にまとめます。包み文字列は毎ターン変わらないため、
  DeepSeek の自動プレフィックスキャッシュはシステムプロンプト・
  ツール一覧・既存履歴に対して継続的にヒットします。詳細は
  [Swarm モード](#swarm-モード本フォーク限定)。
- **Auto モード** — `--model auto` / `/model auto` がターンごとにモデルと推論強度を選択
- **ネイティブ RLM** (`rlm_open`/`rlm_eval`) — 永続 REPL セッションでバッチ解析を行い、`peek`、`search`、`chunk`、`sub_query_batch` などの補助関数で低コストな `deepseek-v4-flash` 子タスクを実行
- **Thinking-mode ストリーミング** — モデルがタスクに取り組む様子をリアルタイムで観察し、思考連鎖の展開を追える
- **完全なツールスイート** — ファイル操作、シェル実行、Git、Web 検索／ブラウズ、apply-patch、サブエージェント、MCP サーバー
- **100 万トークンコンテキスト** — コンテキスト追跡、手動または設定ベースのコンパクション、プレフィックスキャッシュのテレメトリ
- **3 つのモード** — Plan（読み取り専用の探索）、Agent（承認ありのインタラクティブ）、YOLO（自動承認）
- **推論努力ティア** — `Shift + Tab` で `off → high → max` を切り替え
- **セッション保存／再開** — 長時間実行のセッションをチェックポイント化して再開可能
- **ワークスペースのロールバック** — リポジトリの `.git` には触れずに、サイド Git によるターン前後のスナップショットを `/restore` と `revert_turn` で扱える
- **永続的タスクキュー** — 再起動を超えて生き残るバックグラウンドタスク。スケジュール自動化や長時間レビューなどに
- **HTTP/SSE ランタイム API** — `deepseek serve --http` でヘッドレスエージェントワークフローを実現
- **MCP プロトコル** — Model Context Protocol サーバーに接続して拡張ツールを利用可能。詳細は [docs/MCP.md](docs/MCP.md) を参照
- **LSP 診断** — rust-analyzer、pyright、typescript-language-server、gopls、clangd により、編集ごとにエラー／警告をインライン表示
- **ユーザーメモリ** — クロスセッションの嗜好をシステムプロンプトに注入できる、オプションの永続メモファイル
- **ローカライズ済み UI** — `en`、`ja`、`zh-Hans`、`pt-BR` を自動検出
- **ライブコスト追跡** — ターンごと／セッションごとのトークン使用量とコスト見積もり、キャッシュヒット／ミスの内訳
- **スキルシステム** — GitHub から取得できる命令パック。初回起動時に `skill-creator`、`mcp-builder`、`documents`、`presentations`、`spreadsheets`、`pdf`、`feishu` などのスターターセットを同梱

---

## 仕組み

`deepseek`（ディスパッチャー CLI）→ `deepseek-tui`（コンパニオンバイナリ）→ ratatui インターフェース ↔ 非同期エンジン ↔ OpenAI 互換のストリーミングクライアント。ツール呼び出しは型付きレジストリ（シェル、ファイル操作、Git、Web、サブエージェント、MCP、RLM）を経由してルーティングされ、結果はトランスクリプトへとストリーム返送されます。エンジンはセッション状態、ターン管理、永続タスクキューを管理し、LSP サブシステムは編集後の診断を次の推論ステップ前にモデルのコンテキストへ供給します。

詳しくは [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) を参照してください。

---

## クイックスタート

```bash
git clone https://github.com/mooseo011/DeepSeek-TUI.git
cd DeepSeek-TUI
cargo install --path crates/cli --locked
cargo install --path crates/tui --locked
deepseek --version
deepseek --model auto
```

Rust 1.88+ が必要です（`rustup default stable`）。Linux ビルド依存:
`build-essential`、`pkg-config`、`libdbus-1-dev`（`apt`）または
`gcc make pkgconf-pkg-config dbus-devel`（`dnf`）。上流
`Hmbown/DeepSeek-TUI` はビルド済みバイナリを Linux x64/ARM64、
macOS x64/ARM64、Windows x64 で公開していますが、**本フォークの
`/swarm` は含まれていません**。詳細は README 冒頭のインストール案内を
参照してください。

初回起動時に [DeepSeek API キー](https://platform.deepseek.com/api_keys) の入力を求められます。キーは `~/.deepseek/config.toml` に保存されるため、OS のクレデンシャルプロンプトなしに任意のディレクトリから利用できます。

事前に設定することもできます:

```bash
deepseek auth set --provider deepseek   # ~/.deepseek/config.toml に保存

export DEEPSEEK_API_KEY="YOUR_KEY"      # 環境変数による代替方法。非対話シェルでは ~/.zshenv を使用
deepseek

deepseek doctor                         # セットアップを検証
```

> 保存済みキーをローテーション／削除するには: `deepseek auth clear --provider deepseek`。

### 中国 / ミラーフレンドリーなソースビルド

中国本土から `crates.io` への取得が遅い場合は、まず
`~/.cargo/config.toml` にミラーを設定してから、clone した本フォークに対して
`cargo install --path` を実行してください:

```toml
# ~/.cargo/config.toml
[source.crates-io]
replace-with = "tuna"

[source.tuna]
registry = "sparse+https://mirrors.tuna.tsinghua.edu.cn/crates.io-index/"
```

```bash
cargo install --path crates/cli --locked
cargo install --path crates/tui --locked
deepseek --version
```

### その他の API プロバイダー

```bash
# NVIDIA NIM
deepseek auth set --provider nvidia-nim --api-key "YOUR_NVIDIA_API_KEY"
deepseek --provider nvidia-nim

# AtlasCloud
deepseek auth set --provider atlascloud --api-key "YOUR_ATLASCLOUD_API_KEY"
deepseek --provider atlascloud

# OpenRouter
deepseek auth set --provider openrouter --api-key "YOUR_OPENROUTER_API_KEY"
deepseek --provider openrouter --model deepseek/deepseek-v4-pro

# Novita
deepseek auth set --provider novita --api-key "YOUR_NOVITA_API_KEY"
deepseek --provider novita --model deepseek/deepseek-v4-pro

# Fireworks
deepseek auth set --provider fireworks --api-key "YOUR_FIREWORKS_API_KEY"
deepseek --provider fireworks --model deepseek-v4-pro

# 汎用 OpenAI 互換エンドポイント
deepseek auth set --provider openai --api-key "YOUR_OPENAI_COMPATIBLE_API_KEY"
OPENAI_BASE_URL="https://openai-compatible.example/v4" deepseek --provider openai --model glm-5

# セルフホスト SGLang
SGLANG_BASE_URL="http://localhost:30000/v1" deepseek --provider sglang --model deepseek-v4-flash

# セルフホスト vLLM
VLLM_BASE_URL="http://localhost:8000/v1" deepseek --provider vllm --model deepseek-v4-flash

# セルフホスト Ollama
ollama pull deepseek-coder:1.3b
deepseek --provider ollama --model deepseek-coder:1.3b
```

TUI 内では `/provider` でプロバイダーピッカー、`/model` でモデルピッカーを開けます。`/provider openrouter` や `/model <id>` で直接切り替え、`/models` で API から返るライブモデル一覧を確認できます。`/model` ピッカーは、利用可能な場合は現在のプロバイダーのライブモデルカタログを使い、ない場合はプロバイダー別の既定モデルにフォールバックします。

---

## リリースノート

バージョンごとの変更点は [CHANGELOG.md](CHANGELOG.md) にまとめています。この README は、現在のインストール方法、主要ワークフロー、プロバイダー設定、ランタイムインターフェース、拡張ポイントに絞っています。

---

## 使い方

```bash
deepseek                                         # インタラクティブ TUI
deepseek "explain this function"                 # ワンショットプロンプト
deepseek exec --auto --output-format stream-json "fix this bug"  # NDJSON バックエンドストリーム
deepseek exec --resume <SESSION_ID> "follow up"  # 非対話セッションを継続
deepseek --model deepseek-v4-flash "summarize"   # モデルの上書き
deepseek --model auto "fix this bug"             # モデルと推論強度を自動選択
deepseek --yolo                                  # ツールを自動承認
deepseek auth set --provider deepseek            # API キーの保存
deepseek doctor                                  # セットアップと接続性のチェック
deepseek doctor --json                           # 機械可読の診断
deepseek setup --status                          # 読み取り専用のセットアップ状態
deepseek setup --tools --plugins                 # ツール／プラグインディレクトリの雛形作成
deepseek models                                  # ライブ API モデル一覧
deepseek sessions                                # 保存済みセッション一覧
deepseek resume --last                           # 最新セッションを再開
deepseek resume <SESSION_ID>                     # UUID 指定で特定セッションを再開
deepseek fork <SESSION_ID>                       # 任意のターンでセッションを fork
deepseek serve --http                            # HTTP/SSE API サーバー
deepseek serve --acp                             # Zed/カスタムエージェント向け ACP stdio アダプター
deepseek run pr <N>                              # PR を取得しレビュープロンプトに先行投入
deepseek mcp list                                # 設定された MCP サーバー一覧
deepseek mcp validate                            # MCP の設定／接続性を検証
deepseek mcp-server                              # ディスパッチャー MCP stdio サーバーを実行
```

> `deepseek update` は本フォークでは**使うべきではありません**。
> 上流 `Hmbown/DeepSeek-TUI` のリリースバイナリを取得し、ソースビルド
> 版（`/swarm` を含む）を上書きしてしまいます。本フォークを更新する
> 場合は、clone したディレクトリで `git pull` してから
> `cargo install --path crates/cli --locked --force` と
> `cargo install --path crates/tui --locked --force` を再実行してください。

### キーボードショートカット

| キー | 動作 |
|---|---|
| `Tab` | `/` または `@` のエントリ補完。実行中はドラフトをフォローアップとしてキューに追加。それ以外はモード切替 |
| `Shift+Tab` | 推論努力の切替: off → high → max |
| `F1` | 検索可能なヘルプオーバーレイ |
| `Esc` | 戻る／閉じる |
| `Ctrl+K` | コマンドパレット |
| `Ctrl+R` | 以前のセッションを再開 |
| `Alt+R` | プロンプト履歴を検索し、消去したドラフトを復元 |
| `Ctrl+S` | 現在のドラフトを退避（`/stash list`、`/stash pop` で復元） |
| `@path` | コンポーザーにファイル／ディレクトリのコンテキストを添付 |
| `↑`（コンポーザー先頭で） | 添付ファイル行を選択して削除 |
| `Alt+↑` | キュー済みの最後のメッセージを編集 |

ショートカット完全版: [docs/KEYBINDINGS.md](docs/KEYBINDINGS.md)。

---

## モード

| モード | 動作 |
| --- | --- |
| **Plan** 🔍 | 読み取り専用の調査 — 変更を加える前に、モデルが探索して計画を提案（`update_plan` + `checklist_write`） |
| **Agent** 🤖 | デフォルトのインタラクティブモード — 承認ゲート付きのマルチステップなツール利用。モデルは `checklist_write` で作業を概説 |
| **YOLO** ⚡ | 信頼できるワークスペースですべてのツールを自動承認。可視性のための計画とチェックリストは引き続き維持 |

---

## Swarm モード（本フォーク限定）

`/swarm` は既存 3 モードの上に**オーケストレーター主導のマルチエージェント**
層を追加します。スワーム有効中は、各ユーザーターンが API 送信時に
バイト単位で安定したオーケストレーター指示書で包まれ、アシスタントは
タスクを分解し、1 ターンで `agent_open` ワーカーを並列派遣し、
`agent_eval` で集約・副作用を検証して 1 つの回答に統合します。画面上の
「ユーザー」セルには入力原文がそのまま表示され、ラッパーはワイヤー上
にのみ存在します。

なぜキャッシュが効くか（これが設計の主眼です）:

- **ラッパーがターン間でバイト一致** — DeepSeek の自動プレフィックス
  キャッシュは system prompt・ツール一覧・既存履歴にヒットし続け
  ます（ユニットテストで保証済み）。
- **安定したワーカーセッション名** — 指示書は `worker_search`、
  `worker_patch`、`worker_verify` のような固定名の使い回しを要求し、
  各ワーカーの定型プレフィックスを後続ターンでもキャッシュにとどめます。
- **デフォルトで `fork_context: false`** — 狭いコンテキストで新規に
  立ち上げるワーカーは prefill が小さく安価。親履歴が本当に必要な
  時だけ `fork_context: true` を許可します。
- **`resident_file` リース** — 単一ファイルを繰り返し触る作業では
  そのファイルをワーカーの system prefix に常駐させ、`send_input` /
  `agent_eval` をまたいでキャッシュ温度を維持（既存
  `RESIDENT_LEASES` ルールに従い同時 1 ファイル）。
- **1 ターンでの並列 `agent_open`** — ディスパッチャーはもともと
  並列実行可能。指示書は連鎖呼び出しではなくバッチ派遣を強制します。
- **再引用ではなく `handle_read`** — ワーカーの大きな出力は
  `handle_read` で範囲指定して取り、親コンテキストに丸ごとコピーは
  しません。
- **履歴は追記のみ** — 既存メッセージの言い換え・並び替えは厳禁。
  並びを崩すと以後のキャッシュが全滅します。

サブコマンド:

```text
/swarm                       # 有効/無効を切り替え
/swarm on                    # 有効化（即時派遣はしない）
/swarm off                   # 無効化
/swarm status                # 現在の状態とセッションブリーフを表示
/swarm brief <text>          # ラッパーに埋め込むセッションブリーフを固定
/swarm brief clear           # ブリーフをクリア
/swarm <task>                # 必要に応じて有効化して <task> を即時派遣
```

セッションブリーフは、フォーカスする領域・除外したい場所・リポジトリ
固有の規約など、毎ターン再記述したくない定常情報を置く場所です。
別名: `/fengqun`、`/蜂群`。

スワーム有効時のターンの流れ:

1. 普段どおりプロンプトを入力。
2. オーケストレーター（メインアシスタント）がタスクを分解し、
   1 ターン内で並列に `agent_open` ワーカーを派遣。
3. `agent_eval` で集約、副作用を再検証（ファイル編集・シェルコマンド・
   テスト結果はワーカーの主張のまま信用せず、事実として扱う前に
   再確認）、統合された答えを返す。
4. ユーザーが次のプロンプトを送る。ワーカーは設計上ターンをまたいで
   開いたまま残り、作業完了・長時間アイドル・`resident_file` リース
   解放が必要な時だけ `agent_close` します。

本リビジョンでスワームのフラグは保存セッションに永続化されません。
`/load` 後は手動で `/swarm on` してください。

---

## 設定

ユーザー設定: `~/.deepseek/config.toml`。プロジェクトオーバーレイ: `<workspace>/.deepseek/config.toml`（拒否される項目: `api_key`、`base_url`、`provider`、`mcp_config_path`）。すべてのオプションは [config.example.toml](config.example.toml) にあります。

主な環境変数:

| 変数 | 用途 |
|---|---|
| `DEEPSEEK_API_KEY` | API キー |
| `DEEPSEEK_BASE_URL` | API ベース URL |
| `DEEPSEEK_HTTP_HEADERS` | 任意のモデルリクエストヘッダー |
| `DEEPSEEK_MODEL` | デフォルトモデル |
| `DEEPSEEK_STREAM_IDLE_TIMEOUT_SECS` | ストリームのアイドルタイムアウト秒数 |
| `DEEPSEEK_PROVIDER` | `deepseek`（デフォルト）、`nvidia-nim`、`openai`、`atlascloud`、`openrouter`、`novita`、`fireworks`、`sglang`、`vllm`、`ollama` |
| `DEEPSEEK_PROFILE` | 設定プロファイル名 |
| `DEEPSEEK_MEMORY` | `on` に設定するとユーザーメモリを有効化 |
| `DEEPSEEK_ALLOW_INSECURE_HTTP=1` | 信頼できるネットワークで非ローカル `http://` API ベース URL を許可 |
| `NVIDIA_API_KEY` / `OPENAI_API_KEY` / `ATLASCLOUD_API_KEY` / `OPENROUTER_API_KEY` / `NOVITA_API_KEY` / `FIREWORKS_API_KEY` / `SGLANG_API_KEY` / `VLLM_API_KEY` / `OLLAMA_API_KEY` | プロバイダー認証 |
| `OPENAI_BASE_URL` / `OPENAI_MODEL` | 汎用 OpenAI 互換エンドポイントとモデル ID |
| `ATLASCLOUD_BASE_URL` / `ATLASCLOUD_MODEL` | AtlasCloud エンドポイントとモデル上書き |
| `OPENROUTER_BASE_URL` | OpenRouter エンドポイント上書き |
| `NOVITA_BASE_URL` | Novita エンドポイント上書き |
| `FIREWORKS_BASE_URL` | Fireworks エンドポイント上書き |
| `SGLANG_BASE_URL` | セルフホスト SGLang のエンドポイント |
| `SGLANG_MODEL` | セルフホスト SGLang のモデル ID |
| `VLLM_BASE_URL` | セルフホスト vLLM のエンドポイント |
| `VLLM_MODEL` | セルフホスト vLLM のモデル ID |
| `OLLAMA_BASE_URL` | セルフホスト Ollama のエンドポイント |
| `OLLAMA_MODEL` | セルフホスト Ollama のモデルタグ |
| `NO_ANIMATIONS=1` | 起動時にアクセシビリティモードを強制 |
| `SSL_CERT_FILE` | 企業プロキシ向けのカスタム CA バンドル |

UI のロケールはモデルの言語とは別です。`settings.toml` で `locale` を設定するか、`/config locale zh-Hans` を使うか、`LC_ALL`/`LANG` に依存させてください。詳しくは [docs/CONFIGURATION.md](docs/CONFIGURATION.md) と [docs/MCP.md](docs/MCP.md) を参照してください。

---

## モデルと料金

| モデル | コンテキスト | 入力（キャッシュヒット） | 入力（キャッシュミス） | 出力 |
|---|---|---|---|---|
| `deepseek-v4-pro` | 1M | $0.003625 / 1M* | $0.435 / 1M* | $0.87 / 1M* |
| `deepseek-v4-flash` | 1M | $0.0028 / 1M | $0.14 / 1M | $0.28 / 1M |

レガシーエイリアス `deepseek-chat` / `deepseek-reasoner` は `deepseek-v4-flash` にマップされます。NVIDIA NIM のバリアントはあなたの NVIDIA アカウント条件に従います。

*DeepSeek Pro の料金は現在、期間限定で 75% の割引が適用されており、2026 年 5 月 31 日 15:59 UTC まで有効です。それ以降、TUI のコスト見積もりは Pro の通常料金に戻ります。*

---

## 自分のスキルを公開する

DeepSeek TUI はワークスペースのディレクトリ（`.agents/skills` → `skills` → `.opencode/skills` → `.claude/skills`）とグローバルな `~/.deepseek/skills` からスキルを発見します。各スキルは `SKILL.md` ファイルを持つディレクトリです:

```text
~/.deepseek/skills/my-skill/
└── SKILL.md
```

必要なフロントマター:

```markdown
---
name: my-skill
description: DeepSeek にカスタムワークフローを実行させたいときに利用する。
---

# My Skill
ここにエージェント向けの指示を記述します。
```

コマンド: `/skills`（一覧）、`/skill <name>`（有効化）、`/skill new`（雛形）、`/skill install github:<owner>/<repo>`（コミュニティ）、`/skill update` / `uninstall` / `trust`。GitHub からのコミュニティインストールにバックエンドサービスは不要です。インストール済みのスキルはモデルに見えるセッションコンテキストに表示され、タスクが説明文にマッチした場合はエージェントが `load_skill` ツールを通じて関連スキルを自動選択できます。

---

## ドキュメント

| ドキュメント | トピック |
|---|---|
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | コードベース内部 |
| [CONFIGURATION.md](docs/CONFIGURATION.md) | 設定の完全リファレンス |
| [MODES.md](docs/MODES.md) | Plan / Agent / YOLO モード |
| [MCP.md](docs/MCP.md) | Model Context Protocol 統合 |
| [RUNTIME_API.md](docs/RUNTIME_API.md) | HTTP/SSE API サーバー |
| [INSTALL.md](docs/INSTALL.md) | プラットフォーム別インストールガイド |
| [DOCKER.md](docs/DOCKER.md) | GHCR イメージ、ボリューム、Docker 利用方法 |
| [CNB_MIRROR.md](docs/CNB_MIRROR.md) | CNB ミラーと中国向けインストールメモ |
| [TENCENT_CLOUD_REMOTE_FIRST.md](docs/TENCENT_CLOUD_REMOTE_FIRST.md) | Tencent/CNB/Lighthouse/Feishu のリモート優先パス |
| [TENCENT_LIGHTHOUSE_HK.md](docs/TENCENT_LIGHTHOUSE_HK.md) | Tencent Lighthouse 香港インスタンス設定 |
| [MEMORY.md](docs/MEMORY.md) | ユーザーメモリ機能ガイド |
| [SUBAGENTS.md](docs/SUBAGENTS.md) | サブエージェントの役割分類とライフサイクル |
| [KEYBINDINGS.md](docs/KEYBINDINGS.md) | ショートカット完全カタログ |
| [RELEASE_RUNBOOK.md](docs/RELEASE_RUNBOOK.md) | リリースプロセス |
| [OPERATIONS_RUNBOOK.md](docs/OPERATIONS_RUNBOOK.md) | 運用とリカバリ |

完全な変更履歴: [CHANGELOG.md](CHANGELOG.md)。

---

## 謝辞

このプロジェクトは、増え続けるコントリビューターのコミュニティから助けを得て出荷されています:

- **[merchloubna70-dot](https://github.com/merchloubna70-dot)** — 機能、修正、VS Code 拡張のスキャフォールドにまたがる 28 件の PR (#645–#681)
- **[WyxBUPT-22](https://github.com/WyxBUPT-22)** — 表、太字／斜体、水平線の Markdown レンダリング (#579)
- **[loongmiaow-pixel](https://github.com/loongmiaow-pixel)** — Windows と中国向けインストールドキュメント (#578)
- **[20bytes](https://github.com/20bytes)** — ユーザーメモリのドキュメントとヘルプの磨き込み (#569)
- **[staryxchen](https://github.com/staryxchen)** — glibc 互換性のプリフライト (#556)
- **[Vishnu1837](https://github.com/Vishnu1837)** — glibc 互換性の改善 (#565)
- **[shentoumengxin](https://github.com/shentoumengxin)** — シェル `cwd` の境界バリデーション (#524)
- **[toi500](https://github.com/toi500)** — Windows 貼り付け修正の報告
- **[xsstomy](https://github.com/xsstomy)** — ターミナル起動時の再描画報告
- **[melody0709](https://github.com/melody0709)** — スラッシュ接頭辞の Enter アクティベーション報告
- **[lloydzhou](https://github.com/lloydzhou)** と **[jeoor](https://github.com/jeoor)** — コンパクションコストの報告
- **[Agent-Skill-007](https://github.com/Agent-Skill-007)** — README の明瞭化対応 (#685)
- **[woyxiang](https://github.com/woyxiang)** — Windows Scoop インストールドキュメント (#696)
- **[wangfeng](mailto:wangfengcsu@qq.com)** — 料金／割引情報の更新 (#692)
- **[zichen0116](https://github.com/zichen0116)** — CODE_OF_CONDUCT.md (#686)
- **Hafeez Pizofreude** — `fetch_url` の SSRF 保護と Star History チャート
- **Unic (YuniqueUnic)** — スキーマ駆動の設定 UI（TUI + Web）
- **Jason** — SSRF セキュリティの強化

---

## コントリビューション

本フォークは
[github.com/mooseo011/DeepSeek-TUI](https://github.com/mooseo011/DeepSeek-TUI)
にあります。フォーク固有の issue / PR はこちらへ。上流リポジトリの
[CONTRIBUTING.md](CONTRIBUTING.md) と
[Open Issues](https://github.com/Hmbown/DeepSeek-TUI/issues) の方針は
そのまま適用されます。

> [!Note]
> *DeepSeek Inc. とは関係ありません。*

## ライセンス

[MIT](LICENSE)

## Star History

[![Star History Chart](https://api.star-history.com/chart?repos=Hmbown/DeepSeek-TUI&type=date&legend=top-left)](https://www.star-history.com/?repos=Hmbown%2FDeepSeek-TUI&type=date&logscale=&legend=top-left)
