# Codex / OpenAI Agent Guidance

仕事の入口と権限は [AGENTS.md](../../AGENTS.md#session-start)。このディレクトリはモデル補助を確認・調整するときに読む。通常の質問や既知の作業で毎回全ファイルを読み直す必要はない。

- [Shared Contract](shared.md): 共通契約との接続と指示競合の扱い。
- [モデル差分メモ](model-notes.md): 明示された実モデルに合う補助だけを使う。未知モデルへ別世代の性質を転記しない。
- [マージ検証とhelper](merge-evidence.md): SHA手作業、実装後の状態保存、GitHub保護とdocs経路。

## Model updates

モデル選択は [.codex/README.md](../../.codex/README.md) の config / 一時指定が所有する。モデル更新時は対象・確認日・公式根拠を確認し、観測された不足にだけ補助を追加する。ある model の助言を別の model や Claude の実測済み特性として扱わない。

個人の応答スタイルは ignored `AGENTS.override.md`。override は tracked AGENTS をロードしてから適用し、共通の権限や gate を上書きしない。

個人拡張・会話全文・実測ログはlocal-only。文字数の削減と実モデルのtoken・判断結果を区別する。

出典: [OpenAI AGENTS.md](https://learn.chatgpt.com/docs/agent-configuration/agents-md)、[Using GPT-6.2](https://developers.openai.com/api/docs/guides/latest-model)、[Prompting GPT-6.2 in Codex](https://learn.chatgpt.com/guides/prompting-gpt-6-2-astra)。
