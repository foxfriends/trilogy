#include "trilogy_number.h"
#include "internal.h"
#include <assert.h>
#include <stdlib.h>

trilogy_number_value*
trilogy_number_init(trilogy_value* tv, trilogy_number_value* n) {
    assert(tv->tag == TAG_UNDEFINED);
    tv->tag = TAG_NUMBER;
    tv->payload = (uint64_t)n;
    return n;
}

trilogy_number_value* trilogy_number_init_new(
    trilogy_value* tv, bool re_is_negative, size_t re_numer_length,
    digit_t* re_numer, size_t re_denom_length, digit_t* re_denom,
    bool im_is_negative, size_t im_numer_length, digit_t* im_numer,
    size_t im_denom_length, digit_t* im_denom
) {
    trilogy_number_value* value = malloc_safe(sizeof(trilogy_number_value));
    value->is_negative = re_is_negative;
    value->re_numer = bigint_zero;
    value->re_denom = bigint_one;
    value->im_numer = bigint_zero;
    value->im_denom = bigint_one;
    bigint_init_const(&value->re_numer, re_numer_length, re_numer);
    bigint_init_const(&value->re_denom, re_denom_length, re_denom);
    bigint_init_const(&value->im_numer, im_numer_length, im_numer);
    bigint_init_const(&value->im_denom, im_denom_length, im_denom);
    return trilogy_number_init(tv, value);
}

trilogy_number_value*
trilogy_number_init_bigint(trilogy_value* tv, bool is_negative, bigint* num) {
    trilogy_number_value* value = malloc_safe(sizeof(trilogy_number_value));
    value->is_negative = is_negative;
    value->re_numer = *num;
    value->re_denom = bigint_one;
    value->im_numer = bigint_zero;
    value->im_denom = bigint_one;
    bigint_init_from_u64(&value->im_numer, 0);
    return trilogy_number_init(tv, value);
}

trilogy_number_value* trilogy_number_init_u64(trilogy_value* tv, uint64_t num) {
    trilogy_number_value* value = malloc_safe(sizeof(trilogy_number_value));
    value->is_negative = false;
    value->re_numer = bigint_zero;
    value->re_denom = bigint_one;
    value->im_numer = bigint_zero;
    value->im_denom = bigint_one;
    bigint_init_from_u64(&value->re_numer, num);
    return trilogy_number_init(tv, value);
}

trilogy_number_value*
trilogy_number_clone_into(trilogy_value* tv, const trilogy_number_value* num) {
    trilogy_number_value* clone = malloc_safe(sizeof(trilogy_number_value));
    clone->is_negative = num->is_negative;
    bigint_clone(&clone->re_numer, &num->re_numer);
    bigint_clone(&clone->re_denom, &num->re_denom);
    bigint_clone(&clone->im_numer, &num->im_numer);
    bigint_clone(&clone->im_denom, &num->im_denom);
    return trilogy_number_init(tv, clone);
}

uint64_t trilogy_number_to_u64(trilogy_number_value* val) {
    if (!bigint_is_zero(&val->im_numer))
        internal_panic("expected uint64_t, but number is complex");
    return bigint_to_u64(&val->re_numer);
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
    bigint_destroy(&val->re_numer);
}

int trilogy_number_compare(
    trilogy_number_value* lhs, trilogy_number_value* rhs
) {
    if (lhs->is_negative == rhs->is_negative) {
        int cmp = bigint_cmp(&lhs->re_numer, &rhs->re_numer);
        return lhs->is_negative ? -cmp : cmp;
    }
    return lhs->is_negative ? -1 : 1;
}

bool trilogy_number_eq(trilogy_number_value* lhs, trilogy_number_value* rhs) {
    if (lhs->is_negative != rhs->is_negative) return false;
    return bigint_eq(&lhs->re_numer, &rhs->re_numer) &&
           bigint_eq(&lhs->re_denom, &rhs->re_denom) &&
           bigint_eq(&lhs->im_numer, &rhs->im_numer) &&
           bigint_eq(&lhs->im_denom, &rhs->im_denom);
}

