#!/bin/bash

set -e

BRANCHES=(android10-release android11-release android12-release android13-release android14-release)
for B in "${BRANCHES[@]}"; do
	N=IPackageManager.aidl_$B
	if [ -f "$N" ]; then continue; fi
	U="https://android.googlesource.com/platform/frameworks/base/+/refs/heads/$B/core/java/android/content/pm/IPackageManager.aidl?format=TEXT"
	curl "$U" | base64 --decode >"$N"
done

BRANCHES+=(android9) # android9 does not have a seperate branch, only a tag, dl manually
for B in "${BRANCHES[@]}"; do
	N=IPackageManager.aidl_$B
	CODE=$(cat "$N" | sed '0,/^interface /d' | sed '/PackageInfo getPackageInfo/Q' | grep -c ';')
	CODE=$((CODE + 1))
	D=$(cut -d- -f1 <<<"$B")
	echo "#define getPackageInfo_${D}_code $CODE"
done

# generates java with "aidl" binary. but parsing the files with with sed is easier tbh
# AIDL_SRC=$(pwd)/aidl-src
# OUT_DIR=$(pwd)/aidl-out
# function pull_aidl() {
# 	for B in "${BRANCHES[@]}"; do
# 		echo "clone $B"
# 		CLOUT=$AIDL_SRC/$B
# 		if [ -d $CLOUT ]; then continue; fi
# 		git clone --depth 1 -b $B https://android.googlesource.com/platform/frameworks/base $CLOUT
# 		git clone --depth 1 -b $B https://android.googlesource.com/platform/frameworks/native $CLOUT
# 	done
# }
# (
# 	cd $AIDL_SRC/base
# 	~/Android/Sdk/build-tools/34.0.0/aidl --lang=java ./core/java/android/content/pm/IPackageManager.aidl --out $OUT_DIR -I. -I./core/java -I./graphics/java/ -I../native/aidl/binder/
# )
