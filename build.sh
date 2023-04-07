#!/bin/bash

BINARY_NAME="binus_access"
PROFILE="release"

OSes=("x86_64-apple-darwin" "aarch64-apple-darwin" "aarch64-unknown-linux-musl" "i686-pc-windows-gnu" "x86_64-pc-windows-gnu"
      "aarch64-unknown-linux-gnu" "i686-unknown-linux-gnu" "x86_64-unknown-linux-gnu" "i686-unknown-linux-musl"
      "x86_64-unknown-linux-musl" "i586-unknown-linux-gnu" "i586-unknown-linux-musl" "x86_64-unknown-linux-gnux32")
build_binary() {
  echo "Building for $1..."
  if [[ "$1" == *"aarch64-apple-darwin"* ]]; then
    CC="$ZIG_HOME/zig cc -target aarch64-macos --sysroot=$MACOS_SDK" cargo build --target aarch64-apple-darwin \
      --config target.aarch64-apple-darwin.linker=\""$(pwd)/zcc"\" --$PROFILE
  elif [ "$1" == "x86_64-unknown-linux-gnux32" ]; then
    CC="$ZIG_HOME/zig cc -target x86_64-linux-gnux32" cargo build --target x86_64-unknown-linux-gnux32 --$PROFILE
  elif [ "$1" == "x86_64-apple-darwin" ]; then
    CC="$ZIG_HOME/zig cc -target x86_64-macos --sysroot=$MACOS_SDK" cargo build --target x86_64-apple-darwin \
          --config target.x86_64-apple-darwin.linker=\""$(pwd)/zcc"\" --$PROFILE
  else
    cargo zigbuild --$PROFILE --target "$1"
  fi
  if [[ "$1" == *"windows"* ]]; then
    binary_loc=target/"$1"/$PROFILE/$BINARY_NAME.exe
    echo Copying "$binary_loc" to dist/$BINARY_NAME-"$1".exe
    cp "$binary_loc" dist/$BINARY_NAME-"$1".exe
  else
    binary_loc=target/"$1"/$PROFILE/$BINARY_NAME
    echo Copying "$binary_loc" to dist/$BINARY_NAME-"$1"
    cp "$binary_loc" dist/$BINARY_NAME-"$1"
  fi
}

minimized() {
  echo "Creating minimized binaries for $1..."
  extension=""
  if [[ "$1" == *"windows"* ]]; then
      extension=".exe"
  fi
  upx dist/$BINARY_NAME-"$1"$extension -o dist/$BINARY_NAME-"$1"_min$extension
  upx dist/$BINARY_NAME-"$1"_min$extension -t
}

archive() {
  echo "Creating tar.gz file containing all binaries..."
  tar -czf dist_compressed/$BINARY_NAME"_all".tar.gz dist
  echo "Creating tar.gz file with minimized binaries..."
  tar -czf dist_compressed/$BINARY_NAME"_min.tar.gz" dist/*_min*
  echo "Creating tar.gz file with original binaries..."
  tar -czf dist_compressed/$BINARY_NAME.tar.gz $(find dist -type f -not -name "$BINARY_NAME*_min*" | tr '\n' ' ')
}

prepare_directories() {
  mkdir -p dist
  mkdir -p dist_compressed
  rm -r ./dist/* > /dev/null 2>&1
  rm -r ./dist_compressed/* > /dev/null 2>&1
}

check_dependencies() {
  if ! command -v cargo &> /dev/null
  then
    echo "cargo could not be found"
    exit
  fi
  if ! command -v upx &> /dev/null
  then
    echo "upx could not be found"
    exit
  fi
  if [[ -z "$ZIG_HOME" ]]; then
    echo "ZIG_HOME is not set"
    exit
  fi
  if [[ -z "$MACOS_SDK" ]]; then
    echo "MACOS_SDK is not set"
    exit
  fi
  if ! cargo zigbuild --help &> /dev/null
  then
    echo "cargo zigbuild could not be found"
    exit
  fi
  if ! command -v docker &> /dev/null
  then
    echo "docker could not be found... skipping x86_64-apple-darwin..."
    sleep 1
  fi

  for os in "${OSes[@]}"
  do
    rustup target add "$os"
  done
}

run() {
  check_dependencies
  echo "Preparing directories..."
  prepare_directories
  for os in "${OSes[@]}"
  do
    build_binary "$os"
    minimized "$os"
  done

  echo "Creating archives..."
  archive
}

run 2>&1 | tee -a build.log
