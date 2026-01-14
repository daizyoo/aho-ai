# 将棋-チェス ハイブリッドゲーム Web 版

オンライン対戦可能な将棋とチェスのハイブリッドゲームです。

## 機能

- 🎮 **Human vs Human** オンライン対戦
- 🏠 **ルーム システム** ルーム作成・参加
- 🎲 **複数の盤タイプ**
  - 将棋
  - チェス
  - ハイブリッド（将棋 vs チェス）
  - カスタム盤（自由に駒を配置）
- 🎯 **持ち駒システム** 将棋ルールで持ち駒使用可能
- 💾 **カスタム盤保存** 作成した盤を保存して再利用

## 技術スタック

### フロントエンド

- Next.js 14 (App Router)
- TypeScript
- Socket.io Client
- Vanilla CSS

### バックエンド

- Node.js + Express
- Socket.io
- Supabase (PostgreSQL)

## セットアップ

### 前提条件

- Node.js 18+
- npm
- Supabase アカウント（データベース用）

### 1. リポジトリをクローン

```bash
cd web
```

### 2. フロントエンドのセットアップ

```bash
# 依存関係をインストール
npm install

# 環境変数を設定
cp .env.example .env.local
# .env.localを編集してバックエンドのURLを設定
```

`.env.local`:

```env
NEXT_PUBLIC_API_URL=http://localhost:3001
NEXT_PUBLIC_SOCKET_URL=ws://localhost:3001
```

### 3. バックエンドのセットアップ

```bash
cd server

# 依存関係をインストール
npm install

# 環境変数を設定
cp .env.example .env
# .envを編集してSupabase認証情報を設定
```

`.env`:

```env
PORT=3001
CORS_ORIGIN=http://localhost:3000
SUPABASE_URL=your-supabase-url
SUPABASE_KEY=your-supabase-anon-key
```

### 4. Supabase データベース設定

1. [Supabase](https://supabase.com/)でプロジェクトを作成
2. SQL Editor を開く
3. `server/src/db/migrations/001_initial.sql`の内容を実行
4. `Settings` → `API`から URL と Anon Key を取得し、`.env`に設定

### 5. 開発サーバーを起動

**ターミナル 1（バックエンド）:**

```bash
cd server
npm run dev
```

**ターミナル 2（フロントエンド）:**

```bash
cd ..  # webディレクトリに戻る
npm run dev
```

アプリケーションが起動します：

- フロントエンド: http://localhost:3000
- バックエンド: http://localhost:3001

## 開発

### ディレクトリ構造

```
web/
├── app/              # Next.js App Router
├── components/       # Reactコンポーネント
├── lib/              # ユーティリティと型定義
├── hooks/            # カスタムフック
├── styles/           # CSSモジュール
├── public/           # 静的ファイル
└── server/           # バックエンドサーバー
    ├── src/
    │   ├── socket/   # WebSocketハンドラー
    │   ├── api/      # REST API
    │   └── db/       # データベース
```

### 素材の追加

駒の画像を`public/assets/pieces/`に配置：

```
public/assets/pieces/
├── shogi/    # 将棋の駒（SVG推奨）
└── chess/    # チェスの駒（SVG推奨）
```

フリー素材のダウンロード先：

- [Wikimedia Commons](https://commons.wikimedia.org/)
- [lichess.org](https://github.com/lichess-org/lila/tree/master/public/piece)

## デプロイ

### Vercel（フロントエンド）

```bash
npm run build
vercel --prod
```

### Railway/Render（バックエンド）

```bash
cd server
npm run build

# Railway CLIの場合
railway up

# または Renderのダッシュボードからデプロイ
```

環境変数を本番環境で設定することを忘れずに！

## ライセンス

MIT

## 貢献

プルリクエスト歓迎！
