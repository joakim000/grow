#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

# readonly PACKAGE=rpi3
# readonly FEATURE_FLAG=-F
# readonly FEATURES=syncsend
# readonly TARGET_HOST=rb
# readonly TARGET_PATH=./bin
# readonly TARGET_ARCH=armv7-unknown-linux-gnueabihf
# readonly SOURCE_PATH=./target/${TARGET_ARCH}/debug/${PACKAGE}

PACKAGE=rpi3
FEATURE_FLAG=-F 
FEATURES=syncsend
TARGET_HOST=rb
TARGET_PATH=./bin
TARGET_ARCH=armv7-unknown-linux-gnueabihf
SOURCE_PATH=./target/${TARGET_ARCH}/debug/${PACKAGE}

# cargo build --target=${TARGET_ARCH} --workspace ${FEATURE_FLAG} ${FEATURES} 
cargo build --target=${TARGET_ARCH} --workspace
scp ${SOURCE_PATH} ${TARGET_HOST}:${TARGET_PATH}


