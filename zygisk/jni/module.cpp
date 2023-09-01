#include <android/log.h>
#include <asm-generic/ioctl.h>
#include <fcntl.h>
#include <inttypes.h>
#include <linux/android/binder.h>
#include <stdio.h>
#include <string.h>
#include <sys/sendfile.h>
#include <sys/stat.h>
#include <sys/sysmacros.h>
#include <unistd.h>

#include "parcel.hpp"
#include "zygisk.hpp"

using zygisk::Api;
using zygisk::AppSpecializeArgs;
using zygisk::ServerSpecializeArgs;

#define LOGD(...) __android_log_print(ANDROID_LOG_DEBUG, "zygisk-detach", __VA_ARGS__)

int (*ioctl_orig)(int, int, char*);

#define DETACH_CAP 512
static unsigned char DETACH_TXT[DETACH_CAP] = {0};
static size_t DETACH_LEN = 0;

void handle_write(binder_transaction_data* btd) {
    size_t detach_len = DETACH_LEN;
    unsigned char* detach_txt = DETACH_TXT;

    unsigned char* data = (unsigned char*)btd->data.ptr.buffer;
    auto p = FakeParcel{data, 0};
    if (!p.enforceInterface(btd->code)) return;

    uint32_t pkg_len = p.readInt32();
    uint32_t pkg_len_b = pkg_len * 2 - 1;
    auto pkg_ptr = p.readString16(pkg_len);

    size_t i = 0;
    while (i < detach_len) {
        uint8_t dlen = detach_txt[i];
        unsigned char* dptr = detach_txt + i + sizeof(dlen);
        i += sizeof(dlen) + dlen;
        if (dlen != pkg_len_b) continue;
        if (!memcmp(dptr, pkg_ptr, dlen)) {
            *pkg_ptr = 0;
            return;
        }
    }
}

int ioctl_hook(int fd, int request, char* argp) {
    if (request == (int)BINDER_WRITE_READ) {
        binder_write_read* bwr = (binder_write_read*)argp;
        if (bwr->write_size > 0) {
            uint32_t cmd = *((uint32_t*)bwr->write_buffer);
            auto btd = (binder_transaction_data*)((char*)bwr->write_buffer + bwr->write_consumed + sizeof(cmd));
            switch (cmd) {
                case BC_TRANSACTION:
                case BC_REPLY:
                    handle_write(btd);
                    break;
                default:
                    break;
            }
        }
    }
    return ioctl_orig(fd, request, argp);
}

class Sigringe : public zygisk::ModuleBase {
   public:
    void onLoad(Api* api, JNIEnv* env) override {
        this->api = api;
        this->env = env;
    }

    void preAppSpecialize(AppSpecializeArgs* args) override {
        const char* process = env->GetStringUTFChars(args->nice_name, nullptr);
        if (strcmp(process, "com.android.vending") && strcmp(process, "com.android.vending:background")) {
            env->ReleaseStringUTFChars(args->nice_name, process);
            api->setOption(zygisk::Option::DLCLOSE_MODULE_LIBRARY);
            return;
        }
        env->ReleaseStringUTFChars(args->nice_name, process);
        api->setOption(zygisk::FORCE_DENYLIST_UNMOUNT);

        int fd = api->connectCompanion();
        DETACH_LEN = this->read_companion(fd);
        close(fd);
        if (DETACH_LEN == 0) {
            api->setOption(zygisk::Option::DLCLOSE_MODULE_LIBRARY);
            return;
        }

        ino_t inode;
        dev_t dev;
        if (getBinder(&inode, &dev)) {
            this->api->pltHookRegister(dev, inode, "ioctl", (void**)&ioctl_hook, (void**)&ioctl_orig);
            if (this->api->pltHookCommit()) {
                // LOGD("Loaded!");
            } else {
                LOGD("ERROR: pltHookCommit");
                api->setOption(zygisk::Option::DLCLOSE_MODULE_LIBRARY);
            }
        } else {
            LOGD("ERROR: Module not found!");
            api->setOption(zygisk::Option::DLCLOSE_MODULE_LIBRARY);
        }
    }

   private:
    Api* api;
    JNIEnv* env;

    bool getBinder(ino_t* inode, dev_t* dev) {
        FILE* fp = fopen("/proc/self/maps", "r");
        if (!fp) return false;
        char mapbuf[256];
        while (fgets(mapbuf, sizeof mapbuf, fp)) {
            char flags[8];
            unsigned int dev_major, dev_minor;
            int cur;
            sscanf(mapbuf, "%*s %s %*x %x:%x %lu%n", flags, &dev_major, &dev_minor, inode, &cur);
            while (mapbuf[cur] != '\n') cur++;
            if (memcmp(&mapbuf[cur - 12], "libbinder.so", 12) == 0 && flags[2] == 'x') {
                *dev = makedev(dev_major, dev_minor);
                fclose(fp);
                return true;
            }
        }
        fclose(fp);
        return false;
    }

    size_t read_companion(int fd) {
        off_t size;
        if (read(fd, &size, sizeof(size)) < 0) {
            LOGD("ERROR: read companion size");
            return 0;
        }
        if (size <= 0) {
            LOGD("ERROR: detach.bin <= 0");
            return 0;
        } else if (size > DETACH_CAP) {
            LOGD("ERROR: detach.bin > %d", DETACH_CAP);
            return 0;
        }
        int received = 0;
        while (received < size) {
            auto red = read(fd, DETACH_TXT + received, size - received);
            if (red < 0) {
                LOGD("ERROR: read companion");
                return 0;
            }
            received += red;
        }
        return (size_t)size;
    }
};

static void companion_handler(int remote_fd) {
    off_t size = 0;
    int fd = open("/sdcard/detach.bin", O_RDONLY);
    if (fd == -1)
        fd = open("/data/adb/modules/zygisk-detach/detach.bin", O_RDONLY);
    if (fd == -1) {
        LOGD("ERROR: companion open");
        if (write(remote_fd, &size, sizeof(size)) < 0)
            LOGD("ERROR: write remote_fd 1");
        return;
    }

    struct stat st;
    if (fstat(fd, &st) == -1) {
        LOGD("ERROR: fstat");
        if (write(remote_fd, &size, sizeof(size)) < 0)
            LOGD("ERROR: write remote_fd 2");
        close(fd);
        return;
    }
    size = st.st_size;
    if (write(remote_fd, &size, sizeof(size)) < 0) {
        LOGD("ERROR: write remote_fd 3");
        close(fd);
        return;
    }
    if (size > 0) {
        if (sendfile(remote_fd, fd, NULL, size) < 0)
            LOGD("ERROR: sendfile");
    }
    close(fd);
}

REGISTER_ZYGISK_MODULE(Sigringe)
REGISTER_ZYGISK_COMPANION(companion_handler)
