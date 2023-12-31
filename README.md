# Yomiage-mon
ボイスチャネルでミュートの人の発言をずんだもんが読み上げるDiscord Botです。

## 導入の仕方

1. 事前にDocker､Docker Compose v2がインストールされている必要があります｡

### 利用者向け(`compose.yml`のみ利用する方式)
1. トップページ右下部の[Releases](releases)をクリックし､｢Assets｣のドロップダウンメニューにある[compose.yml](deployment/compose.yml)をダウンロードしてください｡

2. 以下のファイルを次の形式で作成してください｡
    ### `.env` (環境変数用)
    ```
    PREFIX=-
    ```
    + `-`の部分は､すきな接頭辞を設定できます｡`!`を指定した場合､コマンドは`!zunda`の用に利用できます｡
    
    ### `.secret`(機密情報用)
    ```
    TOKEN=BotEXAMPLETOKEN
    ```
   + Discord Developer Portalで作成したBotのトークンを`TOKEN=`につづいてコピペなどで作成してください｡
   + **`.secret`は絶対に外部に公開しないでください｡** これが外部に公開された場合､第三者によって悪意ある操作が行われ､犯罪に巻き込まれる可能性があります｡

3. 以下のコマンドを実行してください ($は入力しないでください)
    ```bash
    $ docker compose build
    $ docker compose up -d
    ```

### 開発者向け(クローンする方式)
1. Repositoryをクローンしてください｡
    GitHub CLIの場合:
    ```bash
    $ gh repo clone https://github.com/approvers/yomiage-mon
    ```

2. 環境変数を`.secret`, `.env`に記述してください｡以下を必ず設定してください｡ なお､書き方は[.secret.example](.secret.example)､[.env.example](.env.example)を参考にしてください｡

    ### .secret
    
    | 環境変数名 |    説明    |
    |:-----:|:--------:|
    | TOKEN | Botのトークン |
    
    ### .env
    
    | 環境変数名  |      説明      |
    |:------:|:------------:|
    | PREFIX | Botのコマンドの接頭辞 |
    
    (注:`TOKEN`を記載した`.secret`を外部に公開しないでください! Gitでコミットする前にかならず`.env`を`.gitignore`に追加するなど予防策をとってください｡)

3. Dockerのビルドを行ってください｡
    ```bash
    $ docker compose build
    ``````

4. Dockerの起動を行ってください｡
    ```bash
    $ docker compose up -d
    ```

## 使い方

Discord Developer PortalにてBotの設定は行われているものとします｡
Botには以下の権限が必要です｡

- メッセージを送信(`Send Messages`)
- メッセージの閲覧･チャネルの閲覧(`Read Messages / View Channels`)
- VCに参加(`Connect`)
- VCで発言(`Speak`)

Botをサーバーに追加後は､以下のコマンドが実行できます｡
設定された`PREFIX`をつけて送信してください｡(例:`PREFIX`が`!`の場合､ `!zunda`など)

|      コマンド       |                     説明                      |
|:---------------:|:-------------------------------------------:|
|     `zunda`     |             ずんだもんがあなたに挨拶してくれます｡             |
|     `help`      |                 ヘルプを表示します｡                  |
|      `vc`       | BotをVCに参加させます｡ただしコマンドの送り主がVCに接続している必要があります｡ |
|     `leave`     |               BotをVCから退出させます｡               |
|     `list`      |        Botが読み上げる対象のテキストチャンネルを表示します｡         |
| `listen`, `add` |    コマンドを送信したテキストチャネルをBotが読み上げる対象に追加します｡     |
|    `remove`     |    コマンドを送信したテキストチャネルをBotが読み上げる対象から削除します｡    |

またVCに接続しているBot以外の利用者がいなくなった場合､BotはVCから退出します｡

## 問い合わせ･バグ報告

問い合わせやバグ報告は[ahoxa/ライガー](https://.com/ahoxa1rx)までお願いします｡
[限界開発鯖](https://approvers.dev)のメンバーはDiscordでの問い合わせも可能です｡

機能要望は[GitHub Issues](https://github.com/approvers/yomiage-mon/issues)にて受け付けています｡
(現在Discussionsを整備中です｡)

## ライセンス
MIT Licenseに準拠します｡
詳しくは[LICENSE](LICENSE)をご覧ください｡
