#pragma once
#include <stdbool.h>

typedef enum trilogy_tag : unsigned char {
    TAG_UNDEFINED = 0,
    TAG_UNIT = 1,
    TAG_BOOL = 2,
    TAG_ATOM = 3,
    TAG_CHAR = 4,
    TAG_STRING = 5,
    TAG_INTEGER = 6,
    TAG_BITS = 7,
    TAG_STRUCT = 8,
    TAG_TUPLE = 9,
    TAG_ARRAY = 10,
    TAG_SET = 11,
    TAG_RECORD = 12,
    TAG_CALLABLE = 13,
} trilogy_tag;

struct trilogy_value {
    trilogy_tag tag;
    unsigned long payload;
};

struct trilogy_string_value {
    unsigned long len;
    char* contents;
};

struct trilogy_bits_value {
    unsigned long len; // len is the number of bits, the length of contents is len / 8
    unsigned char* contents;
};

struct trilogy_struct_value {
    unsigned long atom;
    struct trilogy_value* contents;
};

struct trilogy_tuple_value {
    struct trilogy_value* fst;
    struct trilogy_value* snd;
};

struct trilogy_array_value {
    unsigned long len;
    unsigned long cap;
    struct trilogy_value* contents;
};

struct trilogy_set_value {
    unsigned long len;
    unsigned long cap;
    struct trilogy_value* contents;
};

struct trilogy_record_value {
    unsigned long len;
    unsigned long cap;
    struct trilogy_tuple_value* contents;
};

extern const struct trilogy_value trilogy_undefined;
extern const struct trilogy_value trilogy_unit;
extern const struct trilogy_value trilogy_true;
extern const struct trilogy_value trilogy_false;

struct trilogy_value trilogy_bool(bool b);
struct trilogy_value trilogy_char(unsigned int c);
struct trilogy_value trilogy_int(unsigned long i);
struct trilogy_value trilogy_callable(void* p);