static void rat_reduce(bigint* numer, bigint* denom) {
    if (!bigint_is_one(denom)) {
        bigint* gcd = bigint_gcd(numer, denom);
        bigint_div(numer, gcd);
        bigint_div(denom, gcd);
        bigint_destroy(gcd);
        free(gcd);
    }
}

static void trilogy_number_reduce(trilogy_number_value* val) {
    rat_reduce(&val->re_numer, &val->re_denom);
    rat_reduce(&val->im_numer, &val->im_denom);
}

static void rat_add(
    bigint* lhs_numer, bigint* lhs_denom, const bigint* rhs_numer,
    const bigint* rhs_denom
) {
    if (bigint_eq(lhs_denom, rhs_denom)) {
        bigint_add(lhs_numer, rhs_numer);
        return;
    }
    bigint* gcd = bigint_gcd(lhs_denom, rhs_denom);
    bigint rhs_fac;
    bigint_clone(&rhs_fac, rhs_denom);
    bigint_div(&rhs_fac, gcd);
    bigint lhs_fac;
    bigint_clone(&lhs_fac, lhs_denom);
    bigint_div(&lhs_fac, gcd);
    bigint_destroy(gcd);
    free(gcd);

    // Get LHS up to LCM
    bigint_mul(lhs_numer, &rhs_fac);
    bigint_mul(lhs_denom, &rhs_fac);
    // Get RHS numer to LCM
    bigint_mul(&lhs_fac, rhs_numer);
    // Do the add of numerators
    bigint_add(lhs_numer, &lhs_fac);

    bigint_destroy(&lhs_fac);
    bigint_destroy(&rhs_fac);
}

void trilogy_number_add(
    trilogy_value* tv, const trilogy_number_value* lhs,
    const trilogy_number_value* rhs
) {
    trilogy_number_value* lhs_mut = trilogy_number_clone_into(tv, lhs);
    rat_add(
        &lhs_mut->re_numer, &lhs_mut->re_denom, &rhs->re_numer, &rhs->re_denom
    );
    rat_add(
        &lhs_mut->im_numer, &lhs_mut->im_denom, &rhs->im_numer, &rhs->im_denom
    );
    trilogy_number_reduce(lhs_mut);
}

void trilogy_number_sub(
    trilogy_value* tv, const trilogy_number_value* lhs,
    const trilogy_number_value* rhs
) {
    // TODO: this is intentionally not supporting negative at this time
    trilogy_number_value* lhs_mut = trilogy_number_clone_into(tv, lhs);
    bigint_sub(&lhs_mut->re_numer, &rhs->re_numer);
    bigint_sub(&lhs_mut->im_numer, &rhs->im_numer);
}

void trilogy_number_mul(
    trilogy_value* tv, const trilogy_number_value* lhs,
    const trilogy_number_value* rhs
) {
    // TODO: this is intentionally not supporting negative at this time
    trilogy_number_value* lhs_mut = trilogy_number_clone_into(tv, lhs);
    bigint_mul(&lhs_mut->re_numer, &rhs->re_numer);
}

void trilogy_number_div(
    trilogy_value* tv, const trilogy_number_value* lhs,
    const trilogy_number_value* rhs
) {
    // TODO: this is intentionally not supporting negative at this time
    trilogy_number_value* lhs_mut = trilogy_number_clone_into(tv, lhs);
    bigint_div(&lhs_mut->re_numer, &rhs->re_numer);
}

void trilogy_number_rem(
    trilogy_value* tv, const trilogy_number_value* lhs,
    const trilogy_number_value* rhs
) {
    // TODO: this is intentionally not supporting negative at this time
    trilogy_number_value* lhs_mut = trilogy_number_clone_into(tv, lhs);
    bigint_rem(&lhs_mut->re_numer, &rhs->re_numer);
}

char* trilogy_number_to_string(const trilogy_number_value* lhs) {
    // TODO: this is intentionally not supporting negative at this time
    return bigint_to_string(&lhs->re_numer);
}
