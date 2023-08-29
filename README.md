# zygisk-detach

Detaches installed apps from Play Store against its aggressive updating policy which ignores the auto-update option user sets.

### How
Hooks libbinder with zygisk instead of applying SQL queries to database files

### Usage
* Enable zygisk in magisk or ZygiskOnKernelSU module if you are using KernelSU
* Flash zygisk-detach module
* Reboot
* Run helper cli in termux:
	`$ detach` or `$ su -c detach`
* Select apps you wish to detach. Changes are applied immediately, no need to reboot.

### Notes
To make the usage portable, zygisk-detach generates `detach.bin` in both /sdcard and in the magisk module folder. Which means by putting your cli generated `detach.bin` inside the module zip, you can flash and detach apps without needing to run the cli again.