#include "bigint.h"
#include "internal.h"
#include "rational.h"
#include "trilogy_array.h"
#include "trilogy_atom.h"
#include "trilogy_bits.h"
#include "trilogy_boolean.h"
#include "trilogy_character.h"
#include "trilogy_number.h"
#include "trilogy_record.h"
#include "trilogy_set.h"
#include "trilogy_string.h"
#include "trilogy_struct.h"
#include "trilogy_tuple.h"
#include "trilogy_value.h"
#include "types.h"
#include <assert.h>
#include <errno.h>
#include <execinfo.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>

void panic(
    trilogy_value* _rv, // NOLINT(misc-unused-parameters)
    trilogy_value* val
) {
    trilogy_string_value* str = trilogy_string_untag(val);
    char* cstr = malloc_safe(sizeof(char) * (str->len + 2));
    strncpy(cstr, str->contents, str->len);
    cstr[str->len] = '\n';
    cstr[str->len + 1] = '\0';
    trilogy_value_destroy(val);
    internal_panic(cstr);
}

void print(trilogy_value* rv, trilogy_value* val) {
    char* ptr = trilogy_string_as_c(trilogy_string_untag(val));
    printf("%s", ptr);
    free(ptr);
    trilogy_value_destroy(val);
    trilogy_number_init_u64(rv, 0);
}

void readline(trilogy_value* rv) {
    char* lineptr = NULL;
    size_t len = 0;
    ssize_t read = getline(&lineptr, &len, stdin);
    if (read == -1) {
        if (feof(stdin)) {
            trilogy_atom_init(rv, ATOM_EOF);
        } else if (ferror(stdin)) {
            trilogy_number_init_u64(rv, errno);
        }
    } else {
        trilogy_string_init_new(rv, read, lineptr);
    }
    free(lineptr);
}

void readchar(trilogy_value* rv) {
    int ch = getc(stdin);
    if (ch == EOF) {
        if (feof(stdin)) {
            trilogy_atom_init(rv, ATOM_EOF);
        } else if (ferror(stdin)) {
            trilogy_number_init_u64(rv, errno);
        }
    } else {
        trilogy_character_init(rv, ch);
    }
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
    trilogy_value_destroy(lhs);
    trilogy_value_destroy(rhs);
}

void referential_neq(
    trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs
) {
    trilogy_boolean_init(rv, !trilogy_value_referential_eq(lhs, rhs));
    trilogy_value_destroy(lhs);
    trilogy_value_destroy(rhs);
}

void structural_eq(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_boolean_init(rv, trilogy_value_structural_eq(lhs, rhs));
    trilogy_value_destroy(lhs);
    trilogy_value_destroy(rhs);
}

void structural_neq(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_boolean_init(rv, !trilogy_value_structural_eq(lhs, rhs));
    trilogy_value_destroy(lhs);
    trilogy_value_destroy(rhs);
}

void add(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_number_value* lnum = trilogy_number_untag(lhs);
    trilogy_number_value* rnum = trilogy_number_untag(rhs);
    trilogy_number_add(rv, lnum, rnum);
    trilogy_value_destroy(lhs);
    trilogy_value_destroy(rhs);
}

void subtract(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_number_value* lnum = trilogy_number_untag(lhs);
    trilogy_number_value* rnum = trilogy_number_untag(rhs);
    trilogy_number_sub(rv, lnum, rnum);
    trilogy_value_destroy(lhs);
    trilogy_value_destroy(rhs);
}

void multiply(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_number_value* lnum = trilogy_number_untag(lhs);
    trilogy_number_value* rnum = trilogy_number_untag(rhs);
    trilogy_number_mul(rv, lnum, rnum);
    trilogy_value_destroy(lhs);
    trilogy_value_destroy(rhs);
}

void divide(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_number_value* lnum = trilogy_number_untag(lhs);
    trilogy_number_value* rnum = trilogy_number_untag(rhs);
    trilogy_number_div(rv, lnum, rnum);
    trilogy_value_destroy(lhs);
    trilogy_value_destroy(rhs);
}

void int_divide(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_number_value* lnum = trilogy_number_untag(lhs);
    trilogy_number_value* rnum = trilogy_number_untag(rhs);
    trilogy_number_int_div(rv, lnum, rnum);
    trilogy_value_destroy(lhs);
    trilogy_value_destroy(rhs);
}

void rem(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_number_value* lnum = trilogy_number_untag(lhs);
    trilogy_number_value* rnum = trilogy_number_untag(rhs);
    trilogy_number_rem(rv, lnum, rnum);
    trilogy_value_destroy(lhs);
    trilogy_value_destroy(rhs);
}

void power(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_number_value* lnum = trilogy_number_untag(lhs);
    trilogy_number_value* rnum = trilogy_number_untag(rhs);
    trilogy_number_pow(rv, lnum, rnum);
    trilogy_value_destroy(lhs);
    trilogy_value_destroy(rhs);
}

