#!/usr/bin/env bash
# 定时 dev 分支同步：提交本地改动 -> 快进同步 -> 推送 origin/dev
# （不再创建 dev -> main 的 PR —— main 只接受上游同步，见 AGENTS.md 分支策略）
#
# 用法:
#   scripts/scheduled-dev-sync.sh              # dry-run，只检查并打印将要执行的操作
#   APPLY=1 scripts/scheduled-dev-sync.sh      # 实际执行（提交/推送）
#
# 可覆盖环境变量:
#   REPO           仓库路径，默认脚本所在目录的上一级
#   TARGET_BRANCH  目标分支，默认 dev
#   BASE_BRANCH    基线分支，默认 main（仅供报告；不再向它发 PR）
#   COMMIT_MESSAGE 提交信息，默认 "chore($TARGET_BRANCH): scheduled sync <时间戳>"
#   RUSTCODE_PR_TOKEN  可选；用于通过远程仓库的 v5 兼容 API 创建 PR，
#                      缺失或无法从 origin 推断托管地址时降级为输出手工创建提示
#   EXCLUDE_SPECS  额外排除路径，默认已排除 .codebuddy/（IDE 本地数据，不属于仓库内容）
#   COMMIT_AUTHOR  提交身份 "Name <email>"；未设置时取仓库最近一次提交的作者。
#                  身份通过 git -c 临时传入，不写入 git config。
#
# 安全边界（硬性，脚本不提供绕过开关）:
#   - 只操作 TARGET_BRANCH，绝不修改/重置/强推 BASE_BRANCH
#   - 禁止 force push、分支/tag 删除、git clean/reset/stash/restore/checkout -- .
#   - 禁止 --no-verify，钩子失败即停止
#   - 分支分歧、未完成 merge/rebase/cherry-pick、推送被拒时 fail-closed 停止并报告

set -euo pipefail

REPO="${REPO:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
TARGET_BRANCH="${TARGET_BRANCH:-dev}"
BASE_BRANCH="${BASE_BRANCH:-main}"
APPLY="${APPLY:-0}"
DRY=$([ "$APPLY" = "1" ] && echo 0 || echo 1)

log()  { printf '[dev-sync] %s\n' "$*"; }
fail() { printf '[dev-sync][STOP] %s\n' "$*" >&2; exit 1; }

# dry-run 时只打印，不执行
run() {
  if [ "$DRY" = "1" ]; then
    printf '[dev-sync][dry-run] %s\n' "$*"
  else
    printf '[dev-sync][exec] %s\n' "$*"
    "$@"
  fi
}

cd "$REPO" || fail "无法进入仓库目录: $REPO"
git rev-parse --is-inside-work-tree >/dev/null 2>&1 || fail "不是 git 仓库: $REPO"
[ "$DRY" = "1" ] && log "DRY-RUN: 仅检查与打印，需 APPLY=1 才会提交/推送"

# ---------- 1. 状态检查 ----------
CURRENT_BRANCH="$(git rev-parse --abbrev-ref HEAD)"
DIRTY_COUNT="$(git status --porcelain | wc -l | tr -d ' ')"
log "当前分支: $CURRENT_BRANCH (目标: $TARGET_BRANCH, 基线: $BASE_BRANCH), 工作区改动条目: $DIRTY_COUNT"

for marker in MERGE_HEAD CHERRY_PICK_HEAD REVERT_HEAD rebase-merge rebase-apply; do
  [ -e ".git/$marker" ] && fail "存在未完成的 git 操作 ($marker)，请先人工处理后再运行"
done

run git fetch --all --prune

# ---------- 2. 分支基线 ----------
local_has_target="$(git rev-parse --verify --quiet "refs/heads/$TARGET_BRANCH" || true)"
remote_has_target="$(git rev-parse --verify --quiet "refs/remotes/origin/$TARGET_BRANCH" || true)"

if [ -z "$local_has_target" ] && [ -z "$remote_has_target" ]; then
  log "分支 $TARGET_BRANCH 不存在，将基于 $BASE_BRANCH 创建"
  run git branch "$TARGET_BRANCH" "$BASE_BRANCH"
