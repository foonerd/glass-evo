#!/bin/bash
# Build the Android app of the bundle: Glass's remote app with the face in
# it. The app project is Glass's own (remote/android), taken from the Glass
# this face is built on; SDL2 for Android comes from the SDL source (pinned
# by version and checksum); the face is libmain.so for each ABI, built with
# cargo-ndk; Gradle puts the app around them; it is signed with the keystore
# the environment names, else with a throwaway key for a build nobody
# publishes. The result is dist/glass-evo-<version>-android.apk.
#
# The app is the same app as Glass's standalone remote (the same
# application id) and, signed with the same key, installs over it and keeps
# its settings. Its version code is that of the Glass it is built on, as the
# standalone's is, so Android compares the two by the Glass in them.
#
# Runs where the Android SDK (with the NDK 27, build tools 35 and platform
# 35), a JDK, Gradle, the Android Rust targets and cargo-ndk are installed,
# such as the image Glass's scripts/android/Dockerfile describes.
#
# Signing, for a published build:
#   ANDROID_KEYSTORE        path of the keystore (a .jks)
#   ANDROID_KEYSTORE_PASS   its password
#   ANDROID_KEY_ALIAS       the key's alias (default glass)

set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
cd "$ROOT"
TARGET_DIR=${CARGO_TARGET_DIR:-$ROOT/target}
# Beside the sources, not under target/: a CI cache walks target/ and trips
# over folders in the SDL source named target and trybuild.
WORK=$ROOT/.android
APP=$WORK/project/app

SDL_VERSION=2.32.10
SDL_SHA256=5f5993c530f084535c65a6879e9b26ad441169b3e25d789d83287040a9ca5165
SDL_SRC=$WORK/sdl/SDL2-$SDL_VERSION
ABIS="arm64-v8a armeabi-v7a x86_64"
PLATFORM=24

: "${ANDROID_HOME:?ANDROID_HOME must name the Android SDK}"
NDK=${ANDROID_NDK_HOME:-$(ls -d "$ANDROID_HOME"/ndk/27.* 2>/dev/null | sort -V | tail -n1)}
[ -x "$NDK/ndk-build" ] || { echo "ship-android: no NDK at $NDK" >&2; exit 1; }
BUILD_TOOLS=$(ls -d "$ANDROID_HOME"/build-tools/35.* 2>/dev/null | sort -V | tail -n1)
[ -x "$BUILD_TOOLS/apksigner" ] || { echo "ship-android: no build tools 35 under $ANDROID_HOME" >&2; exit 1; }
command -v cargo-ndk >/dev/null 2>&1 || cargo ndk --version >/dev/null 2>&1 || { echo "ship-android: cargo-ndk is missing (cargo install cargo-ndk)" >&2; exit 1; }
command -v gradle >/dev/null 2>&1 || { echo "ship-android: gradle is missing" >&2; exit 1; }
export ANDROID_NDK_HOME=$NDK

VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)
# The Glass this face is built on: where its app project is, and its version.
GLASS=$(cargo metadata --format-version 1 --locked | python3 -c '
import json, os, sys
packages = [p for p in json.load(sys.stdin)["packages"] if p["name"] == "glass"]
if packages:
    print(os.path.dirname(os.path.dirname(os.path.dirname(packages[0]["manifest_path"]))), packages[0]["version"])')
GLASS_SRC=${GLASS% *}
GLASS_RELEASE=${GLASS##* }
[ -n "$GLASS_SRC" ] && [ -f "$GLASS_SRC/remote/android/app/build.gradle" ] || { echo "ship-android: Glass's Android app project was not found (looked under: $GLASS_SRC)" >&2; exit 1; }
# 0.8.24 -> 8024, as the standalone app of that Glass has it.
major=${GLASS_RELEASE%%.*}
rest=${GLASS_RELEASE#*.}
minor=${rest%%.*}
patch=${rest#*.}
export GLASS_VERSION="$GLASS_RELEASE with glass-evo $VERSION"
export GLASS_VERSION_CODE=$((major * 100000 + minor * 1000 + patch))
echo "ship-android: glass-evo $VERSION on Glass $GLASS_RELEASE (version code $GLASS_VERSION_CODE)"

# The app project, fresh from the Glass checkout each time.
rm -rf "$WORK/project" && mkdir -p "$WORK"
cp -r "$GLASS_SRC/remote/android" "$WORK/project"
chmod -R u+w "$WORK/project"
rm -rf "$WORK/project/.sdl" "$WORK/project/build" "$WORK/project/.gradle" "$APP/build" "$APP/src/main/jniLibs"

# SDL's source: the Android build of libSDL2.so and the Java side of the
# activity both come from it.
if [ ! -f "$SDL_SRC/Android.mk" ]; then
  mkdir -p "$WORK/sdl"
  tarball=$WORK/sdl/SDL2-$SDL_VERSION.tar.gz
  "$ROOT/scripts/fetch.sh" "https://github.com/libsdl-org/SDL/releases/download/release-$SDL_VERSION/SDL2-$SDL_VERSION.tar.gz" \
    "$tarball" "$SDL_SHA256"
  tar -C "$WORK/sdl" -xzf "$tarball"
fi
rm -rf "$APP/jni/SDL" && ln -s "$SDL_SRC" "$APP/jni/SDL"
rm -rf "$APP/src/main/java/org" && mkdir -p "$APP/src/main/java/org/libsdl/app"
cp "$SDL_SRC"/android-project/app/src/main/java/org/libsdl/app/*.java "$APP/src/main/java/org/libsdl/app/"

echo "ship-android: SDL2 for $ABIS"
JNI_LIBS=$APP/src/main/jniLibs
rm -rf "$JNI_LIBS" "$TARGET_DIR/android/obj"
"$NDK/ndk-build" -j"$(nproc)" \
  NDK_PROJECT_PATH=null \
  APP_BUILD_SCRIPT="$APP/jni/Android.mk" \
  NDK_APPLICATION_MK="$APP/jni/Application.mk" \
  NDK_OUT="$TARGET_DIR/android/obj" \
  NDK_LIBS_OUT="$JNI_LIBS" >/dev/null

echo "ship-android: the face for $ABIS"
# Each Rust target links against the libSDL2.so of its ABI.
export CARGO_TARGET_AARCH64_LINUX_ANDROID_RUSTFLAGS="-L native=$JNI_LIBS/arm64-v8a"
export CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_RUSTFLAGS="-L native=$JNI_LIBS/armeabi-v7a"
export CARGO_TARGET_X86_64_LINUX_ANDROID_RUSTFLAGS="-L native=$JNI_LIBS/x86_64"
targets=""
for abi in $ABIS; do targets="$targets -t $abi"; done
# shellcheck disable=SC2086
cargo ndk $targets -P "$PLATFORM" -o "$JNI_LIBS" build --release --locked -p glass-evo-android

echo "ship-android: the app"
(cd "$WORK/project" && gradle --quiet --no-daemon assembleRelease)
unsigned=$(ls "$APP"/build/outputs/apk/release/*.apk | head -n1)
[ -f "$unsigned" ] || { echo "ship-android: Gradle produced no apk" >&2; exit 1; }

mkdir -p dist
out=dist/glass-evo-$VERSION-android.apk
aligned=$TARGET_DIR/android/aligned.apk
"$BUILD_TOOLS/zipalign" -f -p 4 "$unsigned" "$aligned"
if [ -n "${ANDROID_KEYSTORE:-}" ]; then
  echo "ship-android: signing with $ANDROID_KEYSTORE"
  "$BUILD_TOOLS/apksigner" sign --ks "$ANDROID_KEYSTORE" --ks-pass "pass:${ANDROID_KEYSTORE_PASS:?}" \
    --ks-key-alias "${ANDROID_KEY_ALIAS:-glass}" --out "$out" "$aligned"
else
  # A key of this build alone: the apk installs, and neither a build signed
  # with the project's key nor the standalone app updates it. Not for a release.
  throwaway=$TARGET_DIR/android/throwaway.jks
  [ -f "$throwaway" ] || keytool -genkeypair -keystore "$throwaway" -storepass throwaway -alias glass \
    -keyalg RSA -keysize 2048 -validity 30 -dname "CN=glass-evo throwaway build" >/dev/null 2>&1
  echo "ship-android: no ANDROID_KEYSTORE, signing with a throwaway key"
  "$BUILD_TOOLS/apksigner" sign --ks "$throwaway" --ks-pass pass:throwaway --ks-key-alias glass --out "$out" "$aligned"
fi
"$BUILD_TOOLS/apksigner" verify --print-certs "$out" | sed 's/^/ship-android:   /' | head -n 3
echo "ship-android: payload"
ls -l "$out"
