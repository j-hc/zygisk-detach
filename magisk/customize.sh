#!/system/bin/sh

mv -f "$MODPATH/system/bin/detach-${ARCH}" "$MODPATH/system/bin/detach"
rm "$MODPATH"/system/bin/detach-*

# preserve detach.bin
if [ -f "$NVBASE/modules/zygisk-detach/detach.bin" ]; then
	ui_print "- Preserving existing detach.bin"
	cp -f "$NVBASE/modules/zygisk-detach/detach.bin" "$MODPATH/detach.bin"
fi
echo "alias detach='su -c detach'" >/data/data/com.termux/files/home/.bashrc

ui_print "- Run 'detach' or 'su -c detach' in termux after the reboot"
ui_print "  by j-hc (github.com/j-hc)"
