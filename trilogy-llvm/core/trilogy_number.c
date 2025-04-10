#include "trilogy_number.h"
#include "internal.h"
#include "rational.h"
#include <assert.h>

trilogy_number_value*
trilogy_number_init(trilogy_value* tv, trilogy_number_value* n) {
    assert(tv->tag == TAG_UNDEFINED);
    tv->tag = TAG_NUMBER;
    tv->payload = (uint64_t)n;
    return n;
}

trilogy_number_value* trilogy_number_init_const(
    trilogy_value* tv, bool re_is_negative, size_t re_numer_length,
    digit_t* re_numer, size_t re_denom_length, digit_t* re_denom,
    bool im_is_negative, size_t im_numer_length, digit_t* im_numer,
    size_t im_denom_length, digit_t* im_denom
) {
    trilogy_number_value* value = malloc_safe(sizeof(trilogy_number_value));
    rational_init_const(
        &value->re, re_is_negative, re_numer_length, re_numer, re_denom_length,
        re_denom
    );
    rational_init_const(
        &value->im, im_is_negative, im_numer_length, im_numer, im_denom_length,
        im_denom
    );
    return trilogy_number_init(tv, value);
}

trilogy_number_value* trilogy_number_init_u64(trilogy_value* tv, uint64_t num) {
    trilogy_number_value* value = malloc_safe(sizeof(trilogy_number_value));
    value->re = rational_zero;
    value->im = rational_zero;
    bigint_init_from_u64(&value->re.numer, num);
    return trilogy_number_init(tv, value);
}

trilogy_number_value*
trilogy_number_clone_into(trilogy_value* tv, const trilogy_number_value* num) {
    trilogy_number_value* clone = malloc_safe(sizeof(trilogy_number_value));
    rational_clone(&clone->re, &num->re);
    rational_clone(&clone->im, &num->im);
    return trilogy_number_init(tv, clone);
}

uint64_t trilogy_number_to_u64(trilogy_number_value* val) {
    if (!rational_is_zero(&val->im))
        internal_panic("expected uint64_t, but number is complex");
    if (!rational_is_whole(&val->re))
        internal_panic("expected uint64_t, but number is fractional");
    return bigint_to_u64(&val->re.numer);
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
    rational_destroy(&val->re);
    rational_destroy(&val->im);
}

int trilogy_number_compare(
    trilogy_number_value* lhs, trilogy_number_value* rhs
) {
    // TODO: handle `im` here
    return rational_cmp(&lhs->re, &rhs->re);
}

bool trilogy_number_eq(trilogy_number_value* lhs, trilogy_number_value* rhs) {
    return rational_eq(&lhs->re, &rhs->re) && rational_eq(&lhs->im, &rhs->im);
}

void trilogy_number_add(
    trilogy_value* tv, const trilogy_number_value* lhs,
    const trilogy_number_value* rhs
) {
    trilogy_number_value* lhs_mut = trilogy_number_clone_into(tv, lhs);
    rational_add(&lhs_mut->re, &rhs->re);
    rational_add(&lhs_mut->im, &rhs->im);
}

void trilogy_number_sub(
    trilogy_value* tv, const trilogy_number_value* lhs,
    const trilogy_number_value* rhs
) {
    trilogy_number_value* lhs_mut = trilogy_number_clone_into(tv, lhs);
    rational_sub(&lhs_mut->re, &rhs->re);
    rational_sub(&lhs_mut->im, &rhs->im);
}

void trilogy_number_mul(
    trilogy_value* tv, const trilogy_number_value* lhs,
    const trilogy_number_value* rhs
) {
    // TODO: this is intentionally not supporting complex at this time
    trilogy_number_value* lhs_mut = trilogy_number_clone_into(tv, lhs);
    rational_mul(&lhs_mut->re, &rhs->re);
}

void trilogy_number_div(
    trilogy_value* tv, const trilogy_number_value* lhs,
    const trilogy_number_value* rhs
) {
    // TODO: this is intentionally not supporting complex
    trilogy_number_value* lhs_mut = trilogy_number_clone_into(tv, lhs);
    rational_div(&lhs_mut->re, &rhs->re);
}

void trilogy_number_rem(
    trilogy_value* tv, const trilogy_number_value* lhs,
    const trilogy_number_value* rhs
) {
    // TODO: this is intentionally not supporting complex at this time
    trilogy_number_value* lhs_mut = trilogy_number_clone_into(tv, lhs);
    rational_rem(&lhs_mut->re, &rhs->re);
}

void trilogy_number_negate(trilogy_value* tv, const trilogy_number_value* val) {
    trilogy_number_value* lhs_mut = trilogy_number_clone_into(tv, val);
    rational_negate(&lhs_mut->re);
    rational_negate(&lhs_mut->im);
}

char* trilogy_number_to_string(const trilogy_number_value* lhs) {
    // TODO: this is intentionally not supporting complex at this time
    return rational_to_string(&lhs->re);
}
