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

void trilogy_unit_untag(trilogy_value* val) {
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
        case TAG_STRING: {
            trilogy_value t;
            trilogy_string_clone_into(&t, trilogy_string_assume(value));
            return t;
        }
        case TAG_BITS: {
            trilogy_value t;
            trilogy_bits_clone_into(&t, trilogy_bits_assume(value));
            return t;
        }
        case TAG_STRUCT: {
            trilogy_value t;
            trilogy_struct_clone_into(&t, trilogy_struct_assume(value));
            return t;
        }
        case TAG_TUPLE: {
            trilogy_value t;
            trilogy_tuple_clone_into(&t, trilogy_tuple_assume(value));
            return t;
        }
        case TAG_ARRAY: {
            trilogy_value t;
            trilogy_array_clone_into(&t, trilogy_array_assume(value));
            return t;
        }
        case TAG_SET: {
            trilogy_value t;
            trilogy_set_clone_into(&t, trilogy_set_assume(value));
            return t;
        }
        case TAG_RECORD:
            return trilogy_record_clone(trilogy_record_assume(value));
        case TAG_CALLABLE: {
            trilogy_value t;
            trilogy_callable_clone_into(&t, trilogy_callable_assume(value));
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
            trilogy_string_value* p = trilogy_string_assume(value);
            trilogy_string_destroy(p);
            free(p);
            break;
        }
        case TAG_BITS: {
            trilogy_bits_value* p = trilogy_bits_assume(value);
            trilogy_bits_destroy(p);
            break;
        }
        case TAG_TUPLE: {
            trilogy_tuple_value* p = trilogy_tuple_assume(value);
            trilogy_tuple_destroy(p);
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
            free(p);
            break;
        }
        case TAG_SET: {
            trilogy_set_value* p = trilogy_set_assume(value);
            trilogy_set_destroy(p);
            free(p);
            break;
        }
        case TAG_RECORD: {
            trilogy_record_value* p = trilogy_record_assume(value);
            trilogy_record_destroy(p);
            free(p);
            break;
        }
        case TAG_CALLABLE: {
            trilogy_callable_value* p = trilogy_callable_assume(value);
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
