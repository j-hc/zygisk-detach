#include "parcel.hpp"

#include <stdint.h>

#define ARRAY_LEN(a) (sizeof(a) / sizeof(a[0]))
#define PM_DESCRIPTOR_LEN (ARRAY_LEN(u"android.content.pm.IPackageManager") - 1)
#define PM_DESCRIPTOR_BYTES (PM_DESCRIPTOR_LEN * 2)
#define U32SZ (sizeof(uint32_t))

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

bool FakeParcel::enforceInterface(size_t data_size, uint8_t header_count) {
    //              |        headers       |des len|      descriptor       |null+next|
    if (data_size < (U32SZ * header_count) + U32SZ + PM_DESCRIPTOR_BYTES + (U32SZ * 2)) return false;
    skip(U32SZ * header_count);
    uint32_t len = readInt32();
    readString16(len);  // pi;
    return PM_DESCRIPTOR_LEN == len;
}
