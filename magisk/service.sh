#!/bin/sh
# for magisk we just create the symlink at every boot these tmpfs dirs
# magisk will add /debug_ramdisk, /sbin on $PATH
[ -f /data/adb/magisk/magisk ] && {
	[ -w /sbin ] && rwdir=/sbin
	[ -w /debug_ramdisk ] && rwdir=/debug_ramdisk
	ln -sf /data/adb/modules/zygisk-detach/detach $rwdir/detach
}

# EOF
