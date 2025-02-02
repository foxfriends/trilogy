#include "internal.h"
#include "trilogy_array.h"
#include "trilogy_bits.h"
#include "trilogy_boolean.h"
#include "trilogy_character.h"
#include "trilogy_number.h"
#include "trilogy_string.h"
#include "trilogy_value.h"
#include "types.h"
#include <execinfo.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>

void panic(trilogy_value* rv, trilogy_value* val) {
    internal_panic(trilogy_string_as_c(trilogy_string_untag(val)));
}

void print(trilogy_value* rv, trilogy_value* val) {
    char* ptr = trilogy_string_as_c(trilogy_string_untag(val));
    printf("%s", ptr);
    free(ptr);
    *rv = trilogy_unit;
}

void trace(trilogy_value* rt) {
    void* buffer[100];
    int count = backtrace(buffer, 100);
    trilogy_array_value* arr = trilogy_array_init_cap(rt, count);

    char** trace = backtrace_symbols(buffer, count);
    for (int i = 0; i < count; ++i) {
        trilogy_string_init_from_c(&arr->contents[i], trace[i]);
    }
    free(trace);
}

void referential_eq(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_boolean_init(rv, trilogy_value_referential_eq(lhs, rhs));
}

void structural_eq(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_boolean_init(rv, trilogy_value_structural_eq(lhs, rhs));
}

void length(trilogy_value* rv, trilogy_value* val) {
    switch (val->tag) {
    case TAG_STRING:
        trilogy_number_init(rv, trilogy_string_len(trilogy_string_assume(val)));
        break;
    case TAG_ARRAY:
        trilogy_number_init(rv, trilogy_array_len(trilogy_array_assume(val)));
        break;
    default:
        rte("string, bits, array, set, or record", val->tag);
    }
}

void push(trilogy_value* rv, trilogy_value* arr, trilogy_value* val) {
    switch (arr->tag) {
    case TAG_ARRAY:
        trilogy_array_push(trilogy_array_assume(arr), val);
        break;
    default:
        rte("array, set, or record", arr->tag);
    }
    *rv = trilogy_unit;
}

void append(trilogy_value* rv, trilogy_value* arr, trilogy_value* val) {
    switch (arr->tag) {
    case TAG_ARRAY:
        trilogy_array_append(trilogy_array_assume(arr), val);
        break;
    default:
        rte("array, set, or record", arr->tag);
    }
    *rv = trilogy_unit;
}

void member_access(trilogy_value* rv, trilogy_value* c, trilogy_value* index) {
    switch (c->tag) {
    case TAG_STRING: {
        unsigned long i = trilogy_number_untag(index);
        unsigned int ch = trilogy_string_at(trilogy_string_assume(c), i);
        trilogy_character_init(rv, ch);
        break;
    }
    case TAG_BITS: {
        unsigned int i = trilogy_number_untag(index);
        bool b = trilogy_bits_at(trilogy_bits_assume(c), i);
        trilogy_boolean_init(rv, b);
        break;
    }
    case TAG_ARRAY: {
        unsigned long i = trilogy_number_untag(index);
        trilogy_array_at(rv, trilogy_array_assume(c), i);
        break;
    }
    default:
        rte("string, bits, tuple, array, or record", c->tag);
    }
}
