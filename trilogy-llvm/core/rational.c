#include "rational.h"
#include "internal.h"
#include <assert.h>
#include <stdlib.h>
#include <string.h>

const rational rational_zero = {
    .is_negative = false,
    .numer = {.capacity = 0, .length = 1, .contents = {.value = 0}},
    .denom = {.capacity = 0, .length = 1, .contents = {.value = 1}},
};

void rational_init_const(
    rational* rat, bool is_negative, size_t numer_length, const digit_t* numer,
    size_t denom_length, const digit_t* denom
) {
    rat->is_negative = is_negative;
    bigint_init_const(&rat->numer, numer_length, numer);
    bigint_init_const(&rat->denom, denom_length, denom);
    rational_reduce(rat);
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
        if (!bigint_is_one(gcd)) {
            bigint_div(&val->numer, gcd);
            bigint_div(&val->denom, gcd);
        }
        bigint_destroy(gcd);
        free(gcd);
    }
    if (bigint_is_zero(&val->numer)) val->is_negative = false;
}

void rational_negate(rational* val) {
    if (rational_is_zero(val)) return;
    val->is_negative = !val->is_negative;
}

static void rational_add_unsigned(rational* lhs, const rational* rhs) {
    if (bigint_eq(&lhs->denom, &rhs->denom)) {
        bigint_add(&lhs->numer, &rhs->numer);
        rational_reduce(lhs);
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
    rational_reduce(lhs);
}

static void rational_sub_unsigned(rational* lhs, const rational* rhs) {
    if (bigint_eq(&lhs->denom, &rhs->denom)) {
        bool is_negative = bigint_sub(&lhs->numer, &rhs->numer);
        if (is_negative) rational_negate(lhs);
        rational_reduce(lhs);
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
    // Do the sub of numerators
    bool is_negative = bigint_sub(&lhs->numer, &lhs_fac);
    if (is_negative) rational_negate(lhs);

    bigint_destroy(&lhs_fac);
    bigint_destroy(&rhs_fac);
    rational_reduce(lhs);
}

void rational_add(rational* lhs, const rational* rhs) {
    if (lhs->is_negative == rhs->is_negative) {
        rational_add_unsigned(lhs, rhs);
    } else {
        rational_sub_unsigned(lhs, rhs);
    }
}

void rational_sub(rational* lhs, const rational* rhs) {
    if (lhs->is_negative == rhs->is_negative) {
        rational_sub_unsigned(lhs, rhs);
    } else {
        rational_add_unsigned(lhs, rhs);
    }
}

void rational_mul(rational* lhs, const rational* rhs) {
    bigint_mul(&lhs->numer, &rhs->numer);
    bigint_mul(&lhs->denom, &rhs->denom);
    lhs->is_negative = lhs->is_negative != rhs->is_negative;
    rational_reduce(lhs);
}

void rational_div(rational* lhs, const rational* rhs) {
    assert(!rational_is_zero(rhs));
    bigint_mul(&lhs->numer, &rhs->denom);
    bigint_mul(&lhs->denom, &rhs->numer);
    lhs->is_negative = lhs->is_negative != rhs->is_negative;
    rational_reduce(lhs);
}

void rational_rem(rational* lhs, const rational* rhs) {
    // TODO: this is intentionally not supporting fractions at this time
    assert(rational_is_whole(lhs));
    assert(rational_is_whole(rhs));
    bigint_rem(&lhs->numer, &rhs->numer);
    rational_reduce(lhs);
}

char* rational_to_string(const rational* val) {
    if (bigint_is_one(&val->denom)) {
        if (!val->is_negative) return bigint_to_string(&val->numer);
        char* numer = bigint_to_string(&val->numer);
        size_t len = strlen(numer);
        char* negated = malloc_safe(sizeof(char) * len + 2);
        negated[0] = '-';
        strncpy(negated + 1, numer, len);
        negated[len + 1] = '\0';
        free(numer);
        return negated;
    }
    char* numer = bigint_to_string(&val->numer);
    char* denom = bigint_to_string(&val->denom);
    size_t numer_len = strlen(numer);
    size_t denom_len = strlen(denom);
    size_t new_len = numer_len + denom_len + 1;
    if (val->is_negative) ++new_len;
    char* joined = malloc_safe(sizeof(char) * new_len + 1);
    size_t i = 0;
    if (val->is_negative) {
        joined[0] = '-';
        ++i;
    }
    strncpy(joined + i, numer, numer_len);
    i += numer_len;
    joined[i] = '/';
    ++i;
    strncpy(joined + i, denom, denom_len);
    i += denom_len;
    joined[i] = '\0';
    free(numer);
    free(denom);
    return joined;
}
