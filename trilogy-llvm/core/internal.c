#include <execinfo.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include "trilogy_string.h"
#include "trilogy_array.h"
#include "internal.h"
#include "types.h"

void print_trace() {
    void* buffer[100];
    int count = backtrace(buffer, 100);
    char** trace = backtrace_symbols(buffer, count);
    for (int i = 0; i < count; ++i) {
        fprintf(stderr, "%s\n", trace[i]);
    }
    free(trace);
}

void trace(struct trilogy_value* rt) {
    void* buffer[100];
    int count = backtrace(buffer, 100);
    trilogy_array_value* arr = trilogy_array_init_cap(rt, count);

    char** trace = backtrace_symbols(buffer, count);
    for (int i = 0; i < count; ++i) {
        trilogy_string_init_from_c(&arr->contents[i], trace[i]);
    }
    free(trace);
}

void internal_panic(char* msg) {
    fprintf(stderr, "%s", msg);
    print_trace();
    exit(255);
}

void rte(char* expected, unsigned char tag) {
    fprintf(stderr, "runtime type error: expected %s but received %s\n", expected, type_name(tag));
    print_trace();
    exit(255);
}

void exit_(struct trilogy_value* val) {
    switch (val->tag) {
        case TAG_UNIT: exit(0);
        case TAG_INTEGER: exit(val->payload);
        default:
            rte("number", val->tag);
    }
}

bool is_structural_eq(
    struct trilogy_value* lhs,
    struct trilogy_value* rhs
) {
    if (lhs == rhs) return true;
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
            return lhs_st->atom == rhs_st->atom && is_structural_eq(&lhs_st->contents, &rhs_st->contents);
            break;
        }
        case TAG_TUPLE: {
            struct trilogy_tuple_value* lhs_tup = (struct trilogy_tuple_value*)lhs->payload;
            struct trilogy_tuple_value* rhs_tup = (struct trilogy_tuple_value*)rhs->payload;
            return is_structural_eq(&lhs_tup->fst, &rhs_tup->fst) && is_structural_eq(&lhs_tup->snd, &rhs_tup->snd);
        }
        case TAG_ARRAY: {
            struct trilogy_array_value* lhs_arr = (struct trilogy_array_value*)lhs->payload;
            struct trilogy_array_value* rhs_arr = (struct trilogy_array_value*)rhs->payload;
            if (lhs_arr->len != rhs_arr->len) return false;
            for (unsigned long i = 0; i < lhs_arr->len; ++i) {
                if (!is_structural_eq(&lhs_arr->contents[i], &rhs_arr->contents[i])) return false;
            }
            return true;
        }
        case TAG_SET:
        case TAG_RECORD:
        default:
            internal_panic("unimplemented");
            return false;
    }
}
