#include <stdbool.h>

struct trilogy_value {
    unsigned char tag;
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

static const unsigned char TAG_UNDEFINED = 0;
static const unsigned char TAG_UNIT = 1;
static const unsigned char TAG_BOOL = 2;
static const unsigned char TAG_ATOM = 3;
static const unsigned char TAG_CHAR = 4;
static const unsigned char TAG_STRING = 5;
static const unsigned char TAG_INTEGER = 6;
static const unsigned char TAG_BITS = 7;
static const unsigned char TAG_STRUCT = 8;
static const unsigned char TAG_TUPLE = 9;
static const unsigned char TAG_ARRAY = 10;
static const unsigned char TAG_SET = 11;
static const unsigned char TAG_RECORD = 12;
static const unsigned char TAG_CALLABLE = 13;

static const struct trilogy_value trilogy_undefined = { .tag = TAG_UNDEFINED, .payload = 0 };
static const struct trilogy_value trilogy_unit = { .tag = TAG_UNIT, .payload = 0 };
static const struct trilogy_value trilogy_true = { .tag = TAG_BOOL, .payload = 0 };
static const struct trilogy_value trilogy_false = { .tag = TAG_BOOL, .payload = 1 };

static struct trilogy_value trilogy_bool(bool b) {
    struct trilogy_value t = { .tag = TAG_BOOL, .payload = b };
    return t;
};

void untag_unit(struct trilogy_value* val);
bool untag_bool(struct trilogy_value* val);
unsigned long untag_atom(struct trilogy_value* val);
unsigned int untag_char(struct trilogy_value* val);
struct trilogy_string_value* untag_string(struct trilogy_value* val);
long untag_integer(struct trilogy_value* val);
struct trilogy_bits_value* untag_bits(struct trilogy_value* val);
struct trilogy_struct_value* untag_struct(struct trilogy_value* val);
struct trilogy_tuple_value* untag_tuple(struct trilogy_value* val);
struct trilogy_array_value* untag_array(struct trilogy_value* val);
struct trilogy_set_value* untag_set(struct trilogy_value* val);
struct trilogy_record_value* untag_record(struct trilogy_value* val);
void* untag_callable(struct trilogy_value* val);

void trilogy_panic(
    struct trilogy_value* rv,
    struct trilogy_value* message
);

void trilogy_exit(
    struct trilogy_value* rv,
    struct trilogy_value* code
);

void trilogy_printf(
    struct trilogy_value* rv,
    struct trilogy_value* str
);

void trilogy_structural_eq(
    struct trilogy_value* rv,
    struct trilogy_value* lhs,
    struct trilogy_value* rhs
);
