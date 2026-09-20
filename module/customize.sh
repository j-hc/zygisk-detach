#!/system/bin/sh

chmod -R +x "$MODPATH/bin/"

if [ -n "$KSU" ]; then
	ui_print "- KernelSU detected. Make sure you are using a Zygisk module!"

	uid=$(dumpsys package "com.android.vending" 2>&1 | grep -m1 "uid")
	uid=${uid#*=} uid=${uid%% *}
	if [ -z "$uid" ]; then
		uid=$(dumpsys package "com.android.vending" 2>&1 | grep -m1 "userId")
		uid=${uid#*=} uid=${uid%% *}
	fi
	if [ -z "$uid" ]; then
		ui_print "* UID could not be found for com.android.vending"
		return 1
	fi

	if ! OP=$("$MODPATH/bin/$ARCH/ksu_profile" "$uid" "com.android.vending" 2>&1); then
		ui_print "ERROR ksu_profile: $OP"
	fi
fi

mv -f "$MODPATH/bin/$ARCH/detach" "$MODPATH/detach"
mkdir -p /data/adb/zygisk-detach/

DBIN="/data/adb/zygisk-detach/detach.bin"
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
	ui_print "- Or use the WebUI"
fi
ui_print ""
ui_print "  by j-hc (github.com/j-hc)"
