#!/system/bin/sh

[ "$MAGISK_VER_CODE" -lt 26100 ] && {
	ui_print
	ui_print "*******************************"
	ui_print "This module is only supported in Magisk >= v26.1"
	ui_print "*******************************"
	abort
}

mv -f "$MODPATH/system/bin/detach-${ARCH}" "$MODPATH/system/bin/detach"
rm "$MODPATH"/system/bin/detach-*

# preserve detach.bin
if [ -f "$NVBASE/modules/zygisk-detach/detach.bin" ]; then
	ui_print "- Preserving existing detach.bin"
	cp -f "$NVBASE/modules/zygisk-detach/detach.bin" "$MODPATH/detach.bin"
fi

if [ -f "$MODPATH/detach.txt" ]; then
	ui_print "- detach.txt inside module! Generating detach.bin"
	APPS=$(tr -d ' \t\r' <"$MODPATH/detach.txt" | grep -v '^$')
	for app in $APPS; do
		ui_print "   $app"
	done
	if ! OP=$("$MODPATH"/system/bin/detach --serialize "$MODPATH/detach.txt" 2>&1); then
		ui_print "$OP"
	fi
fi

ALIAS="alias detach='su -c detach'"
BASHRC="/data/data/com.termux/files/home/.bashrc"
if grep -qxF "$ALIAS" "$BASHRC" || echo "$ALIAS" >>"$BASHRC"; then
	ui_print "- Run 'detach' in termux after the reboot"
else
	ui_print "- Run 'su -c detach' in terminal after the reboot"
fi

ui_print "  by j-hc (github.com/j-hc)"