void negate(trilogy_value* rv, trilogy_value* val) {
    trilogy_number_value* num = trilogy_number_untag(val);
    trilogy_number_negate(rv, num);
    trilogy_value_destroy(val);
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
    case TAG_SET:
        trilogy_number_init_u64(rv, trilogy_set_len(trilogy_set_assume(val)));
        break;
    case TAG_RECORD:
        trilogy_number_init_u64(
            rv, trilogy_record_len(trilogy_record_assume(val))
        );
        break;
    default:
        rte("string, bits, array, set, or record", val->tag);
    }
    trilogy_value_destroy(val);
}

void push(trilogy_value* rv, trilogy_value* arr, trilogy_value* val) {
    switch (arr->tag) {
    case TAG_ARRAY: {
        trilogy_array_push(trilogy_array_assume(arr), val);
        break;
    }
    case TAG_SET: {
        trilogy_set_insert(trilogy_set_assume(arr), val);
        break;
    }
    default:
        rte("array or set", arr->tag);
    }
    trilogy_value_destroy(arr);
    *rv = trilogy_unit;
}

void pop(trilogy_value* rv, trilogy_value* arr) {
    trilogy_array_pop(rv, trilogy_array_untag(arr));
    trilogy_value_destroy(arr);
}

void append(trilogy_value* rv, trilogy_value* arr, trilogy_value* val) {
    switch (arr->tag) {
    case TAG_ARRAY: {
        trilogy_array_append(trilogy_array_assume(arr), val);
        break;
    }
    case TAG_SET: {
        trilogy_set_append(trilogy_set_assume(arr), val);
        break;
    }
    case TAG_RECORD: {
        trilogy_record_append(trilogy_record_assume(arr), val);
        break;
    }
    default:
        rte("array, set, or record", arr->tag);
    }
    trilogy_value_destroy(arr);
    *rv = trilogy_unit;
}

void contains_key(trilogy_value* rv, trilogy_value* arr, trilogy_value* val) {
    switch (arr->tag) {
    case TAG_SET: {
        trilogy_boolean_init(
            rv, trilogy_set_contains(trilogy_set_assume(arr), val)
        );
        break;
    }
    case TAG_RECORD: {
        trilogy_boolean_init(
            rv, trilogy_record_contains_key(trilogy_record_assume(arr), val)
        );
        break;
    }
    default:
        rte("set, or record", arr->tag);
    }
    trilogy_value_destroy(arr);
    trilogy_value_destroy(val);
}

void delete_member(trilogy_value* rv, trilogy_value* arr, trilogy_value* val) {
    switch (arr->tag) {
    case TAG_SET: {
        trilogy_boolean_init(
            rv, trilogy_set_delete(trilogy_set_assume(arr), val)
        );
        break;
    }
    case TAG_RECORD: {
        trilogy_boolean_init(
            rv, trilogy_record_delete(trilogy_record_assume(arr), val)
        );
        break;
    }
    default:
        rte("set, or record", arr->tag);
    }
    trilogy_value_destroy(arr);
    trilogy_value_destroy(val);
}

void glue(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_string_value* lstr = trilogy_string_untag(lhs);
    trilogy_string_value* rstr = trilogy_string_untag(rhs);
    trilogy_string_concat(rv, lstr, rstr);
    trilogy_value_destroy(lhs);
    trilogy_value_destroy(rhs);
}

void compare(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    assert(lhs->tag != TAG_UNDEFINED);
    assert(rhs->tag != TAG_UNDEFINED);
    trilogy_atom_make_cmp(rv, trilogy_value_compare(lhs, rhs));
    trilogy_value_destroy(lhs);
    trilogy_value_destroy(rhs);
}

void lt(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_boolean_init(rv, trilogy_value_compare(lhs, rhs) == -1);
    trilogy_value_destroy(lhs);
    trilogy_value_destroy(rhs);
}

void lte(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    int cmp = trilogy_value_compare(lhs, rhs);
    trilogy_boolean_init(rv, cmp == -1 || cmp == 0);
    trilogy_value_destroy(lhs);
    trilogy_value_destroy(rhs);
}

void gt(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_boolean_init(rv, trilogy_value_compare(lhs, rhs) == 1);
    trilogy_value_destroy(lhs);
    trilogy_value_destroy(rhs);
}

void gte(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    int cmp = trilogy_value_compare(lhs, rhs);
    trilogy_boolean_init(rv, cmp == 1 || cmp == 0);
    trilogy_value_destroy(lhs);
    trilogy_value_destroy(rhs);
}

void boolean_not(trilogy_value* rv, trilogy_value* v) {
    trilogy_boolean_not(rv, v);
    trilogy_value_destroy(v);
}

