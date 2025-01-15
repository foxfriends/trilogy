#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include "internal.h"

void panic(char* msg) {
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

char* tocstr(struct trilogy_value* val) {
    struct trilogy_string_value* str = untag_string(val);
    char* ptr = malloc(sizeof(char) * (str->len + 1));
    strncpy(ptr, str->contents, str->len);
    ptr[str->len] = '\0';
    return ptr;
}

void rte(char* expected, unsigned char tag) {
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
