#!/bin/sh
# 全角記号が含まれていないか調べる。
# AGENTS.md の「カッコなどの記号は半角を使う」に対応する。
#
# 引数にファイルを渡すとそれらを調べる。
#   .claude/hooks/check-fullwidth.sh AGENTS.md
#   git ls-files '*.md' '*.rs' | xargs .claude/hooks/check-fullwidth.sh
#
# 引数がない場合は PostToolUse hook として動き、標準入力の JSON から
# 編集されたファイルを取り出す。対象は .md と .rs のみ。
# 見つかった場合は終了コード 2 で内容を stderr に出し、Claude に修正を促す。
#
# 検出したい文字を [（）] のように角カッコで並べるとバイト単位で分解され、
# 「。」「、」「」」を含む普通の日本語まで一致する。交替 (|) で書く必要がある。
set -u

PATTERN='（|）|！|？|：|；'

check() {
    hits=$(grep -nE "$PATTERN" "$1") || return 0
    echo "$1 に全角記号があります。半角に直してください。" >&2
    echo "$hits" >&2
    return 1
}

if [ $# -gt 0 ]; then
    status=0
    for f in "$@"; do
        [ -f "$f" ] || continue
        check "$f" || status=2
    done
    exit $status
fi

file=$(jq -r '.tool_input.file_path // .tool_response.filePath // empty')
[ -n "$file" ] || exit 0

case "$file" in
*.md | *.rs) ;;
*) exit 0 ;;
esac

[ -f "$file" ] || exit 0
check "$file" || exit 2
