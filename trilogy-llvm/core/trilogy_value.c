#include "trilogy_value.h"
#include "bigint.h"
#include "hash.h"
#include "internal.h"
#include "trace.h"
#include "trilogy_array.h"
#include "trilogy_atom.h"
#include "trilogy_bits.h"
#include "trilogy_boolean.h"
#include "trilogy_callable.h"
#include "trilogy_character.h"
#include "trilogy_module.h"
#include "trilogy_number.h"
#include "trilogy_record.h"
#include "trilogy_reference.h"
#include "trilogy_set.h"
#include "trilogy_string.h"
#include "trilogy_struct.h"
#include "trilogy_tuple.h"
#include "types.h"
#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

const trilogy_value trilogy_undefined = {.tag = TAG_UNDEFINED, .payload = 0};

const trilogy_value trilogy_unit = {.tag = TAG_UNIT, .payload = 0};

void trilogy_unit_untag(trilogy_value* val) {
    if (val->tag != TAG_UNIT) rte("unit", val->tag);
}

void trilogy_value_clone_into(trilogy_value* into, trilogy_value* from) {
    assert(into != NULL);
    assert(from != NULL);
    assert(into->tag == TAG_UNDEFINED);
    assert(from->tag != TAG_UNDEFINED);
    TRACE("Cloning value    (%2d): %p\n", from->tag, from);
    switch (from->tag) {
    case TAG_UNIT:
    case TAG_BOOL:
    case TAG_ATOM:
    case TAG_CHAR:
        *into = *from;
        break;
    case TAG_NUMBER:
        trilogy_number_clone_into(into, trilogy_number_assume(from));
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
    case TAG_MODULE:
        trilogy_module_clone_into(into, trilogy_module_assume(from));
        break;
    default:
        internal_panic("invalid trilogy value\n");
    }
}

