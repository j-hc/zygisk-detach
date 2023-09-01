
#include "parcel.hpp"

#include <stdint.h>
#include <string.h>

#define ARRAY_LEN(a) (sizeof(a) / sizeof(a[0]))

const size_t PM_DESCRIPTOR_LEN = ARRAY_LEN(u"android.content.pm.IPackageManager") - 1;

// bool String16Eq(const char16_t* s1, size_t len1, const char16_t* s2, size_t len2) {
//     return (len1 == len2 && !memcmp(s1, s2, len1 * sizeof(char16_t)));
// }

void FakeParcel::skip(size_t skip) {
    cur += skip;
}

uint32_t FakeParcel::readInt32() {
    uint32_t i = *((uint32_t*)(data + cur));
    skip(sizeof(i));
    return i;
}

char16_t* FakeParcel::readString16(uint32_t len) {
    char16_t* s = (char16_t*)(data + cur);
    skip(sizeof(len) + (len * sizeof(char16_t)));
    return s;
}

bool FakeParcel::enforceInterfaceIntent() {
    readInt32();
    readInt32();
    uint32_t len = readInt32();
    readString16(len);  // pi;
    readInt32();
    return PM_DESCRIPTOR_LEN == len;
    // return String16Eq(PM_DESCRIPTOR, PM_DESCRIPTOR_LEN, pi, len);
}

bool FakeParcel::enforceInterfaceInfo() {
    readInt32();
    readInt32();
    readInt32();
    uint32_t len = readInt32();
    readString16(len);  // pi;
    return PM_DESCRIPTOR_LEN == len;
    // return String16Eq(len == PM_DESCRIPTOR_LEN && !memcmp(pi, PM_DESCRIPTOR, len * sizeof(char16_t)));
}

bool FakeParcel::enforceInterface(uint32_t code) {
    switch (code) {
        case 3:
        case 51:
        case 83:
            return enforceInterfaceInfo();
        default:
            return false;
    }
}
