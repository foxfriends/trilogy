#include "trilogy_value.h"
#include "internal.h"
#include "trilogy_array.h"
#include "trilogy_bits.h"
#include "trilogy_callable.h"
#include "trilogy_record.h"
#include "trilogy_reference.h"
#include "trilogy_set.h"
#include "trilogy_string.h"
#include "trilogy_struct.h"
#include "trilogy_tuple.h"
#include <assert.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>

const trilogy_value trilogy_undefined = {.tag = TAG_UNDEFINED, .payload = 0};

const trilogy_value trilogy_unit = {.tag = TAG_UNIT, .payload = 0};

void trilogy_unit_untag(trilogy_value* val) {
    if (val->tag != TAG_UNIT) rte("unit", val->tag);
}

void trilogy_value_clone_into(trilogy_value* into, trilogy_value* from) {
    assert(into->tag == TAG_UNDEFINED);
    assert(from->tag != TAG_UNDEFINED);
    switch (from->tag) {
    case TAG_UNIT:
    case TAG_BOOL:
    case TAG_ATOM:
    case TAG_CHAR:
    case TAG_NUMBER:
        *into = *from;
        break;
    case TAG_STRING:
        trilogy_string_clone_into(into, trilogy_string_assume(from));
        break;
    case TAG_BITS:
        trilogy_bits_clone_into(into, trilogy_bits_assume(from));
        break;
    case TAG_STRUCT:
        trilogy_struct_clone_into(into, trilogy_struct_assume(from));
        break;
    case TAG_TUPLE:
        trilogy_tuple_clone_into(into, trilogy_tuple_assume(from));
        break;
    case TAG_ARRAY:
        trilogy_array_clone_into(into, trilogy_array_assume(from));
        break;
    case TAG_SET:
        trilogy_set_clone_into(into, trilogy_set_assume(from));
        break;
    case TAG_RECORD:
        trilogy_record_clone_into(into, trilogy_record_assume(from));
        break;
    case TAG_CALLABLE:
        trilogy_callable_clone_into(into, trilogy_callable_assume(from));
        break;
    case TAG_REFERENCE:
        trilogy_reference_clone_into(into, trilogy_reference_assume(from));
        break;
    default:
        internal_panic("invalid trilogy value\n");
    }
}

void trilogy_value_destroy(trilogy_value* value) {
    switch (value->tag) {
    case TAG_STRING: {
        trilogy_string_value* p = trilogy_string_assume(value);
        trilogy_string_destroy(p);
        free(p);
        break;
    }
    case TAG_BITS: {
        trilogy_bits_value* p = trilogy_bits_assume(value);
        trilogy_bits_destroy(p);
        free(p);
        break;
    }
    case TAG_TUPLE: {
        trilogy_tuple_value* p = trilogy_tuple_assume(value);
        trilogy_tuple_destroy(p);
        free(p);
        break;
    }
    case TAG_STRUCT: {
        trilogy_struct_value* p = trilogy_struct_assume(value);
        trilogy_struct_destroy(p);
        free(p);
        break;
    }
    case TAG_ARRAY: {
        trilogy_array_value* p = trilogy_array_assume(value);
        trilogy_array_destroy(p);
        break;
    }
    case TAG_SET: {
        trilogy_set_value* p = trilogy_set_assume(value);
        trilogy_set_destroy(p);
        break;
    }
    case TAG_RECORD: {
        trilogy_record_value* p = trilogy_record_assume(value);
        trilogy_record_destroy(p);
        break;
    }
    case TAG_CALLABLE: {
        trilogy_callable_value* p = trilogy_callable_assume(value);
        trilogy_callable_destroy(p);
        break;
    }
    case TAG_REFERENCE: {
        trilogy_reference* p = trilogy_reference_assume(value);
        trilogy_reference_destroy(p);
        break;
    }
    default:
        break;
    }
    *value = trilogy_undefined;
}

