#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
AVD="${ANDROID_AVD:-Medium_Phone_API_36.1}"

# shellcheck disable=SC1091
source "$ROOT/scripts/android-env.sh"

stop_heavy() {
  pkill -f 'qemu-system-x86_64.*Medium_Phone' 2>/dev/null || true
  "$ROOT/android/gradlew" --stop >/dev/null 2>&1 || true
}

case "${1:-start}" in
  stop)
    stop_heavy
    pkill -f 'react-native start' 2>/dev/null || true
    pkill -f 'metro' 2>/dev/null || true
    echo "Procesos pesados detenidos (emulador, Metro, Gradle)."
    ;;
  start)
    stop_heavy
    echo "Node: $(node -v)"
    echo "Java: $(java -version 2>&1 | head -1)"
    echo "Arrancando Metro..."
    cd "$ROOT"
    npm start &
    METRO_PID=$!
    sleep 3

    if ! ss -tln | grep -q ':8081'; then
      echo "Metro no arrancó. Revisa la versión de Node (necesitas >= 22)." >&2
      kill "$METRO_PID" 2>/dev/null || true
      exit 1
    fi

    echo "Arrancando emulador $AVD (espera 1-2 min)..."
    emulator -avd "$AVD" -no-snapshot-save &
    for _ in $(seq 1 36); do
      if adb devices 2>/dev/null | grep -q 'emulator-5554[[:space:]]*device'; then
        break
      fi
      sleep 5
    done

    adb reverse tcp:8081 tcp:8081
    npm run android
    ;;
  *)
    echo "Uso: $0 [start|stop]" >&2
    exit 1
    ;;
esac