void boolean_and(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_boolean_and(rv, lhs, rhs);
    trilogy_value_destroy(lhs);
    trilogy_value_destroy(rhs);
}

void boolean_or(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_boolean_or(rv, lhs, rhs);
    trilogy_value_destroy(lhs);
    trilogy_value_destroy(rhs);
}

void bitwise_or(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_bits_value* lhs_bits = trilogy_bits_untag(lhs);
    trilogy_bits_value* rhs_bits = trilogy_bits_untag(rhs);
    trilogy_bits_value* out = trilogy_bits_or(lhs_bits, rhs_bits);
    trilogy_bits_init(rv, out);
    trilogy_value_destroy(lhs);
    trilogy_value_destroy(rhs);
}

void bitwise_and(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_bits_value* lhs_bits = trilogy_bits_untag(lhs);
    trilogy_bits_value* rhs_bits = trilogy_bits_untag(rhs);
    trilogy_bits_value* out = trilogy_bits_and(lhs_bits, rhs_bits);
    trilogy_bits_init(rv, out);
    trilogy_value_destroy(lhs);
    trilogy_value_destroy(rhs);
}

void bitwise_xor(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_bits_value* lhs_bits = trilogy_bits_untag(lhs);
    trilogy_bits_value* rhs_bits = trilogy_bits_untag(rhs);
    trilogy_bits_value* out = trilogy_bits_xor(lhs_bits, rhs_bits);
    trilogy_bits_init(rv, out);
    trilogy_value_destroy(lhs);
    trilogy_value_destroy(rhs);
}

void bitwise_invert(trilogy_value* rv, trilogy_value* value) {
    trilogy_bits_value* bits = trilogy_bits_untag(value);
    trilogy_bits_value* inverted = trilogy_bits_invert(bits);
    trilogy_bits_init(rv, inverted);
    trilogy_value_destroy(value);
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
    trilogy_value_destroy(lhs);
    trilogy_value_destroy(rhs);
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
    trilogy_value_destroy(lhs);
    trilogy_value_destroy(rhs);
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
    trilogy_value_destroy(lhs);
    trilogy_value_destroy(rhs);
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
    trilogy_value_destroy(lhs);
    trilogy_value_destroy(rhs);
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
    trilogy_value_destroy(lhs);
    trilogy_value_destroy(rhs);
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
    trilogy_value_destroy(lhs);
    trilogy_value_destroy(rhs);
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
            internal_panic("invalid index for tuple member access");
        }
        break;
    }
    case TAG_ARRAY: {
        trilogy_number_value* number = trilogy_number_untag(index);
        uint64_t i = trilogy_number_to_u64(number);
        trilogy_array_at(rv, trilogy_array_assume(c), i);
        break;
    }
    case TAG_RECORD: {
        trilogy_record_get(rv, trilogy_record_assume(c), index);
        break;
    }
    default:
        rte("string, bits, tuple, array, or record", c->tag);
    }
    trilogy_value_destroy(c);
    trilogy_value_destroy(index);
}

void member_assign(
    trilogy_value* rv, trilogy_value* c, trilogy_value* index,
    trilogy_value* value
) {
    switch (c->tag) {
    case TAG_ARRAY: {
        trilogy_number_value* number = trilogy_number_untag(index);
        uint64_t i = trilogy_number_to_u64(number);
        trilogy_array_set(trilogy_array_assume(c), i, value);
        trilogy_value_destroy(c);
        trilogy_value_destroy(index);
        break;
    }
    case TAG_RECORD: {
        trilogy_record_insert(trilogy_record_assume(c), index, value);
        trilogy_value_destroy(c);
        break;
    }
    default:
        rte("string, bits, tuple, array, or record", c->tag);
    }
    *rv = trilogy_unit;
}

void cons(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_tuple_init_take(rv, lhs, rhs);
}

void primitive_to_string(trilogy_value* rv, trilogy_value* val) {
    trilogy_value_to_string(rv, val);
    trilogy_value_destroy(val);
}

void lookup_atom(trilogy_value* rv, trilogy_value* atom) {
    uint64_t atom_id = trilogy_atom_untag(atom);
    const trilogy_string_value* repr = trilogy_atom_repr(atom_id);
    if (repr != NULL) {
        trilogy_string_clone_into(rv, repr);
    } else {
        *rv = trilogy_unit;
    }
    trilogy_value_destroy(atom);
}

void construct(trilogy_value* rv, trilogy_value* atom, trilogy_value* value) {
    uint64_t atom_id = trilogy_atom_untag(atom);
    trilogy_struct_init_take(rv, atom_id, value);
    trilogy_value_destroy(atom);
}

