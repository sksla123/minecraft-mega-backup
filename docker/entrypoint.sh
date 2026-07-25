#!/bin/bash
# ==========================================
# docker/entrypoint.sh
# ==========================================
set -e

CONFIG_FILE="${CONFIG_PATH:-/config/config.toml}"

echo "========================================"
echo "mcmgb Docker Entrypoint"
echo "========================================"

echo "MEGAcmd 데몬을 백그라운드에서 시작합니다..."
# 데몬 실행 로그를 버리지 않고 남김
mega-cmd-server &

echo "MEGAcmd 데몬의 준비 완료를 대기 중..."
MAX_RETRIES=30
RETRY_COUNT=0

# mega-version 명령어는 로그인 여부와 관계없이 서버와 통신만 되면 0을 반환합니다.
until mega-version > /dev/null 2>&1 || [ $RETRY_COUNT -eq $MAX_RETRIES ]; do
    sleep 1
    RETRY_COUNT=$((RETRY_COUNT + 1))
done

if [ $RETRY_COUNT -eq $MAX_RETRIES ]; then
    echo "[ERROR] MEGAcmd 데몬이 제한 시간 내에 시작되지 않았습니다."
    if [ -f /root/.megaCmd/megacmdserver.log ]; then
        echo "=== /root/.megaCmd/megacmdserver.log 내용 ==="
        cat /root/.megaCmd/megacmdserver.log
    fi
    exit 1
fi

echo "MEGAcmd 데몬 준비 완료."

if [ -n "$MEGA_EMAIL" ] && [ -n "$MEGA_PASSWORD" ]; then
    echo "MEGA 로그인을 시도합니다..."
    if ! mega-login "$MEGA_EMAIL" "$MEGA_PASSWORD"; then
        echo "첫 번째 로그인 시도 실패. 3초 후 다시 시도합니다..."
        sleep 3
        mega-login "$MEGA_EMAIL" "$MEGA_PASSWORD"
    fi

    unset MEGA_EMAIL
    unset MEGA_PASSWORD
    echo "환경변수에서 MEGA 인증 정보를 안전하게 삭제했습니다."
else
    echo "[WARN] MEGA_EMAIL 또는 MEGA_PASSWORD가 누락되었습니다."
fi

if [ ! -f "$CONFIG_FILE" ]; then
    echo "설정 파일($CONFIG_FILE)이 마운트되지 않아 환경변수를 기반으로 자동 생성합니다."
    mkdir -p "$(dirname "$CONFIG_FILE")"

    cat <<EOF > "$CONFIG_FILE"
[daemon]
interval = "${MCMGB_DAEMON_INTERVAL:-12h}"
startup_retry = ${MCMGB_STARTUP_RETRY:-10}
startup_retry_interval = "${MCMGB_STARTUP_RETRY_INTERVAL:-1m}"

[minecraft]
server_dir = "${MCMGB_MC_SERVER_DIR:-/home/minecraft/server}"
backup_temp_dir = "${MCMGB_MC_BACKUP_DIR:-/home/minecraft/backups}"

[mega]
remote_dir = "${MCMGB_MEGA_REMOTE_DIR:-/MinecraftBackups}"
extra_margin_gb = ${MCMGB_MEGA_EXTRA_MARGIN_GB:-5}
keep_versions = ${MCMGB_MEGA_KEEP_VERSIONS:-3}

[shoutrrr]
url = "${MCMGB_SHOUTRRR_URL:-}"

[rcon]
enable = ${MCMGB_RCON_ENABLE:-true}
address = "${MCMGB_RCON_ADDRESS:-127.0.0.1:25575}"
password = "${MCMGB_RCON_PASSWORD:-}"
EOF
fi

echo "의존성 및 MEGA 연동 상태를 점검합니다 (RCON 검사 제외)..."
TMP_CONFIG="/tmp/temp_check_config.toml"
cp "$CONFIG_FILE" "$TMP_CONFIG"
sed -i 's/enable = true/enable = false/g' "$TMP_CONFIG"

mcmgb --config "$TMP_CONFIG" --check
rm -f "$TMP_CONFIG"

echo "mcmgb 데몬 프로세스를 시작합니다..."
exec mcmgb --config "$CONFIG_FILE"

