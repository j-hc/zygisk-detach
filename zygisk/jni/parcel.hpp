#pragma once

#include <stddef.h>
#include <stdint.h>

#define ARR_LEN(a) (sizeof(a) / sizeof((a)[0]))
#define STR_LEN(a) (ARR_LEN(a) - 1)

#define getPackageInfo_code 3

struct FakeParcel {
    unsigned char* data;
    size_t cur;

    void skip(size_t skip);
    uint32_t readInt32();
    char16_t* readString16(uint32_t len);
    bool enforceInterface(size_t, uint8_t);
};
