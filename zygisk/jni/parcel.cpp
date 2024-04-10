#include "parcel.hpp"

#include <stdint.h>

#define PM_DESCRIPTOR_LEN (STR_LEN(u"android.content.pm.IPackageManager"))
#define PM_DESCRIPTOR_BYTES (PM_DESCRIPTOR_LEN * 2)

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

bool FakeParcel::enforceInterface(size_t data_size, uint8_t headers) {
    //            | headers |     des len      |      descriptor       |    null+next       |
    if (data_size < headers + sizeof(uint32_t) + PM_DESCRIPTOR_BYTES + (sizeof(uint32_t) * 2)) return false;
    skip(headers);
    uint32_t len = readInt32();
    readString16(len);  // pi;
    return PM_DESCRIPTOR_LEN == len;
}