elif [ -z "$local_has_target" ] && [ -n "$remote_has_target" ]; then
  log "本地无 $TARGET_BRANCH，将跟踪 origin/$TARGET_BRANCH"
  run git branch --track "$TARGET_BRANCH" "origin/$TARGET_BRANCH"
else
  log "分支 $TARGET_BRANCH 已存在"
fi

# ---------- 3. 切换分支 ----------
if [ "$CURRENT_BRANCH" != "$TARGET_BRANCH" ]; then
  if ! run git checkout "$TARGET_BRANCH"; then
    fail "切换 $TARGET_BRANCH 失败（通常是工作区改动会冲突），脚本未做任何 stash/reset，请人工处理"
  fi
fi

# ---------- 4. 提交本地改动 ----------
if [ "$DIRTY_COUNT" != "0" ]; then
  msg="${COMMIT_MESSAGE:-"chore($TARGET_BRANCH): scheduled sync $(date '+%Y-%m-%d %H:%M')"}"
  author="${COMMIT_AUTHOR:-"$(git log -1 --format='%an <%ae>' 2>/dev/null || true)"}"
  [ -z "$author" ] && fail "无法确定提交身份：本机 git user.name/email 未配置，请设置 COMMIT_AUTHOR=\"Name <email>\""
  log "提交身份(临时 -c，不写入 git config): $author"
  run git add -A -- . "${EXCLUDE_SPECS:-":!.codebuddy/"}"
  GIT_AUTHOR_NAME="${author%% <*}" GIT_AUTHOR_EMAIL="$(printf '%s' "$author" | sed -E 's/.*<(.*)>/\1/')" \
    GIT_COMMITTER_NAME="${author%% <*}" GIT_COMMITTER_EMAIL="$(printf '%s' "$author" | sed -E 's/.*<(.*)>/\1/')" \
    run git commit -m "$msg"
else
  log "工作区无改动，跳过提交"
fi

# ---------- 5. 与远程快进同步 ----------
remote_has_target="$(git rev-parse --verify --quiet "refs/remotes/origin/$TARGET_BRANCH" || true)"
if [ -n "$remote_has_target" ]; then
  if ! run git pull --ff-only origin "$TARGET_BRANCH"; then
    fail "origin/$TARGET_BRANCH 与本地 $TARGET_BRANCH 分歧，禁止自动 merge/rebase，请人工处理"
  fi
else
  log "origin/$TARGET_BRANCH 不存在，跳过快进同步（首次推送将创建远程分支）"
fi

# ---------- 6. 推送 ----------
remote_has_target="$(git rev-parse --verify --quiet "refs/remotes/origin/$TARGET_BRANCH" || true)"
if [ -z "$remote_has_target" ]; then
  ahead_remote="1"   # 远程分支不存在，属于首次推送
else
  ahead_remote="$(git rev-list --count "origin/$TARGET_BRANCH..$TARGET_BRANCH" 2>/dev/null || echo 0)"
fi
if [ "$ahead_remote" != "0" ]; then
  if ! run git push -u origin "$TARGET_BRANCH"; then
    fail "推送 origin/$TARGET_BRANCH 被拒绝，未做任何 force push，请人工处理（常见原因：鉴权失败、分支保护、非快进）"
  fi
else
  log "$TARGET_BRANCH 与 origin/$TARGET_BRANCH 一致，无需推送"
fi

# ---------- 7. PR（默认关闭）: dev -> BASE_BRANCH ----------
ahead_base="$(git rev-list --count "$BASE_BRANCH..$TARGET_BRANCH" 2>/dev/null || echo 0)"
behind_base="$(git rev-list --count "$TARGET_BRANCH..$BASE_BRANCH" 2>/dev/null || echo 0)"

