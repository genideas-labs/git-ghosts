#!/usr/bin/env bash
# git-ghosts daily scan — scans all repos in a directory and sends summary to Telegram
set -euo pipefail

# Config
SCAN_DIR="${GIT_GHOSTS_SCAN_DIR:-$HOME/genideas-labs}"
BOT_TOKEN="${AW_TELEGRAM_BOT_TOKEN:-}"
CHAT_ID="${AW_TELEGRAM_CHAT_ID:-}"

# Load .env from aw project if available
if [[ -f "$HOME/genideas-labs/aw/.env" ]]; then
    set -a
    source "$HOME/genideas-labs/aw/.env"
    set +a
    BOT_TOKEN="${AW_TELEGRAM_BOT_TOKEN:-$BOT_TOKEN}"
    CHAT_ID="${AW_TELEGRAM_CHAT_ID:-$CHAT_ID}"
fi

send_telegram() {
    local msg="$1"
    if [[ -n "$BOT_TOKEN" && -n "$CHAT_ID" ]]; then
        curl -s -X POST "https://api.telegram.org/bot${BOT_TOKEN}/sendMessage" \
            -d chat_id="$CHAT_ID" \
            -d parse_mode=HTML \
            -d text="$msg" > /dev/null 2>&1
    fi
}

# Scan all git repos
report=""
total_ghosts=0
total_zombies=0
total_orphans=0
repo_count=0

for repo in "$SCAN_DIR"/*/; do
    [[ -d "$repo/.git" ]] || continue
    repo_name=$(basename "$repo")

    git-ghosts scan "$repo" 2>/dev/null || continue
    output=$(git-ghosts report "$repo" 2>/dev/null || echo "")
    if [[ -z "$output" ]]; then
        continue
    fi

    ghosts=$(echo "$output" | grep "Ghost Files" | awk '{print $NF}' || echo "0")
    zombies=$(echo "$output" | grep "Zombie Branches" | awk '{print $NF}' || echo "0")
    orphans=$(echo "$output" | grep "Orphan Commits" | awk '{print $NF}' || echo "0")

    ghosts=${ghosts:-0}
    zombies=${zombies:-0}
    orphans=${orphans:-0}

    if (( ghosts + zombies + orphans > 0 )); then
        report+="📁 <b>${repo_name}</b>: 👻${ghosts} 🧟${zombies} 🔮${orphans}"$'\n'
    fi

    total_ghosts=$((total_ghosts + ghosts))
    total_zombies=$((total_zombies + zombies))
    total_orphans=$((total_orphans + orphans))
    repo_count=$((repo_count + 1))
done

# Build message
msg="🔍 <b>git-ghosts daily scan</b>"$'\n'
msg+="Scanned: ${repo_count} repos in $(basename "$SCAN_DIR")"$'\n'
msg+="━━━━━━━━━━━━━━━━━━"$'\n'

if [[ -n "$report" ]]; then
    msg+="$report"
    msg+="━━━━━━━━━━━━━━━━━━"$'\n'
fi

msg+="Total: 👻${total_ghosts} ghosts | 🧟${total_zombies} zombies | 🔮${total_orphans} orphans"

# Send
send_telegram "$msg"
echo "$msg" | sed 's/<[^>]*>//g'
