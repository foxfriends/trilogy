#include "types.h"
#include "internal.h"

char* type_name(trilogy_value_tag tag) {
    switch (tag) {
    case TAG_UNDEFINED:
        return "undefined";
    case TAG_UNIT:
        return "unit";
    case TAG_BOOL:
        return "boolean";
    case TAG_ATOM:
        return "atom";
    case TAG_CHAR:
        return "character";
    case TAG_STRING:
        return "string";
    case TAG_NUMBER:
        return "number";
    case TAG_BITS:
        return "bits";
    case TAG_STRUCT:
        return "struct";
    case TAG_TUPLE:
        return "tuple";
    case TAG_ARRAY:
        return "array";
    case TAG_SET:
        return "set";
    case TAG_RECORD:
        return "record";
    case TAG_CALLABLE:
        return "callable";
    default:
        internal_panic("runtime error: invalid trilogy_value tag\n");
        return "undefined";
    }
}
