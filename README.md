# zygisk-detach

Detaches installed apps from Play Store against its aggressive updating policy which ignores the auto-update option user sets.

### How
Hooks libbinder with zygisk

### Usage
* Enable zygisk in Magisk **or flash a Zygisk module if you are on KernelSU**
* Flash zygisk-detach module
* Reboot
* Run the cli: `detach` (or `su -c /data/adb/modules/zygisk-detach/detach` if you are not using termux)  
	or use [zygisk-detach-app](https://github.com/j-hc/zygisk-detach-app) if you dont wanna use the terminal  
	or use the WebUI if you are on KernelSU
* Select apps you wish to detach. Changes are applied immediately, no need to reboot.

### Note
Another way to automatically detach any apps upon flashing the module is to put a `detach.txt` with the package names inside the module zip and the apps will be detached without using the cli once module is flashed.  
An example of the detach.txt that should go in the zygisk-detach zip:
```
com.app1
org.app2
```
