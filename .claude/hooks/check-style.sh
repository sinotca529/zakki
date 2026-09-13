#!/bin/sh
# AGENTS.md の文体のうち、機械的に判定できるものを検査する。
#
# 検査は 2 段になっている。
#   - grep: 全角記号、太字、見出しの形。node がなくても動く。.rs も対象。
#   - textlint: 語彙と日本語の一般的な規則。.md と GitHub のコメント本文だけ。
#     .claude/hooks/node_modules がなければ飛ばす。導入は次の通り。
#       cd .claude/hooks && npm ci
#
# 呼び出し方は 3 通りある。
#   1. 引数にファイルを渡す
#        .claude/hooks/check-style.sh AGENTS.md
#        git ls-files '*.md' '*.rs' | xargs .claude/hooks/check-style.sh
#   2. PostToolUse hook (Edit/Write) として。編集されたファイルを調べる。
#   3. PreToolUse hook (GitHub へのコメント投稿) として。本文を調べ、
#      該当があれば投稿を止める。
#
# 検出したい文字を [（）] のように角カッコで並べるとバイト単位で分解され、
# 「。」「、」「」」を含む普通の日本語まで一致する。交替 (|) で書く必要がある。
set -u

HOOK_DIR=$(dirname "$0")

# 全角記号と強調の太字。textlint は Markdown の ** を本文として読まないため、
# ここで見る。
PATTERNS='（|）|！|？|：|；|\*\*'

# 見出しは名詞句にする。文の述語で終わるもの、接続詞で始まるものを検出する。
# 目次に並べて意味が通らない見出しを弾くのが狙い。
HEAD_NG='^#{1,6} .*(ます|ました|ません|でした|です|ください|しない|になる|がある|が違う)$|^#{1,6} *(ただし|しかし|そして|それでも|なお|また|つまり|ちなみに|さらに)'

# 検査しないファイル。AGENTS.md は規則そのものを引用し、CHANGELOG.md は
# release-plz が生成し、LICENSE.md は配布元の文面をそのまま置いている。
is_exempt() {
    case "${1##*/}" in
    AGENTS.md | CHANGELOG.md | LICENSE.md | LICENSE) return 0 ;;
    *) return 1 ;;
    esac
}

has_textlint() {
    [ -d "$HOOK_DIR/node_modules/textlint" ]
}

# ファイルを検査し、指摘があれば標準出力に出して 0 を返す。
find_hits() {
    is_exempt "$1" && return 1
    hits=$(grep -nE "$PATTERNS|$HEAD_NG" "$1" 2>/dev/null)

    case "$1" in
    *.md)
        if has_textlint; then
            tl=$("$HOOK_DIR/node_modules/.bin/textlint" -c "$HOOK_DIR/.textlintrc.json" \
                -f compact "$1" 2>/dev/null)
            hits=$(printf '%s\n%s' "$hits" "$tl")
        fi
        ;;
    esac

    hits=$(printf '%s' "$hits" | grep -v '^$')
    [ -n "$hits" ] || return 1
    printf '%s\n' "$hits"
}

# --- 1. 引数モード ---
if [ $# -gt 0 ]; then
    status=0
    for f in "$@"; do
        [ -f "$f" ] || continue
        hits=$(find_hits "$f") || continue
        echo "$f" >&2
        echo "$hits" >&2
        status=2
    done
    exit $status
fi

input=$(cat)
event=$(printf '%s' "$input" | jq -r '.hook_event_name // empty')

# --- 3. PreToolUse: GitHub へ投稿する本文 ---
if [ "$event" = "PreToolUse" ]; then
    body=$(printf '%s' "$input" | jq -r '.tool_input.body // .tool_input.text // empty')
    [ -n "$body" ] || exit 0

    hits=$(printf '%s\n' "$body" | grep -nE "$PATTERNS|$HEAD_NG")
    if has_textlint; then
        tl=$(printf '%s\n' "$body" | "$HOOK_DIR/node_modules/.bin/textlint" \
            -c "$HOOK_DIR/.textlintrc.json" -f compact \
            --stdin --stdin-filename comment.md 2>/dev/null)
        hits=$(printf '%s\n%s' "$hits" "$tl")
    fi
    hits=$(printf '%s' "$hits" | grep -v '^$') || exit 0

    reason="AGENTS.md の文体に反する箇所があります。直してから投稿してください。
$hits"
    jq -n --arg r "$reason" '{
      hookSpecificOutput: {
        hookEventName: "PreToolUse",
        permissionDecision: "deny",
        permissionDecisionReason: $r
      }
    }'
    exit 2
fi

# --- 2. PostToolUse: 編集されたファイル ---
file=$(printf '%s' "$input" | jq -r '.tool_input.file_path // .tool_response.filePath // empty')
[ -n "$file" ] || exit 0

case "$file" in
*.md | *.rs) ;;
*) exit 0 ;;
esac

[ -f "$file" ] || exit 0
hits=$(find_hits "$file") || exit 0
echo "$file に AGENTS.md の文体に反する箇所があります。" >&2
echo "$hits" >&2
exit 2
