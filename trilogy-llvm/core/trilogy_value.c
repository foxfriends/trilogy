#include <stdlib.h>
#include <stdbool.h>
#include "trilogy_value.h"
#include "trilogy_array.h"
#include "trilogy_string.h"
#include "trilogy_bits.h"
#include "trilogy_boolean.h"
#include "trilogy_tuple.h"
#include "trilogy_struct.h"
#include "trilogy_array.h"
#include "trilogy_set.h"
#include "trilogy_record.h"
#include "trilogy_callable.h"
#include "internal.h"

const trilogy_value trilogy_undefined = { .tag = TAG_UNDEFINED, .payload = 0 };

const trilogy_value trilogy_unit = { .tag = TAG_UNIT, .payload = 0 };

void untag_unit(trilogy_value* val) {
    if (val->tag != TAG_UNIT) rte("unit", val->tag);
}

trilogy_value trilogy_value_clone(trilogy_value* value) {
    switch (value->tag) {
        case TAG_UNIT:
        case TAG_BOOL:
        case TAG_ATOM:
        case TAG_CHAR:
        case TAG_INTEGER:
            return *value;
        case TAG_STRING:
            return trilogy_string_clone(assume_string(value));
        case TAG_BITS:
            return trilogy_bits_clone(assume_bits(value));
        case TAG_STRUCT:
            return trilogy_struct_clone(assume_struct(value));
        case TAG_TUPLE:
            return trilogy_tuple_clone(assume_tuple(value));
        case TAG_ARRAY:
            return trilogy_array_clone(assume_array(value));
        case TAG_SET:
            return trilogy_set_clone(assume_set(value));
        case TAG_RECORD:
            return trilogy_record_clone(assume_record(value));
        case TAG_CALLABLE: {
            trilogy_value t;
            trilogy_callable_clone_into(&t, assume_callable(value));
            return t;
        }
        default:
            internal_panic("unreachable");
            return trilogy_undefined;
    }
}

void trilogy_value_clone_into(trilogy_value* into, trilogy_value* from) {
    *into = trilogy_value_clone(from);
}

void trilogy_value_destroy(trilogy_value* value) {
    switch (value->tag) {
        case TAG_STRING: {
            trilogy_string_value* p = assume_string(value);
            trilogy_string_destroy(p);
            free(p);
            break;
        }
        case TAG_BITS: {
            trilogy_bits_value* p = assume_bits(value);
            trilogy_bits_destroy(p);
            break;
        }
        case TAG_TUPLE: {
            trilogy_tuple_value* p = assume_tuple(value);
            trilogy_tuple_destroy(p);
            break;
        }
        case TAG_STRUCT: {
            trilogy_struct_value* p = assume_struct(value);
            trilogy_struct_destroy(p);
            free(p);
            break;
        }
        case TAG_ARRAY: {
            trilogy_array_value* p = assume_array(value);
            trilogy_array_destroy(p);
            free(p);
            break;
        }
        case TAG_SET: {
            trilogy_set_value* p = assume_set(value);
            trilogy_set_destroy(p);
            free(p);
            break;
        }
        case TAG_RECORD: {
            trilogy_record_value* p = assume_record(value);
            trilogy_record_destroy(p);
            free(p);
            break;
        }
        case TAG_CALLABLE: {
            trilogy_callable_value* p = assume_callable(value);
            trilogy_callable_destroy(p);
            free(p);
            break;
        }
        default:
            break;
    }
}

void structural_eq(
    struct trilogy_value* rv,
    struct trilogy_value* lhs,
    struct trilogy_value* rhs
) {
    *rv = trilogy_boolean(is_structural_eq(lhs, rhs));
}

static bool is_referential_eq(
    struct trilogy_value* lhs,
    struct trilogy_value* rhs
) {
    if (lhs->tag != rhs->tag) return false;
    switch (lhs->tag) {
        case TAG_ARRAY:
        case TAG_SET:
        case TAG_RECORD: {
            return lhs->payload == rhs->payload;
        }
        default:
            return is_structural_eq(lhs, rhs);
    }
}

void referential_eq(
    struct trilogy_value* rv,
    struct trilogy_value* lhs,
    struct trilogy_value* rhs
) {
    *rv = trilogy_boolean(is_referential_eq(lhs, rhs));
}
