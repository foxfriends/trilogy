#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include "trilogy.h"

static void panic(char* msg) {
    printf("%s", msg);
    exit(255);
}

char* type_name(unsigned char tag) {
    switch (tag) {
        case TAG_UNDEFINED: return "undefined";
        case TAG_UNIT:      return "unit";
        case TAG_BOOL:      return "boolean";
        case TAG_ATOM:      return "atom";
        case TAG_CHAR:      return "character";
        case TAG_STRING:    return "string";
        case TAG_INTEGER:   return "number";
        case TAG_BITS:      return "bits";
        case TAG_STRUCT:    return "struct";
        case TAG_TUPLE:     return "tuple";
        case TAG_ARRAY:     return "array";
        case TAG_SET:       return "set";
        case TAG_RECORD:    return "record";
        case TAG_CALLABLE:  return "callable";
        default:
            panic("runtime error: invalid trilogy_value tag\n");
            return "undefined";
    }
}

static char* tocstr(struct trilogy_value* val) {
    struct trilogy_string_value* str = untag_string(val);
    char* ptr = malloc(sizeof(char) * (str->len + 1));
    strncpy(ptr, str->contents, str->len);
    ptr[str->len] = '\0';
    return ptr;
}

static void rte(char* expected, unsigned char tag) {
    printf("runtime type error: expected %s but received %s\n", expected, type_name(tag));
    exit(255);
}

void untag_unit(struct trilogy_value* val) {
    if (val->tag != TAG_UNIT) rte("unit", val->tag);
}

bool untag_bool(struct trilogy_value* val) {
    if (val->tag != TAG_BOOL) rte("bool", val->tag);
    return (bool)val->payload;
}

unsigned long untag_atom(struct trilogy_value* val) {
    if (val->tag != TAG_ATOM) rte("atom", val->tag);
    return val->payload;
}

unsigned int untag_char(struct trilogy_value* val) {
    if (val->tag != TAG_CHAR) rte("char", val->tag);
    return (unsigned int)val->payload;
}

struct trilogy_string_value* untag_string(struct trilogy_value* val) {
    if (val->tag != TAG_STRING) rte("string", val->tag);
    return (struct trilogy_string_value*)val->payload;
}

long untag_integer(struct trilogy_value* val) {
    if (val->tag != TAG_INTEGER) rte("integer", val->tag);
    return (long)val->payload;
}

struct trilogy_bits_value* untag_bits(struct trilogy_value* val) {
    if (val->tag != TAG_BITS) rte("bits", val->tag);
    return (struct trilogy_bits_value*)val->payload;
}

struct trilogy_struct_value* untag_struct(struct trilogy_value* val) {
    if (val->tag != TAG_STRUCT) rte("struct", val->tag);
    return (struct trilogy_struct_value*)val->payload; }

struct trilogy_tuple_value* untag_tuple(struct trilogy_value* val) {
    if (val->tag != TAG_TUPLE) rte("tuple", val->tag);
    return (struct trilogy_tuple_value*)val->payload;
}

struct trilogy_array_value* untag_array(struct trilogy_value* val) {
    if (val->tag != TAG_ARRAY) rte("array", val->tag);
    return (struct trilogy_array_value*)val->payload;
}

struct trilogy_set_value* untag_set(struct trilogy_value* val) {
    if (val->tag != TAG_SET) rte("set", val->tag);
    return (struct trilogy_set_value*)val->payload;
}

struct trilogy_record_value* untag_record(struct trilogy_value* val) {
    if (val->tag != TAG_RECORD) rte("record", val->tag);
    return (struct trilogy_record_value*)val->payload;
}

void* untag_callable(struct trilogy_value* val) {
    if (val->tag != TAG_CALLABLE) rte("callable", val->tag);
    return (void*)val->payload;
}

void trilogy_panic(
    struct trilogy_value* rv,
    struct trilogy_value* val
) {
    panic(tocstr(val));
}

void trilogy_exit(
    struct trilogy_value* rv,
    struct trilogy_value* val
) {
    switch (val->tag) {
        case TAG_UNIT: exit(0);
        case TAG_INTEGER: exit(val->payload);
        default:
            rte("number", val->tag);
    }
}

void trilogy_printf(
    struct trilogy_value* rv,
    struct trilogy_value* val
) {
    char* ptr = tocstr(val);
    printf("%s", ptr);
    free(ptr);
    *rv = trilogy_unit;
}

static bool structural_eq(
    struct trilogy_value* lhs,
    struct trilogy_value* rhs
) {
    if (lhs->tag != rhs->tag) return false;
    switch (lhs->tag) {
        case TAG_UNIT:
        case TAG_BOOL:
        case TAG_ATOM:
        case TAG_CHAR:
        case TAG_INTEGER:
        case TAG_CALLABLE:
            return lhs->payload == rhs->payload;
        case TAG_STRING: {
            struct trilogy_string_value* lhs_str = (struct trilogy_string_value*)lhs->payload;
            struct trilogy_string_value* rhs_str = (struct trilogy_string_value*)rhs->payload;
            if (lhs_str->len != rhs_str->len) return false;
            return strncmp(lhs_str->contents, rhs_str->contents, lhs_str->len) == 0;
        }
        case TAG_BITS: {
            struct trilogy_bits_value* lhs_bits = (struct trilogy_bits_value*)lhs->payload;
            struct trilogy_bits_value* rhs_bits = (struct trilogy_bits_value*)rhs->payload;
            if (lhs_bits->len != rhs_bits->len) return false;
            if (lhs_bits->len == 0) return true;
            return memcmp(lhs_bits->contents, rhs_bits->contents, lhs_bits->len / 8 + 1) != 0;
        }
        case TAG_STRUCT: {
            struct trilogy_struct_value* lhs_st = (struct trilogy_struct_value*)lhs->payload;
            struct trilogy_struct_value* rhs_st = (struct trilogy_struct_value*)rhs->payload;
            return lhs_st->atom == rhs_st->atom && structural_eq(lhs_st->contents, rhs_st->contents);
            break;
        }
        case TAG_TUPLE: {
            struct trilogy_tuple_value* lhs_tup = (struct trilogy_tuple_value*)lhs->payload;
            struct trilogy_tuple_value* rhs_tup = (struct trilogy_tuple_value*)rhs->payload;
            return structural_eq(lhs_tup->fst, rhs_tup->fst) && structural_eq(lhs_tup->snd, rhs_tup->snd);
        }
        case TAG_ARRAY: {
            struct trilogy_array_value* lhs_arr = (struct trilogy_array_value*)lhs->payload;
            struct trilogy_array_value* rhs_arr = (struct trilogy_array_value*)rhs->payload;
            if (lhs_arr->len != rhs_arr->len) return false;
            for (unsigned long i = 0; i < lhs_arr->len; ++i) {
                if (!structural_eq(&lhs_arr->contents[i], &rhs_arr->contents[i])) return false;
            }
            return true;
        }
        case TAG_SET:
        case TAG_RECORD:
        default:
            panic("unimplemented");
            return false;
    }
}

void trilogy_structural_eq(
    struct trilogy_value* rv,
    struct trilogy_value* lhs,
    struct trilogy_value* rhs
) {
    *rv = trilogy_bool(structural_eq(lhs, rhs));
}