bool trilogy_value_structural_eq(trilogy_value* lhs, trilogy_value* rhs) {
    assert(lhs->tag != TAG_UNDEFINED);
    assert(rhs->tag != TAG_UNDEFINED);
    if (lhs == rhs) return true;
    if (lhs->tag != rhs->tag) return false;
    switch (lhs->tag) {
    case TAG_UNIT:
    case TAG_BOOL:
    case TAG_ATOM:
    case TAG_CHAR:
    case TAG_NUMBER:
        return lhs->payload == rhs->payload;
    case TAG_CALLABLE: {
        // Closures can only be reference equal, but closure-less functions
        // should be treated all the same no matter how they got cloned up
        trilogy_callable_value* lhs_fn = (trilogy_callable_value*)lhs->payload;
        trilogy_callable_value* rhs_fn = (trilogy_callable_value*)rhs->payload;
        if (lhs_fn->closure == NO_CLOSURE && rhs_fn->closure == NO_CLOSURE) {
            return lhs_fn->function == rhs_fn->function;
        } else {
            return lhs_fn == rhs_fn;
        }
    }
    case TAG_STRING: {
        trilogy_string_value* lhs_str = (trilogy_string_value*)lhs->payload;
        trilogy_string_value* rhs_str = (trilogy_string_value*)rhs->payload;
        if (lhs_str->len != rhs_str->len) return false;
        return strncmp(lhs_str->contents, rhs_str->contents, lhs_str->len) == 0;
    }
    case TAG_BITS: {
        trilogy_bits_value* lhs_bits = (trilogy_bits_value*)lhs->payload;
        trilogy_bits_value* rhs_bits = (trilogy_bits_value*)rhs->payload;
        if (lhs_bits->len != rhs_bits->len) return false;
        if (lhs_bits->len == 0) return true;
        return memcmp(
                   lhs_bits->contents, rhs_bits->contents, lhs_bits->len / 8 + 1
               ) != 0;
    }
    case TAG_STRUCT: {
        trilogy_struct_value* lhs_st = (trilogy_struct_value*)lhs->payload;
        trilogy_struct_value* rhs_st = (trilogy_struct_value*)rhs->payload;
        return lhs_st->atom == rhs_st->atom &&
               trilogy_value_structural_eq(
                   &lhs_st->contents, &rhs_st->contents
               );
        break;
    }
    case TAG_TUPLE: {
        trilogy_tuple_value* lhs_tup = (trilogy_tuple_value*)lhs->payload;
        trilogy_tuple_value* rhs_tup = (trilogy_tuple_value*)rhs->payload;
        return trilogy_value_structural_eq(&lhs_tup->fst, &rhs_tup->fst) &&
               trilogy_value_structural_eq(&lhs_tup->snd, &rhs_tup->snd);
    }
    case TAG_ARRAY: {
        trilogy_array_value* lhs_arr = (trilogy_array_value*)lhs->payload;
        trilogy_array_value* rhs_arr = (trilogy_array_value*)rhs->payload;
        if (lhs_arr->len != rhs_arr->len) return false;
        for (unsigned long i = 0; i < lhs_arr->len; ++i) {
            if (!trilogy_value_structural_eq(
                    &lhs_arr->contents[i], &rhs_arr->contents[i]
                ))
                return false;
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

bool trilogy_value_referential_eq(trilogy_value* lhs, trilogy_value* rhs) {
    assert(lhs->tag != TAG_UNDEFINED);
    assert(rhs->tag != TAG_UNDEFINED);
    if (lhs->tag != rhs->tag) return false;
    switch (lhs->tag) {
    case TAG_ARRAY:
    case TAG_SET:
    case TAG_RECORD: {
        return lhs->payload == rhs->payload;
    }
    case TAG_CALLABLE: {
        // Closures can only be reference equal, but closure-less functions
        // should be treated all the same no matter how they got cloned up
        trilogy_callable_value* lhs_fn = (trilogy_callable_value*)lhs->payload;
        trilogy_callable_value* rhs_fn = (trilogy_callable_value*)rhs->payload;
        if (lhs_fn->closure == NO_CLOSURE && rhs_fn->closure == NO_CLOSURE) {
            return lhs_fn->function == rhs_fn->function;
        } else {
            return lhs_fn == rhs_fn;
        }
    }
    default:
        return trilogy_value_structural_eq(lhs, rhs);
    }
}
