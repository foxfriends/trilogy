#include <stdlib.h>
#include "trilogy_value.h"
#include "trilogy_array.h"
#include "trilogy_string.h"
#include "trilogy_bits.h"
#include "trilogy_tuple.h"
#include "trilogy_struct.h"
#include "trilogy_array.h"
#include "trilogy_set.h"
#include "trilogy_record.h"
#include "trilogy_callable.h"
#include "internal.h"

void destroy_trilogy_value(trilogy_value* value) {
    switch (value->tag) {
        case TAG_STRING: {
            trilogy_string_value* p = assume_string(value);
            destroy_string(p);
            free(p);
            break;
        }
        case TAG_BITS: {
            trilogy_bits_value* p = assume_bits(value);
            destroy_bits(p);
            free(p);
            break;
        }
        case TAG_TUPLE: {
            trilogy_tuple_value* p = assume_tuple(value);
            destroy_tuple(p);
            free(p);
            break;
        }
        case TAG_STRUCT: {
            trilogy_struct_value* p = assume_struct(value);
            destroy_struct(p);
            free(p);
            break;
        }
        case TAG_ARRAY: {
            trilogy_array_value* p = assume_array(value);
            destroy_array(p);
            free(p);
            break;
        }
        case TAG_SET: {
            trilogy_set_value* p = assume_set(value);
            destroy_set(p);
            free(p);
            break;
        }
        case TAG_RECORD: {
            trilogy_record_value* p = assume_record(value);
            destroy_record(p);
            free(p);
            break;
        }
        case TAG_CALLABLE: {
            trilogy_callable_value* p = assume_callable(value);
            destroy_callable(p);
            free(p);
            break;
        }
        default:
            break;
    }
}

const trilogy_value trilogy_undefined = { .tag = TAG_UNDEFINED, .payload = 0 };

const trilogy_value trilogy_unit = { .tag = TAG_UNIT, .payload = 0 };

void untag_unit(trilogy_value* val) {
    if (val->tag != TAG_UNIT) rte("unit", val->tag);
}
