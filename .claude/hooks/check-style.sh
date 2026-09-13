#!/bin/sh
# AGENTS.md の文体のうち、機械的に判定できるものを検査する。
#
# 使い方は 3 通りある。
#   1. 引数にファイルを渡す
#        .claude/hooks/check-style.sh AGENTS.md
#        git ls-files '*.md' '*.rs' | xargs .claude/hooks/check-style.sh
#   2. PostToolUse hook (Edit/Write) として。編集されたファイルを調べる。
#   3. PreToolUse hook (GitHub へのコメント投稿) として。本文を調べ、
#      該当があれば投稿を止める。
#
# 検出したい文字を [（）] のように角カッコで並べるとバイト単位で分解され、
# 「。」「、」「」」を含む普通の日本語まで一致する。交替 (|) で書く必要がある。
#
# 主語が誰か、比喩かどうかは判定できない。「ブレーキが効く」のような
# 正しい用法も止まる。その場合は作業者に相談すること。
set -u

PATTERNS='（|）|！|？|：|；|\*\*|効く|効か|効き|効け|効こ|効い|まさに|唯一の|そのものです|生きている'

# AGENTS.md は規則そのものを引用するため、CHANGELOG.md は release-plz が
# 生成するため、どちらも検査しない。
is_exempt() {
    case "${1##*/}" in
    AGENTS.md | CHANGELOG.md) return 0 ;;
    *) return 1 ;;
    esac
}

find_hits() {
    is_exempt "$1" && return 1
    grep -nE "$PATTERNS" "$1" 2>/dev/null
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

    hits=$(printf '%s\n' "$body" | grep -nE "$PATTERNS") || exit 0
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