void destruct(trilogy_value* rv, trilogy_value* val) {
    trilogy_struct_value* s = trilogy_struct_untag(val);
    trilogy_value atom = trilogy_undefined;
    trilogy_atom_init(&atom, s->atom);
    trilogy_tuple_init_new(rv, &atom, &s->contents);
    trilogy_value_destroy(&atom);
    trilogy_value_destroy(val);
}

bool unglue_start(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_string_value* lhs_str = trilogy_string_assume(lhs);
    trilogy_string_value* rhs_str = trilogy_string_assume(rhs);
    bool result = trilogy_string_unglue_start(rv, lhs_str, rhs_str);
    trilogy_value_destroy(lhs);
    trilogy_value_destroy(rhs);
    return result;
}

bool unglue_end(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    trilogy_string_value* lhs_str = trilogy_string_assume(lhs);
    trilogy_string_value* rhs_str = trilogy_string_assume(rhs);
    bool result = trilogy_string_unglue_end(rv, lhs_str, rhs_str);
    trilogy_value_destroy(lhs);
    trilogy_value_destroy(rhs);
    return result;
}

void set_to_array(trilogy_value* rv, trilogy_value* set_val) {
    trilogy_set_value* set = trilogy_set_untag(set_val);
    trilogy_set_to_array(rv, set);
    trilogy_value_destroy(set_val);
}

void record_to_array(trilogy_value* rv, trilogy_value* record_val) {
    trilogy_record_value* record = trilogy_record_untag(record_val);
    trilogy_record_to_array(rv, record);
    trilogy_value_destroy(record_val);
}

void string_to_array(trilogy_value* rv, trilogy_value* string_val) {
    trilogy_string_value* string = trilogy_string_untag(string_val);
    trilogy_string_to_array(rv, string);
    trilogy_value_destroy(string_val);
}

void slice(
    trilogy_value* rv, trilogy_value* val, trilogy_value* start,
    trilogy_value* end
) {
    const size_t start_i =
        (size_t)trilogy_number_to_u64(trilogy_number_untag(start));
    const size_t end_i =
        (size_t)trilogy_number_to_u64(trilogy_number_untag(end));

    trilogy_value_destroy(start);
    trilogy_value_destroy(end);

    switch (val->tag) {
    case TAG_ARRAY:
        trilogy_array_slice(rv, trilogy_array_assume(val), start_i, end_i);
        break;
    case TAG_STRING:
        trilogy_string_slice(rv, trilogy_string_assume(val), start_i, end_i);
        break;
    default:
        rte("string or array", val->tag);
    }
    trilogy_value_destroy(val);
}

void re(trilogy_value* rv, trilogy_value* val) {
    trilogy_number_value* num = trilogy_number_untag(val);
    rational real;
    rational_clone(&real, &num->re);
    rational zero = RATIONAL_ZERO;
    trilogy_number_init_from_re_im(rv, real, zero);
    trilogy_value_destroy(val);
}

void im(trilogy_value* rv, trilogy_value* val) {
    trilogy_number_value* num = trilogy_number_untag(val);
    rational im;
    rational_clone(&im, &num->im);
    rational zero = RATIONAL_ZERO;
    trilogy_number_init_from_re_im(rv, im, zero);
    trilogy_value_destroy(val);
}

void numer(trilogy_value* rv, trilogy_value* val) {
    trilogy_number_value* num = trilogy_number_untag(val);
    rational real = RATIONAL_ONE;
    real.is_negative = num->re.is_negative;
    bigint_clone(&real.numer, &num->re.numer);
    rational zero = RATIONAL_ZERO;
    trilogy_number_init_from_re_im(rv, real, zero);
    trilogy_value_destroy(val);
}

void denom(trilogy_value* rv, trilogy_value* val) {
    trilogy_number_value* num = trilogy_number_untag(val);
    rational real = RATIONAL_ONE;
    bigint_clone(&real.numer, &num->re.denom);
    rational zero = RATIONAL_ZERO;
    trilogy_number_init_from_re_im(rv, real, zero);
    trilogy_value_destroy(val);
}

void pop_count(trilogy_value* rv, trilogy_value* val) {
    trilogy_bits_value* bits = trilogy_bits_untag(val);
    size_t pop = trilogy_bits_pop_count(bits);
    trilogy_number_init_u64(rv, pop);
    trilogy_value_destroy(val);
}

void to_bits(trilogy_value* rv, trilogy_value* val) {
    switch (val->tag) {
    case TAG_NUMBER: {
        trilogy_number_value* num = trilogy_number_assume(val);
        assert(bigint_is_zero(&num->im.numer));
        assert(bigint_is_one(&num->re.denom));
        trilogy_bits_init_from_bigint(rv, &num->re.numer);
        break;
    }
    case TAG_STRING: {
        internal_panic("unimplemented: bits from string\n");
        break;
    }
    default:
        rte("number or string", val->tag);
    }
    trilogy_value_destroy(val);
}
