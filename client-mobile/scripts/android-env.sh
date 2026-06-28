#!/usr/bin/env bash
# Carga variables para desarrollo Android/React Native en este proyecto.
export ANDROID_HOME="${ANDROID_HOME:-$HOME/Android/Sdk}"
export ANDROID_SDK_ROOT="$ANDROID_HOME"
# React Native 0.86 / Gradle 9 requieren Java 17+. Forzar 21 aunque el shell tenga Java 11.
export JAVA_HOME="/usr/lib/jvm/java-21-openjdk-amd64"

# React Native 0.86 / Metro requieren Node >= 22 (Array.toReversed).
if [ -s "$HOME/.nvm/nvm.sh" ]; then
  # shellcheck disable=SC1090
  source "$HOME/.nvm/nvm.sh"
  nvm use 22 >/dev/null 2>&1 || nvm use >/dev/null 2>&1
fi

export PATH="$JAVA_HOME/bin:$ANDROID_HOME/emulator:$ANDROID_HOME/platform-tools:$ANDROID_HOME/tools:$PATH"
