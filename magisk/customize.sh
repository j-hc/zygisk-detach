#!/system/bin/sh

if [ -n "$KSU" ]; then
	ui_print "- KernelSU detected. Make sure you are using a Zygisk module!"
fi

mv -f "$MODPATH/detach-${ARCH}" "$MODPATH/detach"
rm "$MODPATH"/detach-*
chmod +x "$MODPATH/detach"

# preserve detach.bin
if [ -f "$NVBASE/modules/zygisk-detach/detach.bin" ]; then
	ui_print "- Preserving existing detach.bin"
	cp -f "$NVBASE/modules/zygisk-detach/detach.bin" "$MODPATH/detach.bin"
fi

if [ -f "$MODPATH/detach.txt" ]; then
	ui_print "- detach.txt inside module! Generating detach.bin"
	OP=$("$MODPATH"/detach serialize "$MODPATH/detach.txt" "$MODPATH/detach.bin" 2>&1)
	ui_print "$OP"
fi

# symlink detach to manager path
# for ez termux usage
manager_paths="/data/adb/ap/bin /data/adb/ksu/bin"
for i in $manager_paths; do
	if [ -d $i ] && [ ! -f $i/detach ]; then
		echo "[+] creating symlink in $i"
		ln -sf /data/adb/modules/zygisk-detach/detach $i/detach
	fi
done

# caused by 6b8e92
sed -i '/zygisk-detach/d' /data/data/com.termux/files/home/.bashrc > /dev/null 2>&1

ui_print "- Run 'su -c detach' in terminal after the reboot"
ui_print "- Or use zygisk-detach-app"
if [ -n "$KSU" ]; then
	ui_print "- Or use the WebUI from KernelSU app"
fi
ui_print ""
ui_print "  by j-hc (github.com/j-hc)"
