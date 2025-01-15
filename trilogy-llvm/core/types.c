#include "types.h"

const struct trilogy_value trilogy_undefined = { .tag = TAG_UNDEFINED, .payload = 0 };
const struct trilogy_value trilogy_unit = { .tag = TAG_UNIT, .payload = 0 };
const struct trilogy_value trilogy_true = { .tag = TAG_BOOL, .payload = 0 };
const struct trilogy_value trilogy_false = { .tag = TAG_BOOL, .payload = 1 };

struct trilogy_value trilogy_bool(bool b) {
    struct trilogy_value t = { .tag = TAG_BOOL, .payload = b };
    return t;
};

struct trilogy_value trilogy_char(unsigned int c) {
    struct trilogy_value t = { .tag = TAG_CHAR, .payload = c };
    return t;
};

struct trilogy_value trilogy_int(unsigned long i) {
    struct trilogy_value t = { .tag = TAG_INTEGER, .payload = i };
    return t;
};

struct trilogy_value trilogy_callable(void* p) {
    struct trilogy_value t = { .tag = TAG_CALLABLE, .payload = (unsigned long)p };
    return t;
};
