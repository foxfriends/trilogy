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
#include <execinfo.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void panic(trilogy_value* rv, trilogy_value* val) {
    trilogy_string_value* str = trilogy_string_untag(val);
    char* cstr = malloc_safe(sizeof(char) * (str->len + 2));
    strncpy(cstr, str->contents, str->len);
    cstr[str->len] = '\n';
    cstr[str->len+1] = '\0';
    internal_panic(cstr);
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
    case TAG_TUPLE: {
        unsigned long i = trilogy_atom_untag(index);
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
        unsigned long i = trilogy_number_untag(index);
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
    unsigned long atom_id = trilogy_atom_untag(atom);
    const trilogy_string_value* repr = trilogy_atom_repr(atom_id);
    if (repr != NULL) {
        trilogy_string_clone_into(rv, repr);
    } else {
        *rv = trilogy_unit;
    }
}

void construct(trilogy_value* rv, trilogy_value* atom, trilogy_value* value) {
    unsigned long atom_id = trilogy_atom_untag(atom);
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
