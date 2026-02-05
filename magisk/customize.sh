#!/system/bin/sh

if [ -n "$KSU" ]; then
	ui_print "- KernelSU detected. Make sure you are using a Zygisk module!"
fi

mv -f "$MODPATH/detach-${ARCH}" "$MODPATH/detach"
rm "$MODPATH"/detach-*
chmod +x "$MODPATH/detach"

mkdir -p /data/adb/zygisk-detach/

DBIN="/data/adb/zygisk-detach/detach.bin"

# preserve detach.bin for older versions
if [ -f "/data/adb/modules/zygisk-detach/detach.bin" ]; then
	cp -f "/data/adb/modules/zygisk-detach/detach.bin" $DBIN
fi

if [ -f "$MODPATH/detach.txt" ]; then
	ui_print "- detach.txt inside module: generating detach.bin"
	OP=$("$MODPATH"/detach serialize "$MODPATH/detach.txt" $DBIN 2>&1)
	ui_print "$OP"
elif [ -f "$MODPATH/detach.bin" ]; then
	ui_print "- detach.bin inside module: applying"
	mv -f "$MODPATH/detach.bin" $DBIN
fi

CLIPATH=/data/data/com.termux/files/usr/bin/
if [ -d $CLIPATH ]; then
	echo 'su -c /data/adb/modules/zygisk-detach/detach "$@"' >$CLIPATH/detach
	chmod 777 $CLIPATH/detach
	ui_print "- Run 'detach' in termux after the reboot"
else
	ui_print "- Install termux to use the 'detach' cli"
fi

ui_print "- Or use zygisk-detach-app"
if [ -n "$KSU" ]; then
	ui_print "- Or use the WebUI from KernelSU app"
fi
ui_print ""
ui_print "  by j-hc (github.com/j-hc)"
