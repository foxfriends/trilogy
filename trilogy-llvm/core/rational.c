#include "rational.h"
#include <stdlib.h>

const rational rational_zero = {
    .is_negative = false,
    .numer = {.capacity = 0, .length = 1, .contents = {.value = 0}},
    .denom = {.capacity = 0, .length = 1, .contents = {.value = 1}},
};

void rational_init_new(
    rational* rat, bool is_negative, size_t numer_length, const digit_t* numer,
    size_t denom_length, const digit_t* denom
) {
    rat->is_negative = is_negative;
    bigint_init_const(&rat->numer, numer_length, numer);
    bigint_init_const(&rat->denom, denom_length, denom);
}

void rational_clone(rational* into, const rational* from) {
    into->is_negative = from->is_negative;
    bigint_clone(&into->numer, &from->numer);
    bigint_clone(&into->denom, &from->denom);
}

void rational_destroy(rational* val) {
    bigint_destroy(&val->numer);
    bigint_destroy(&val->denom);
}

bool rational_is_zero(const rational* val) {
    return bigint_is_zero(&val->numer);
}

bool rational_is_whole(const rational* val) {
    return bigint_is_one(&val->denom);
}

int rational_cmp(const rational* lhs, const rational* rhs) {
    if (lhs->is_negative == rhs->is_negative) {
        int cmp = 0;
        if (bigint_eq(&lhs->denom, &rhs->denom)) {
            cmp = bigint_cmp(&lhs->numer, &rhs->numer);
        } else {
            bigint lval;
            bigint rval;
            bigint_clone(&lval, &lhs->numer);
            bigint_mul(&lval, &rhs->denom);
            bigint_clone(&rval, &rhs->numer);
            bigint_mul(&rval, &lhs->denom);
            cmp = bigint_cmp(&lval, &rval);
            bigint_destroy(&lval);
            bigint_destroy(&rval);
        }
        return lhs->is_negative ? -cmp : cmp;
    }
    return lhs->is_negative ? -1 : 1;
}

bool rational_eq(const rational* lhs, const rational* rhs) {
    return lhs->is_negative == rhs->is_negative &&
           bigint_eq(&lhs->numer, &rhs->numer) &&
           bigint_eq(&lhs->denom, &rhs->denom);
}

void rational_reduce(rational* val) {
    if (!bigint_is_one(&val->denom)) {
        bigint* gcd = bigint_gcd(&val->numer, &val->denom);
        bigint_div(&val->numer, gcd);
        bigint_div(&val->denom, gcd);
        bigint_destroy(gcd);
        free(gcd);
    }
}

void rational_add(rational* lhs, const rational* rhs) {
    if (bigint_eq(&lhs->denom, &rhs->denom)) {
        bigint_add(&lhs->numer, &rhs->numer);
        return;
    }
    bigint* gcd = bigint_gcd(&lhs->denom, &rhs->denom);
    bigint rhs_fac;
    bigint_clone(&rhs_fac, &rhs->denom);
    bigint_div(&rhs_fac, gcd);
    bigint lhs_fac;
    bigint_clone(&lhs_fac, &lhs->denom);
    bigint_div(&lhs_fac, gcd);
    bigint_destroy(gcd);
    free(gcd);

    // Get LHS up to LCM
    bigint_mul(&lhs->numer, &rhs_fac);
    bigint_mul(&lhs->denom, &rhs_fac);
    // Get RHS numer to LCM
    bigint_mul(&lhs_fac, &rhs->numer);
    // Do the add of numerators
    bigint_add(&lhs->numer, &lhs_fac);

    bigint_destroy(&lhs_fac);
    bigint_destroy(&rhs_fac);
}
