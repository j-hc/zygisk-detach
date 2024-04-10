#include <android/log.h>
#include <fcntl.h>
#include <stdlib.h>
#include <string.h>
#include <sys/sendfile.h>
#include <sys/stat.h>
#include <sys/sysmacros.h>
#include <sys/system_properties.h>
#include <unistd.h>

#include "parcel.hpp"
#include "zygisk.hpp"

#define LOGD(...) __android_log_print(ANDROID_LOG_DEBUG, "zygisk-detach", __VA_ARGS__)

static uint8_t* DETACH_TXT;
static uint8_t HEADERS_LEN;

struct PParcel {
    size_t error;
    uint8_t* data;
    size_t data_size;
};

static inline void detach(PParcel* parcel, uint32_t code) {
    auto p = FakeParcel{parcel->data, 0};
    if (!p.enforceInterface(parcel->data_size, HEADERS_LEN)) return;
    uint32_t pkg_len = p.readInt32();
    uint32_t pkg_len_b = pkg_len * 2 - 1;
    if (pkg_len_b > UINT8_MAX) return;
    if (code == getPackageInfo_code) return;
    auto pkg_ptr = p.readString16(pkg_len);

    size_t i = 0;
    uint8_t dlen;
    while ((dlen = DETACH_TXT[i])) {
        uint8_t* dptr = DETACH_TXT + i + sizeof(dlen);
        i += sizeof(dlen) + dlen;
        if (dlen != pkg_len_b)
            continue;
        if (!memcmp(dptr, pkg_ptr, dlen)) {
            *pkg_ptr = 0;
            return;
        }
    }
}

int (*transact_orig)(void*, int32_t, uint32_t, void*, void*, uint32_t);

int transact_hook(void* self, int32_t handle, uint32_t code, void* pdata, void* preply, uint32_t flags) {
    auto parcel = (PParcel*)pdata;
    detach(parcel, code);
    return transact_orig(self, handle, code, pdata, preply, flags);
}

class ZygiskDetach : public zygisk::ModuleBase {
   public:
    void onLoad(zygisk::Api* api, JNIEnv* env) override {
        this->api = api;
        this->env = env;
    }

    void preServerSpecialize(zygisk::ServerSpecializeArgs* args) override {
        (void)args;
        api->setOption(zygisk::DLCLOSE_MODULE_LIBRARY);
    }

    void preAppSpecialize(zygisk::AppSpecializeArgs* args) override {
        const char* process = env->GetStringUTFChars(args->nice_name, nullptr);
#define vending "com.android.vending"
        if (memcmp(process, vending, STR_LEN(vending))) {
            env->ReleaseStringUTFChars(args->nice_name, process);
            api->setOption(zygisk::Option::DLCLOSE_MODULE_LIBRARY);
            return;
        }
        env->ReleaseStringUTFChars(args->nice_name, process);
        api->setOption(zygisk::FORCE_DENYLIST_UNMOUNT);

        int fd = api->connectCompanion();
        size_t detach_len = this->read_companion(fd);
        close(fd);
        if (detach_len == 0) {
            api->setOption(zygisk::Option::DLCLOSE_MODULE_LIBRARY);
            return;
        }
        char sdk_str[2];
        if (__system_property_get("ro.build.version.sdk", sdk_str)) {
            int sdk = atoi(sdk_str);
            if (sdk >= 30) HEADERS_LEN = 3 * sizeof(uint32_t);
            else if (sdk == 29) HEADERS_LEN = 2 * sizeof(uint32_t);
            else HEADERS_LEN = 1 * sizeof(uint32_t);
        } else {
            LOGD("WARN: could not get sdk version (fallback=3)");
            HEADERS_LEN = 3 * sizeof(uint32_t);
        }

        ino_t inode;
        dev_t dev;
        if (getBinder(&inode, &dev)) {
            this->api->pltHookRegister(dev, inode, "_ZN7android14IPCThreadState8transactEijRKNS_6ParcelEPS1_j",
                                       (void**)&transact_hook, (void**)&transact_orig);
            if (this->api->pltHookCommit()) {
                LOGD("Loaded!");
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
    zygisk::Api* api;
    JNIEnv* env;

    bool getBinder(ino_t* inode, dev_t* dev) {
        FILE* fp = fopen("/proc/self/maps", "r");
        if (!fp) return false;
        char mapbuf[256];
        while (fgets(mapbuf, sizeof(mapbuf), fp)) {
            char flags[8];
            unsigned int dev_major, dev_minor;
            int cur;
            sscanf(mapbuf, "%*s %s %*x %x:%x %lu %*s%n", flags, &dev_major, &dev_minor, inode, &cur);
#define libbinder "libbinder.so"
            if (memcmp(&mapbuf[cur - STR_LEN(libbinder)], libbinder, STR_LEN(libbinder)) == 0 && flags[2] == 'x') {
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
        }
        DETACH_TXT = (uint8_t*)malloc(size + 1);
        auto r = read(fd, DETACH_TXT, size);
        if (r < 0) {
            LOGD("ERROR: read companion");
            return 0;
        }
        if (r != size) {
            LOGD("ERROR: read companion not whole");
            return 0;
        }
        DETACH_TXT[size] = 0;
        return (size_t)size;
    }
};

static void companion_handler(int remote_fd) {
    off_t size = 0;
    int fd = open("/data/adb/modules/zygisk-detach/detach.bin", O_RDONLY);
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

REGISTER_ZYGISK_MODULE(ZygiskDetach)
REGISTER_ZYGISK_COMPANION(companion_handler)
