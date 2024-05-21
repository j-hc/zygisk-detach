#!/system/bin/sh
MODDIR=${0%/*}
[ -f "$MODDIR/detach.txt" ] && "$MODDIR"/system/bin/detach serialize "$MODDIR/detach.txt" "$MODDIR/detach.bin"
