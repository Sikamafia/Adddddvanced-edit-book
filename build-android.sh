#!/usr/bin/env bash
# Builds a release APK for arm64 only. Needs internet ONCE (crates, npm, Gradle, SDK); the app itself is offline.
# Prereqs: Rust (rustup), Node 18+, JDK 17, Android SDK + NDK (ANDROID_HOME, NDK_HOME set), build-tools 35.0.0.
set -euo pipefail
cd "$(dirname "$0")"
BT="${ANDROID_HOME:?set ANDROID_HOME}/build-tools/35.0.0"
rustup target add aarch64-linux-android
npm install
npm run vendor
npx tauri icon app-icon.png
[ -d src-tauri/gen/android ] || npx tauri android init --ci
npx tauri android build --apk --target aarch64

UNSIGNED=$(find src-tauri/gen/android/app/build/outputs/apk -name "*-unsigned.apk" | head -n 1)
KS="${KEYSTORE:-$HOME/.android/release.jks}"
if [ ! -f "$KS" ]; then
  mkdir -p "$(dirname "$KS")"
  keytool -genkeypair -keystore "$KS" -alias watermark -keyalg RSA -keysize 2048 -validity 10000 \
    -storepass android123 -keypass android123 -dname "CN=Watermark Offline"
fi
"$BT/zipalign" -p -f 4 "$UNSIGNED" aligned.apk
"$BT/apksigner" sign --ks "$KS" --ks-key-alias watermark --ks-pass pass:android123 --key-pass pass:android123 \
  --out watermark-offline-arm64-release.apk aligned.apk
rm -f aligned.apk
ls -lh watermark-offline-arm64-release.apk
