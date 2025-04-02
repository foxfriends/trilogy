#include "internal.h"
#include "trilogy_array.h"
#include "trilogy_atom.h"
#include "trilogy_bits.h"
#include "trilogy_boolean.h"
#include "trilogy_character.h"
#include "trilogy_number.h"
#include "trilogy_string.h"
#include "trilogy_struct.h"
#include "trilogy_tuple.h"
#include "trilogy_value.h"
#include "types.h"
#include <assert.h>
#include <execinfo.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void panic(trilogy_value* rv, trilogy_value* val) {
    trilogy_string_value* str = trilogy_string_untag(val);
    char* cstr = malloc_safe(sizeof(char) * (str->len + 2));
    strncpy(cstr, str->contents, str->len);
    cstr[str->len] = '\n';
    cstr[str->len + 1] = '\0';
    internal_panic(cstr);
}

void print(trilogy_value* rv, trilogy_value* val) {
    char* ptr = trilogy_string_as_c(trilogy_string_untag(val));
    printf("%s", ptr);
    free(ptr);
    trilogy_number_init_u64(rv, 0);
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

void referential_neq(
    trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs
) {
    trilogy_boolean_init(rv, !trilogy_value_referential_eq(lhs, rhs));
}

void structural_eq(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_boolean_init(rv, trilogy_value_structural_eq(lhs, rhs));
}

void structural_neq(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_boolean_init(rv, !trilogy_value_structural_eq(lhs, rhs));
}

void add(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_number_value* lnum = trilogy_number_untag(lhs);
    trilogy_number_value* rnum = trilogy_number_untag(rhs);
    trilogy_number_add(rv, lnum, rnum);
}

void subtract(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_number_value* lnum = trilogy_number_untag(lhs);
    trilogy_number_value* rnum = trilogy_number_untag(rhs);
    trilogy_number_sub(rv, lnum, rnum);
}

void multiply(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_number_value* lnum = trilogy_number_untag(lhs);
    trilogy_number_value* rnum = trilogy_number_untag(rhs);
    trilogy_number_mul(rv, lnum, rnum);
}

void divide(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_number_value* lnum = trilogy_number_untag(lhs);
    trilogy_number_value* rnum = trilogy_number_untag(rhs);
    trilogy_number_div(rv, lnum, rnum);
}

void length(trilogy_value* rv, trilogy_value* val) {
    switch (val->tag) {
    case TAG_STRING:
        trilogy_number_init_u64(
            rv, trilogy_string_len(trilogy_string_assume(val))
        );
        break;
    case TAG_BITS:
        trilogy_number_init_u64(rv, trilogy_bits_len(trilogy_bits_assume(val)));
        break;
    case TAG_ARRAY:
        trilogy_number_init_u64(
            rv, trilogy_array_len(trilogy_array_assume(val))
        );
        break;
    default:
        rte("string, bits, array, set, or record", val->tag);
    }
}

void push(trilogy_value* rv, trilogy_value* arr, trilogy_value* val) {
    switch (arr->tag) {
    case TAG_ARRAY: {
        trilogy_value pushing = trilogy_undefined;
        trilogy_value_clone_into(&pushing, val);
        trilogy_array_push(trilogy_array_assume(arr), &pushing);
        break;
    }
    default:
        rte("array, set, or record", arr->tag);
    }
    *rv = trilogy_unit;
}

void append(trilogy_value* rv, trilogy_value* arr, trilogy_value* val) {
    switch (arr->tag) {
    case TAG_ARRAY: {
        trilogy_value appending = trilogy_undefined;
        trilogy_value_clone_into(&appending, val);
        trilogy_array_append(trilogy_array_assume(arr), &appending);
        break;
    }
    default:
        rte("array, set, or record", arr->tag);
    }
    *rv = trilogy_unit;
}

void glue(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_string_value* lstr = trilogy_string_untag(lhs);
    trilogy_string_value* rstr = trilogy_string_untag(rhs);
    trilogy_string_concat(rv, lstr, rstr);
}

void compare(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    assert(lhs->tag != TAG_UNDEFINED);
    assert(rhs->tag != TAG_UNDEFINED);
    trilogy_atom_make_cmp(rv, trilogy_value_compare(lhs, rhs));
}

void lt(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_boolean_init(rv, trilogy_value_compare(lhs, rhs) == -1);
}

void lte(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    int cmp = trilogy_value_compare(lhs, rhs);
    trilogy_boolean_init(rv, cmp == -1 || cmp == 0);
}

void gt(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_boolean_init(rv, trilogy_value_compare(lhs, rhs) == 1);
}

void gte(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    int cmp = trilogy_value_compare(lhs, rhs);
    trilogy_boolean_init(rv, cmp == 1 || cmp == 0);
}

void boolean_not(trilogy_value* rv, trilogy_value* v) {
    trilogy_boolean_not(rv, v);
}
void boolean_and(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_boolean_and(rv, lhs, rhs);
}
void boolean_or(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_boolean_or(rv, lhs, rhs);
}

void bitwise_or(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_bits_value* lhs_bits = trilogy_bits_untag(lhs);
    trilogy_bits_value* rhs_bits = trilogy_bits_untag(rhs);
    trilogy_bits_value* out = trilogy_bits_or(lhs_bits, rhs_bits);
    trilogy_bits_init(rv, out);
}

void bitwise_and(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_bits_value* lhs_bits = trilogy_bits_untag(lhs);
    trilogy_bits_value* rhs_bits = trilogy_bits_untag(rhs);
    trilogy_bits_value* out = trilogy_bits_and(lhs_bits, rhs_bits);
    trilogy_bits_init(rv, out);
}

void bitwise_xor(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_bits_value* lhs_bits = trilogy_bits_untag(lhs);
    trilogy_bits_value* rhs_bits = trilogy_bits_untag(rhs);
    trilogy_bits_value* out = trilogy_bits_xor(lhs_bits, rhs_bits);
    trilogy_bits_init(rv, out);
}

void shift_left(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_bits_value* lhs_bits = trilogy_bits_untag(lhs);
    trilogy_number_value* rhs_num = trilogy_number_untag(rhs);
    size_t n = (size_t)trilogy_number_to_u64(rhs_num);
    if (n == 0) {
        trilogy_bits_clone_into(rv, lhs_bits);
        return;
    }
    trilogy_bits_value* out = trilogy_bits_shift_left(lhs_bits, n);
    trilogy_bits_init(rv, out);
}

void shift_left_extend(
    trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs
) {
    trilogy_bits_value* lhs_bits = trilogy_bits_untag(lhs);
    trilogy_number_value* rhs_num = trilogy_number_untag(rhs);
    size_t n = (size_t)trilogy_number_to_u64(rhs_num);
    if (n == 0) {
        trilogy_bits_clone_into(rv, lhs_bits);
        return;
    }
    trilogy_bits_value* out = trilogy_bits_shift_left_extend(lhs_bits, n);
    trilogy_bits_init(rv, out);
}

void shift_left_contract(
    trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs
) {
    trilogy_bits_value* lhs_bits = trilogy_bits_untag(lhs);
    trilogy_number_value* rhs_num = trilogy_number_untag(rhs);
    size_t n = (size_t)trilogy_number_to_u64(rhs_num);
    if (n == 0) {
        trilogy_bits_clone_into(rv, lhs_bits);
        return;
    }
    trilogy_bits_value* out = trilogy_bits_shift_left_contract(lhs_bits, n);
    trilogy_bits_init(rv, out);
}

void shift_right(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_bits_value* lhs_bits = trilogy_bits_untag(lhs);
    trilogy_number_value* rhs_num = trilogy_number_untag(rhs);
    size_t n = (size_t)trilogy_number_to_u64(rhs_num);
    if (n == 0) {
        trilogy_bits_clone_into(rv, lhs_bits);
        return;
    }
    trilogy_bits_value* out = trilogy_bits_shift_right(lhs_bits, n);
    trilogy_bits_init(rv, out);
}

void shift_right_extend(
    trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs
) {
    trilogy_bits_value* lhs_bits = trilogy_bits_untag(lhs);
    trilogy_number_value* rhs_num = trilogy_number_untag(rhs);
    size_t n = (size_t)trilogy_number_to_u64(rhs_num);
    if (n == 0) {
        trilogy_bits_clone_into(rv, lhs_bits);
        return;
    }
    trilogy_bits_value* out = trilogy_bits_shift_right_extend(lhs_bits, n);
    trilogy_bits_init(rv, out);
}

void shift_right_contract(
    trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs
) {
    trilogy_bits_value* lhs_bits = trilogy_bits_untag(lhs);
    trilogy_number_value* rhs_num = trilogy_number_untag(rhs);
    size_t n = (size_t)trilogy_number_to_u64(rhs_num);
    if (n == 0) {
        trilogy_bits_clone_into(rv, lhs_bits);
        return;
    }
    trilogy_bits_value* out = trilogy_bits_shift_right_contract(lhs_bits, n);
    trilogy_bits_init(rv, out);
}

void member_access(trilogy_value* rv, trilogy_value* c, trilogy_value* index) {
    switch (c->tag) {
    case TAG_STRING: {
        trilogy_number_value* number = trilogy_number_untag(index);
        uint64_t i = trilogy_number_to_u64(number);
        uint32_t ch = trilogy_string_at(trilogy_string_assume(c), i);
        trilogy_character_init(rv, ch);
        break;
    }
    case TAG_BITS: {
        trilogy_number_value* number = trilogy_number_untag(index);
        uint64_t i = trilogy_number_to_u64(number);
        bool b = trilogy_bits_at(trilogy_bits_assume(c), i);
        trilogy_boolean_init(rv, b);
        break;
    }
    case TAG_TUPLE: {
        uint64_t i = trilogy_atom_untag(index);
        switch (i) {
        case ATOM_LEFT:
            trilogy_tuple_left(rv, trilogy_tuple_assume(c));
            break;
        case ATOM_RIGHT:
            trilogy_tuple_right(rv, trilogy_tuple_assume(c));
            break;
        default:
            internal_panic("unimplemented: yield 'MIA\n");
        }
        break;
    }
    case TAG_ARRAY: {
        trilogy_number_value* number = trilogy_number_untag(index);
        uint64_t i = trilogy_number_to_u64(number);
        trilogy_array_at(rv, trilogy_array_assume(c), i);
        break;
    }
    default:
        rte("string, bits, tuple, array, or record", c->tag);
    }
}

void cons(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_value lclone = trilogy_undefined;
    trilogy_value rclone = trilogy_undefined;
    trilogy_value_clone_into(&lclone, lhs);
    trilogy_value_clone_into(&rclone, lhs);
    trilogy_tuple_init_new(rv, &lclone, &rclone);
}

void primitive_to_string(trilogy_value* rv, trilogy_value* val) {
    trilogy_value_to_string(rv, val);
}

void lookup_atom(trilogy_value* rv, trilogy_value* atom) {
    uint64_t atom_id = trilogy_atom_untag(atom);
    const trilogy_string_value* repr = trilogy_atom_repr(atom_id);
    if (repr != NULL) {
        trilogy_string_clone_into(rv, repr);
    } else {
        *rv = trilogy_unit;
    }
}

void construct(trilogy_value* rv, trilogy_value* atom, trilogy_value* value) {
    uint64_t atom_id = trilogy_atom_untag(atom);
    trilogy_value value_clone = trilogy_undefined;
    trilogy_value_clone_into(&value_clone, value);
    trilogy_struct_init_new(rv, atom_id, &value_clone);
}

void destruct(trilogy_value* rv, trilogy_value* val) {
    trilogy_struct_value* s = trilogy_struct_untag(val);
    trilogy_value atom = trilogy_undefined;
    trilogy_value contents = trilogy_undefined;
    trilogy_atom_init(&atom, s->atom);
    trilogy_value_clone_into(&contents, &s->contents);
    trilogy_tuple_init_new(rv, &atom, &contents);
}
