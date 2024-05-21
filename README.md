# zygisk-detach

Detaches installed apps from Play Store against its aggressive updating policy which ignores the auto-update option user sets.

### How
Hooks libbinder with zygisk

### Usage
* Enable zygisk in Magisk **or** [ZygiskNext](https://github.com/Dr-TSNG/ZygiskNext) module if you are on KernelSU
* Flash zygisk-detach module
* Reboot
* Run the cli in termux: `su -c detach`  
	or use [zygisk-detach-app](https://github.com/j-hc/zygisk-detach-app) if you dont wanna use the terminal  
	or use the WebUI if you are on KernelSU
* Select apps you wish to detach. Changes are applied immediately, no need to reboot.

### Notes
zygisk-detach reads the generated `detach.bin` in the magisk module folder (`/data/adb/modules/zygisk-detach/detach.bin`) which means to make the usage portable, by putting your cli generated `detach.bin` inside the module zip, you can flash and detach apps without needing to run the cli again (for example between switching roms). You can get your generated `detach.bin` using the cli or from magisk module folder.  

Or the same way you can put a `detach.txt` with the package names inside the module zip and it will be serialized into a `detach.bin` when flashing
