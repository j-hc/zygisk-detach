#!/system/bin/sh
MODDIR=${0%/*}
[ -f "$MODDIR/detach.txt" ] && "$MODDIR"/system/bin/detach serialize "$MODDIR/detach.txt" "$MODDIR/detach.bin"

# yes we just create the symlink at every boot these tmpfs dirs 
# magisk will add /debug_ramdisk, /sbin on $PATH
[ -f /data/adb/magisk/magisk ] && {
	[ -w /sbin ] && rwdir=/sbin
	[ -w /debug_ramdisk ] && rwdir=/debug_ramdisk
	ln -sf $MODDIR/system/bin/detach $rwdir/detach
}
