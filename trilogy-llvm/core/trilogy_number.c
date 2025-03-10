#include "trilogy_number.h"
#include "internal.h"
#include <assert.h>

trilogy_number_value*
trilogy_number_init(trilogy_value* tv, trilogy_number_value* n) {
    assert(tv->tag == TAG_UNDEFINED);
    tv->tag = TAG_NUMBER;
    tv->payload = (unsigned long)n;
    return n;
}

trilogy_number_value* trilogy_number_init_new(
    trilogy_value* tv, bool re_is_negative, size_t re_numer_length,
    unsigned long* re_numer, size_t re_denom_length, unsigned long* re_denom,
    bool im_is_negative, size_t im_numer_length, unsigned long* im_numer,
    size_t im_denom_length, unsigned long* im_denom
) {
    trilogy_number_value* value = malloc_safe(sizeof(trilogy_number_value));
    value->is_negative = re_is_negative;
    value->re = bigint_zero;
    value->im = bigint_zero;
    bigint_init_const(&value->re, re_numer_length, re_numer);
    bigint_init_const(&value->im, im_numer_length, im_numer);
    return trilogy_number_init(tv, value);
}

trilogy_number_value*
trilogy_number_init_bigint(trilogy_value* tv, bool is_negative, bigint* num) {
    trilogy_number_value* value = malloc_safe(sizeof(trilogy_number_value));
    value->is_negative = is_negative;
    value->re = *num;
    value->im = bigint_zero;
    bigint_init_from_u64(&value->im, 0);
    return trilogy_number_init(tv, value);
}

trilogy_number_value*
trilogy_number_init_u64(trilogy_value* tv, unsigned long num) {
    trilogy_number_value* value = malloc_safe(sizeof(trilogy_number_value));
    value->is_negative = false;
    value->re = bigint_zero;
    value->im = bigint_zero;
    bigint_init_from_u64(&value->re, num);
    bigint_init_from_u64(&value->im, 0);
    return trilogy_number_init(tv, value);
}

trilogy_number_value*
trilogy_number_clone_into(trilogy_value* tv, trilogy_number_value* num) {
    trilogy_number_value* clone = malloc_safe(sizeof(trilogy_number_value));
    clone->is_negative = num->is_negative;
    clone->re = bigint_zero;
    clone->im = bigint_zero;
    bigint_clone(&clone->re, &num->re);
    bigint_clone(&clone->im, &num->im);
    return trilogy_number_init(tv, clone);
}

unsigned long trilogy_number_to_u64(trilogy_number_value* val) {
    if (!bigint_is_zero(&val->im))
        internal_panic("expected unsigned long, but number is complex");
    return bigint_to_u64(&val->re);
}

trilogy_number_value* trilogy_number_untag(trilogy_value* val) {
    if (val->tag != TAG_NUMBER) rte("number", val->tag);
    return trilogy_number_assume(val);
}

trilogy_number_value* trilogy_number_assume(trilogy_value* val) {
    assert(val->tag == TAG_NUMBER);
    return (trilogy_number_value*)val->payload;
}

void trilogy_number_destroy(trilogy_number_value* val) {
    bigint_destroy(&val->re);
}

int trilogy_number_compare(
    trilogy_number_value* lhs, trilogy_number_value* rhs
) {
    if (lhs->is_negative == rhs->is_negative) {
        int cmp = bigint_cmp(&lhs->re, &rhs->re);
        return lhs->is_negative ? -cmp : cmp;
    }
    return lhs->is_negative ? -1 : 1;
}

bool trilogy_number_eq(trilogy_number_value* lhs, trilogy_number_value* rhs) {
    if (lhs->is_negative != rhs->is_negative) return false;
    return bigint_eq(&lhs->re, &rhs->re);
}

void trilogy_number_add(trilogy_value* tv, const trilogy_number_value* lhs, const trilogy_number_value* rhs) {
    // TODO: this is intentionally not supporting negative at this time
    trilogy_number_value* lhs_mut = trilogy_number_clone_into(tv, lhs);
    bigint_add(&lhs_mut->re, &rhs->re);
}

void trilogy_number_sub(trilogy_value* tv, const trilogy_number_value* lhs, const trilogy_number_value* rhs) {
    // TODO: this is intentionally not supporting negative at this time
    trilogy_number_value* lhs_mut = trilogy_number_clone_into(tv, lhs);
    bigint_sub(&lhs_mut->re, &rhs->re);
}