remote_url="$(git remote get-url origin 2>/dev/null || echo '')"
# Derive host/slug strictly from the configured origin remote; this build ships
# no default vendor host. When origin can't be parsed, degrade to generic guidance.
# slug = last two path segments (owner/repo); strip a trailing .git separately
# (POSIX ERE has no non-greedy quantifier, so an inline (\.git)? is never taken).
slug="$(printf '%s' "$remote_url" | sed -E 's#^.*[:/]([^/]+)/([^/]+)$#\1/\2#')"
slug="${slug%.git}"
# host: scheme://authority (drop optional user@ and :port), or scp-like git@host:path.
host="$(printf '%s' "$remote_url" | sed -E \
  -e 's#^[a-z][a-z0-9+.-]*://([^/@]+@)?([^:/]+)(:[0-9]+)?(/.*)?$#\2#' \
  -e 's#^[^@/]+@([^:]+):.*$#\1#')"
[ "$slug" = "$remote_url" ] && slug=""
[ "$host" = "$remote_url" ] && host=""
if [ -n "$host" ] && [ -n "$slug" ]; then
  manual_url="https://$host/$slug/pulls/new?source_branch=$TARGET_BRANCH&target_branch=$BASE_BRANCH"
else
  manual_url=""
fi

# BASE_BRANCH(main) 只接受上游同步(见 AGENTS.md 分支策略)，默认不再向它提 PR。
# 需要旧行为时显式设 PR_TO_BASE=1。
if [ "${PR_TO_BASE:-0}" != "1" ]; then
  log "$BASE_BRANCH 只接受上游同步，默认不创建 PR（如确需，设 PR_TO_BASE=1）"
elif [ "$ahead_base" = "0" ]; then
  log "$TARGET_BRANCH 相对 $BASE_BRANCH 无新增提交，无需创建 PR"
elif [ "$DRY" = "1" ]; then
  log "DRY-RUN 跳过 PR 创建；正式执行将创建 PR: $TARGET_BRANCH -> $BASE_BRANCH (新增 $ahead_base 个提交)"
elif [ -n "${RUSTCODE_PR_TOKEN:-}" ] && [ -n "$host" ] && [ -n "$slug" ]; then
  api="https://$host/api/v5/repos/$slug/pulls"
  title="${PR_TITLE:-"chore: sync $TARGET_BRANCH into $BASE_BRANCH"}"
  body="${PR_BODY:-"定时同步 $TARGET_BRANCH -> $BASE_BRANCH，共 $ahead_base 个提交。"}"
  resp="$(curl -sS -X POST "$api" \
    --data-urlencode "access_token=$RUSTCODE_PR_TOKEN" \
    --data-urlencode "title=$title" \
    --data-urlencode "head=$TARGET_BRANCH" \
    --data-urlencode "base=$BASE_BRANCH" \
    --data-urlencode "body=$body" 2>/dev/null || echo '')"
  if printf '%s' "$resp" | grep -qiE 'exist|已经存在|已存在'; then
    log "PR 已存在（未重复创建）: $manual_url"
  elif printf '%s' "$resp" | grep -qE '"number"'; then
    number="$(printf '%s' "$resp" | sed -E 's/.*"number"[[:space:]]*:[[:space:]]*([0-9]+).*/\1/')"
    log "PR 已创建: https://$host/$slug/pulls/$number"
  else
    log "PR 自动创建未成功（降级，不视为失败），请手工创建: $manual_url"
    [ -n "$resp" ] && log "API 响应: $(printf '%s' "$resp" | head -c 300)"
  fi
else
  if [ -n "$manual_url" ]; then
    log "未设置 RUSTCODE_PR_TOKEN，降级为手工创建链接: $manual_url"
  else
    log "未能从 origin 远程地址推断托管平台，请在你的远程仓库 Web 界面手工创建 PR: $TARGET_BRANCH -> $BASE_BRANCH"
  fi
fi

# ---------- 8. 报告 ----------
log "完成。分支=$TARGET_BRANCH 相对 $BASE_BRANCH: ahead=$ahead_base behind=$behind_base"
if [ "$behind_base" != "0" ]; then
  log "提示: $TARGET_BRANCH 落后 $BASE_BRANCH $behind_base 个提交，脚本不会自动合并，请人工确认"
fi
[ "$DRY" = "1" ] && log "本次为 dry-run，未做任何提交/推送"
exit 0