void trilogy_value_destroy(trilogy_value* value) {
    assert(value != NULL);
    TRACE("Destroying value (%2d): %p\n", value->tag, value);
    switch (value->tag) {
    case TAG_NUMBER: {
        trilogy_number_value* p = trilogy_number_assume(value);
        trilogy_number_destroy(p);
        free(p);
        break;
    }
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
    case TAG_MODULE: {
        trilogy_module* p = trilogy_module_assume(value);
        trilogy_module_destroy(p);
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
    case TAG_MODULE:
        return lhs->payload == rhs->payload;
    case TAG_NUMBER: {
        trilogy_number_value* lhs_num = trilogy_number_assume(lhs);
        trilogy_number_value* rhs_num = trilogy_number_assume(rhs);
        return trilogy_number_eq(lhs_num, rhs_num);
    }
    case TAG_CALLABLE: {
        // Closures can only be reference equal, but closure-less functions
        // should be treated all the same no matter how they got cloned up
        trilogy_callable_value* lhs_fn = (trilogy_callable_value*)lhs->payload;
        trilogy_callable_value* rhs_fn = (trilogy_callable_value*)rhs->payload;
        if (lhs_fn->closure == NO_CLOSURE && rhs_fn->closure == NO_CLOSURE) {
            return lhs_fn->function == rhs_fn->function;
        }
        return lhs_fn == rhs_fn;
    }
    case TAG_STRING: {
        trilogy_string_value* lhs_str = trilogy_string_assume(lhs);
        trilogy_string_value* rhs_str = trilogy_string_assume(rhs);
        if (lhs_str->len != rhs_str->len) return false;
        return strncmp(lhs_str->contents, rhs_str->contents, lhs_str->len) == 0;
    }
    case TAG_BITS: {
        trilogy_bits_value* lhs_bits = trilogy_bits_assume(lhs);
        trilogy_bits_value* rhs_bits = trilogy_bits_assume(rhs);
        return trilogy_bits_eq(lhs_bits, rhs_bits);
    }
    case TAG_STRUCT: {
        trilogy_struct_value* lhs_st = trilogy_struct_assume(lhs);
        trilogy_struct_value* rhs_st = trilogy_struct_assume(rhs);
        return lhs_st->atom == rhs_st->atom &&
               trilogy_value_structural_eq(
                   &lhs_st->contents, &rhs_st->contents
               );
        break;
    }
    case TAG_TUPLE: {
        trilogy_tuple_value* lhs_tup = trilogy_tuple_assume(lhs);
        trilogy_tuple_value* rhs_tup = trilogy_tuple_assume(rhs);
        return trilogy_value_structural_eq(&lhs_tup->fst, &rhs_tup->fst) &&
               trilogy_value_structural_eq(&lhs_tup->snd, &rhs_tup->snd);
    }
    case TAG_ARRAY: {
        trilogy_array_value* lhs_arr = trilogy_array_assume(lhs);
        trilogy_array_value* rhs_arr = trilogy_array_assume(rhs);
        if (lhs_arr->len != rhs_arr->len) return false;
        for (uint64_t i = 0; i < lhs_arr->len; ++i) {
            if (!trilogy_value_structural_eq(
                    &lhs_arr->contents[i], &rhs_arr->contents[i]
                )) {
                return false;
            }
        }
        return true;
    }
    case TAG_RECORD: {
        trilogy_record_value* lhs_rec = trilogy_record_assume(lhs);
        trilogy_record_value* rhs_rec = trilogy_record_assume(rhs);
        return trilogy_record_structural_eq(lhs_rec, rhs_rec);
    }
    case TAG_SET: {
        trilogy_set_value* lhs_set = trilogy_set_assume(lhs);
        trilogy_set_value* rhs_set = trilogy_set_assume(rhs);
        return trilogy_set_structural_eq(lhs_set, rhs_set);
    }
    default:
        internal_panic("unreachable\n");
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
    case TAG_RECORD:
    case TAG_MODULE: {
        return lhs->payload == rhs->payload;
    }
    case TAG_CALLABLE: {
        // Closures can only be reference equal, but closure-less functions
        // should be treated all the same no matter how they got cloned up
        trilogy_callable_value* lhs_fn = (trilogy_callable_value*)lhs->payload;
        trilogy_callable_value* rhs_fn = (trilogy_callable_value*)rhs->payload;
        if (lhs_fn->closure == NO_CLOSURE && rhs_fn->closure == NO_CLOSURE) {
            return lhs_fn->function == rhs_fn->function;
        }
        return lhs_fn == rhs_fn;
    }
    default:
        return trilogy_value_structural_eq(lhs, rhs);
    }
}

void trilogy_value_to_string(trilogy_value* rv, trilogy_value* val) {
    assert(val->tag != TAG_UNDEFINED);
    switch (val->tag) {
    case TAG_UNIT:
        trilogy_string_init_new(rv, 4, "unit");
        break;
    case TAG_BOOL:
        if (trilogy_boolean_assume(val)) {
            trilogy_string_init_new(rv, 4, "true");
        } else {
            trilogy_string_init_new(rv, 5, "false");
        }
        break;
    case TAG_ATOM: {
        const trilogy_string_value* repr =
            trilogy_atom_repr(trilogy_atom_assume(val));
        if (repr == NULL) internal_panic("unknown atom\n");
        trilogy_string_clone_into(rv, repr);
        break;
    }
    case TAG_CHAR: {
        char ch = (char)trilogy_character_assume(val);
        trilogy_string_init_new(rv, 1, &ch);
        break;
    }
    case TAG_NUMBER: {
        trilogy_number_value* number = trilogy_number_assume(val);
        char* str = trilogy_number_to_string(number);
        trilogy_string_init_from_c(rv, str);
        free(str);
        break;
    }
    case TAG_STRING:
        trilogy_string_clone_into(rv, trilogy_string_assume(val));
        break;
    case TAG_BITS: {
        trilogy_bits_value* bits = trilogy_bits_assume(val);
        char* buf = malloc_safe(sizeof(char) * bits->len);
        for (uint64_t i = 0; i < bits->len; ++i) {
            buf[i] = trilogy_bits_at(bits, i) ? '1' : '0';
        }
        trilogy_string_init_new(rv, bits->len, buf);
        free(buf);
        break;
    }
    case TAG_STRUCT:
    case TAG_TUPLE:
    case TAG_ARRAY:
    case TAG_SET:
    case TAG_RECORD:
    case TAG_CALLABLE:
    case TAG_REFERENCE:
    case TAG_MODULE:
    default:
        internal_panic("unimplemented\n");
    }
}

int trilogy_value_compare(trilogy_value* lhs, trilogy_value* rhs) {
    if (lhs->tag != rhs->tag) {
        return -2;
    }
    switch (lhs->tag) {
    case TAG_UNDEFINED:
    case TAG_UNIT:
    case TAG_ATOM:
    case TAG_SET:
    case TAG_RECORD:
    case TAG_CALLABLE:
    case TAG_MODULE:
    case TAG_REFERENCE:
        return -2;
    case TAG_BOOL:
        return trilogy_boolean_compare(
            trilogy_boolean_assume(lhs), trilogy_boolean_assume(rhs)
        );
    case TAG_NUMBER:
        return trilogy_number_compare(
            trilogy_number_assume(lhs), trilogy_number_assume(rhs)
        );
    case TAG_CHAR:
        return trilogy_character_compare(
            trilogy_character_assume(lhs), trilogy_character_assume(rhs)
        );
    case TAG_STRING:
        return trilogy_string_compare(
            trilogy_string_assume(lhs), trilogy_string_assume(rhs)
        );
    case TAG_STRUCT:
        return trilogy_struct_compare(
            trilogy_struct_assume(lhs), trilogy_struct_assume(rhs)
        );
    case TAG_BITS:
        return trilogy_bits_compare(
            trilogy_bits_assume(lhs), trilogy_bits_assume(rhs)
        );
    case TAG_TUPLE:
        return trilogy_tuple_compare(
            trilogy_tuple_assume(lhs), trilogy_tuple_assume(rhs)
        );
    case TAG_ARRAY:
        return trilogy_array_compare(
            trilogy_array_assume(lhs), trilogy_array_assume(rhs)
        );
    }
}

static void bigint_hash_into(hasher* h, bigint* b) {
    hash_update_n(h, (uint8_t*)&b->length, sizeof(size_t));
    if (b->capacity == 0) {
        hash_update_n(h, (uint8_t*)&b->contents.value, sizeof(digit_t));
    } else {
        for (size_t i = 0; i < b->length; ++i) {
            hash_update_n(h, (uint8_t*)&b->contents.digits[i], sizeof(digit_t));
        }
    }
}

static void trilogy_value_hash_into(hasher* h, trilogy_value* value) {
    assert(value != NULL);
    assert(value->tag != TAG_UNDEFINED);
    assert(value->tag != TAG_REFERENCE);
    hash_update(h, value->tag);

    switch (value->tag) {
    case TAG_UNIT:
    case TAG_BOOL:
    case TAG_ATOM:
    case TAG_CHAR:
    case TAG_ARRAY:
    case TAG_SET:
    case TAG_RECORD:
    case TAG_MODULE:
    case TAG_CALLABLE: {
        hash_update_n(h, (uint8_t*)&value->payload, sizeof(uint64_t));
        break;
    }
    case TAG_STRING: {
        trilogy_string_value* str = trilogy_string_assume(value);
        hash_update_n(h, (uint8_t*)&str->len, sizeof(size_t));
        hash_update_n(h, (uint8_t*)str->contents, str->len);
        break;
    }
    case TAG_NUMBER: {
        trilogy_number_value* t = trilogy_number_assume(value);
        hash_update(h, (uint8_t)t->re.is_negative);
        bigint_hash_into(h, &t->re.numer);
        bigint_hash_into(h, &t->re.denom);
        hash_update(h, (uint8_t)t->im.is_negative);
        bigint_hash_into(h, &t->im.numer);
        bigint_hash_into(h, &t->im.denom);
        break;
    }
    case TAG_BITS: {
        trilogy_bits_value* bits = trilogy_bits_assume(value);
        size_t byte_len = trilogy_bits_bytelen(bits);
        hash_update_n(h, (uint8_t*)&bits->len, sizeof(size_t));
        hash_update_n(h, bits->contents, byte_len - 1);
        size_t last_len = bits->len % 8;
        uint8_t mask = ~0 >> (8 - last_len) << (8 - last_len);
        uint8_t last = bits->contents[byte_len - 1] & mask;
        hash_update(h, last);
        break;
    }
    case TAG_STRUCT: {
        trilogy_struct_value* st = trilogy_struct_assume(value);
        hash_update_n(h, (uint8_t*)&st->atom, sizeof(uint64_t));
        trilogy_value_hash_into(h, &st->contents);
        break;
    }
    case TAG_TUPLE: {
        trilogy_tuple_value* t = trilogy_tuple_assume(value);
        trilogy_value_hash_into(h, &t->fst);
        trilogy_value_hash_into(h, &t->snd);
        break;
    }
    case TAG_UNDEFINED:
    case TAG_REFERENCE:
        internal_panic("unreachable");
    }
}

uint64_t trilogy_value_hash(trilogy_value* value) {
    hasher* h = hash_new();
    trilogy_value_hash_into(h, value);
    return hash_finish(h);
}
