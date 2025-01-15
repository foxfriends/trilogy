#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include "runtime.h"
#include "trilogy.h"

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

void trilogy_lookup_atom(
    struct trilogy_value* rv,
    struct trilogy_value* atom
) {
    unsigned int atom_id = untag_atom(atom);
    if (atom_id < atom_registry_sz) {
        rv->tag = TAG_STRING;
        rv->payload = (unsigned long)&atom_registry[atom_id];
    } else {
        *rv = trilogy_unit;
    }
}
