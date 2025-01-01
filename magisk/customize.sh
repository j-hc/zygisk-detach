#!/system/bin/sh

if [ -n "$KSU" ]; then
	ui_print "- KernelSU detected. Make sure you are using a Zygisk module!"
fi

mv -f "$MODPATH/detach-${ARCH}" "$MODPATH/detach"
rm "$MODPATH"/detach-*
chmod +x "$MODPATH/detach"

# preserve detach.bin
if [ -f "/data/adb/modules/zygisk-detach/detach.bin" ]; then
	ui_print "- Preserving existing detach.bin"
	cp -f "/data/adb/modules/zygisk-detach/detach.bin" "$MODPATH/detach.bin"
fi

if [ -f "$MODPATH/detach.txt" ]; then
	ui_print "- detach.txt inside module! Generating detach.bin"
	OP=$("$MODPATH"/detach serialize "$MODPATH/detach.txt" "$MODPATH/detach.bin" 2>&1)
	ui_print "$OP"
fi

if ! grep -q 'alias detach' /data/data/com.termux/files/home/.bashrc; then
	echo "alias detach='su -c /data/adb/modules/zygisk-detach/detach'" >>/data/data/com.termux/files/home/.bashrc
	chmod 777 /data/data/com.termux/files/home/.bashrc
fi

ui_print "- Run 'detach' in termux after the reboot"
ui_print "- Or use zygisk-detach-app"
if [ -n "$KSU" ]; then
	ui_print "- Or use the WebUI from KernelSU app"
fi
ui_print ""
ui_print "  by j-hc (github.com/j-hc)"
