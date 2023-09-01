#!/system/bin/sh

mv -f "$MODPATH/system/bin/detach-${ARCH}" "$MODPATH/system/bin/detach"
rm "$MODPATH"/system/bin/detach-*

# preserve detach.bin
if [ -f "$NVBASE/modules/zygisk-detach/detach.bin" ]; then
	ui_print "- Preserving existing detach.bin"
	cp -f "$NVBASE/modules/zygisk-detach/detach.bin" "$MODPATH/detach.bin"
fi
ALIAS="alias detach='su -c detach'"
BASHRC="/data/data/com.termux/files/home/.bashrc"
if grep -qxF "$ALIAS" "$BASHRC" || echo "$ALIAS" >>"$BASHRC"; then
	ui_print "- Run 'detach' in termux after the reboot"
else
	ui_print "- Run 'su -c detach' in terminal after the reboot"
fi

ui_print "  by j-hc (github.com/j-hc)"
