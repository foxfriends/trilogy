#include "trilogy_number.h"
#include "bigint.h"
#include "internal.h"
#include "rational.h"
#include "trilogy_value.h"
#include "types.h"
#include <assert.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

static const trilogy_number_value trilogy_number_one = {
    .re = RATIONAL_ONE, .im = RATIONAL_ZERO
};

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

static trilogy_number_value*
trilogy_number_clone(const trilogy_number_value* num) {
    trilogy_number_value* clone = malloc_safe(sizeof(trilogy_number_value));
    rational_clone(&clone->re, &num->re);
    rational_clone(&clone->im, &num->im);
    return clone;
}

trilogy_number_value*
trilogy_number_clone_into(trilogy_value* tv, const trilogy_number_value* num) {
    trilogy_number_value* clone = trilogy_number_clone(num);
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
    if (rational_is_zero(&lhs->im) && rational_is_zero(&rhs->im)) {
        return rational_cmp(&lhs->re, &rhs->re);
    }
    return -2;
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

/**
 * Does number multiplication into lhs_mut; lhs_mut should equal lhs but not be
 * pointed to the same location for this to work.
 */
static void number_mul(
    trilogy_number_value* restrict lhs_mut, const trilogy_number_value* lhs,
    const trilogy_number_value* rhs
) {
    if (rational_is_zero(&lhs->im) && rational_is_zero(&rhs->im)) {
        // Real multiplication is easy
        rational_mul(&lhs_mut->re, &rhs->re);
        return;
    }

    rational term;
    // Complex multiplication: (a+bi) * (c+di) = (ac - bd) + (ad + bc)i

    // real part: ac - bd
    rational_clone(&term, &lhs->im /* b */);
    rational_mul(&term, &rhs->im /* d */);
    rational_mul(&lhs_mut->re /* a */, &rhs->re /* c */);
    rational_sub(&lhs_mut->re, &term);
    rational_destroy(&term);

    // imaginary part: ad + bc
    rational_clone(&term, &lhs->re /* a */);
    rational_mul(&term, &rhs->im /* d */);
    rational_mul(&lhs_mut->im /* b */, &rhs->re /* c */);
    rational_add(&lhs_mut->im, &term);
    rational_destroy(&term);
}

void trilogy_number_mul(
    trilogy_value* tv, const trilogy_number_value* lhs,
    const trilogy_number_value* rhs
) {
    trilogy_number_value* lhs_mut = trilogy_number_clone_into(tv, lhs);
    number_mul(lhs_mut, lhs, rhs);
}

void trilogy_number_div(
    trilogy_value* tv, const trilogy_number_value* lhs,
    const trilogy_number_value* rhs
) {
    trilogy_number_value* lhs_mut = trilogy_number_clone_into(tv, lhs);
    if (rational_is_zero(&lhs->im) && rational_is_zero(&rhs->im)) {
        // Real division is easy
        rational_div(&lhs_mut->re, &rhs->re);
        return;
    }

    // Complex division:
    // (u+vi) / (x+yi)
    //     = (ux + vy)/(x^2 + y^2)
    //     + (vx - uy)/(x^2 + y^2)i

    // x^2 + y^2
    rational xy;
    rational y;
    rational_clone(&xy, &rhs->re);
    rational_mul(&xy, &rhs->re);
    rational_clone(&y, &rhs->im);
    rational_mul(&y, &rhs->im);
    rational_add(&xy, &y);
    rational_destroy(&y);

    // (ux + vy)
    rational_clone(&y, &rhs->im);
    rational_mul(&y, &lhs->im);
    rational_mul(&lhs_mut->re, &rhs->re);
    rational_add(&lhs_mut->re, &y);
    // (ux + vy) / (x^2 + y^2)
    rational_div(&lhs_mut->re, &xy);
    rational_destroy(&y);

    // (vx - uy)
    rational_clone(&y, &rhs->im);
    rational_mul(&y, &lhs->re);
    rational_mul(&lhs_mut->im, &rhs->re);
    rational_sub(&lhs_mut->im, &y);
    // (vx - uy) / (x^2 + y^2)
    rational_div(&lhs_mut->im, &xy);
    rational_destroy(&y);
    rational_destroy(&xy);
}

void trilogy_number_int_div(
    trilogy_value* tv, const trilogy_number_value* lhs,
    const trilogy_number_value* rhs
) {
    trilogy_number_value* lhs_mut = trilogy_number_clone_into(tv, lhs);
    if (rational_is_zero(&lhs->im) && rational_is_zero(&rhs->im)) {
        // Real division is easy
        rational_div(&lhs_mut->re, &rhs->re);
        rational_truncate(&lhs_mut->re);
        rational_truncate(&lhs_mut->im);
        return;
    }

    // Complex division: compute the full division, truncate the real part, and
    // throw away the imaginary part.
    //
    // (u+vi) // (x+yi)
    //     = ⌊(ux + vy)/(x^2 + y^2)⌋_0

    // x^2 + y^2
    rational xy;
    rational y;
    rational_clone(&xy, &rhs->re);
    rational_mul(&xy, &rhs->re);
    rational_clone(&y, &rhs->im);
    rational_mul(&y, &rhs->im);
    rational_add(&xy, &y);
    rational_destroy(&y);

    // (ux + vy)
    rational_clone(&y, &rhs->im);
    rational_mul(&y, &lhs->im);
    rational_mul(&lhs_mut->re, &rhs->re);
    rational_add(&lhs_mut->re, &y);
    // (ux + vy) / (x^2 + y^2)
    rational_div(&lhs_mut->re, &xy);
    rational_destroy(&y);
    rational_truncate(&lhs_mut->re);

    rational_destroy(&lhs_mut->im);
    lhs_mut->im = rational_zero;
}

void trilogy_number_rem(
    trilogy_value* tv, const trilogy_number_value* lhs,
    const trilogy_number_value* rhs
) {
    if (rational_is_zero(&lhs->im) && rational_is_zero(&rhs->im)) {
        // Real remainder is easy
        trilogy_number_value* lhs_mut = trilogy_number_clone_into(tv, lhs);
        rational_rem(&lhs_mut->re, &rhs->re);
        return;
    }
    // Using this as reference for "complex remainder" but currently
    // disregarding the final note about "What you definitely should not do is
    // categorically round both [...] towards zero"
    //     https://math.stackexchange.com/questions/889809/calculating-the-reminder-when-dividing-complex-numbers
    // That is to say, the results of this operation may make actually no sense
    // at all, and I will have to improve the concept later
    trilogy_value qv = trilogy_undefined;
    trilogy_number_int_div(&qv, lhs, rhs);
    trilogy_number_value* q = trilogy_number_assume(&qv);
    trilogy_value rhs_q = trilogy_undefined;
    trilogy_number_mul(&rhs_q, q, rhs);
    trilogy_number_sub(tv, lhs, trilogy_number_assume(&rhs_q));
    trilogy_value_destroy(&qv);
    trilogy_value_destroy(&rhs_q);
}

static void int_pow(trilogy_number_value* val, bigint* exp) {
    if (bigint_is_one(exp)) return;
    if (bigint_is_zero(exp)) {
        trilogy_number_destroy(val);
        *val = trilogy_number_one;
        return;
    }

    trilogy_number_value x = *val;
    *val = trilogy_number_one;

    do {
        if (bigint_is_odd(exp)) {
            trilogy_number_value* yy = trilogy_number_clone(val);
            number_mul(val, yy, &x);
            trilogy_number_destroy(yy);
            free(yy);
        }

        trilogy_number_value* xx = trilogy_number_clone(&x);
        number_mul(&x, xx, xx);
        trilogy_number_destroy(xx);
        free(xx);

        bigint_half(exp);
    } while (!bigint_is_zero(exp) && !bigint_is_one(exp));
    trilogy_number_value* yy = trilogy_number_clone(val);
    number_mul(val, yy, &x);
    trilogy_number_destroy(yy);
    free(yy);
    trilogy_number_destroy(&x);
}

void trilogy_number_pow(
    trilogy_value* tv, const trilogy_number_value* lhs,
    const trilogy_number_value* rhs
) {
    // Pow gets kind of tricky...
    if (rational_is_zero(&rhs->im)) {
        // Complex powers not yet implemented
        if (rational_is_whole(&rhs->re)) {
            // If it's an integer power, it's pretty easy, but depends on sign:
            bigint exp = bigint_zero;
            bigint_clone(&exp, &rhs->re.numer);
            if (rhs->re.is_negative) {
                trilogy_number_div(tv, &trilogy_number_one, lhs);
                int_pow(trilogy_number_assume(tv), &exp);
            } else {
                int_pow(trilogy_number_clone_into(tv, lhs), &exp);
            }
            bigint_destroy(&exp);
        } else {
            // If it's not an integer power, this is an nth root and therefore
            // likely irrational, which means we probably cannot represent it
            // accurately.
            internal_panic("unimplemented: fractional powers");
        }
    } else {
        // If it's not a real power, then b^z = e^(z ln b), apparently, but
        // since we cannot represent `e` accurately, this is also not possible.
        internal_panic("unimplemented: complex powers");
    }
}

void trilogy_number_negate(trilogy_value* tv, const trilogy_number_value* val) {
    trilogy_number_value* lhs_mut = trilogy_number_clone_into(tv, val);
    rational_negate(&lhs_mut->re);
    rational_negate(&lhs_mut->im);
}

char* trilogy_number_to_string(const trilogy_number_value* val) {
    if (rational_is_zero(&val->im)) {
        char* re = rational_to_string(&val->re);
        return re;
    }
    if (rational_is_zero(&val->re)) {
        char* im = rational_to_string(&val->im);
        size_t im_len = strlen(im);
        im = realloc_safe(im, im_len + 2);
        im[im_len] = 'i';
        im[im_len + 1] = '\0';
        return im;
    }
    char* re = rational_to_string(&val->re);
    char* im = rational_to_string_unsigned(&val->im);
    size_t re_len = strlen(re);
    size_t im_len = strlen(im);
    char* joined = malloc_safe(sizeof(char) * im_len + re_len + 3);
    strncpy(joined, re, re_len);
    joined[re_len] = val->im.is_negative ? '-' : '+';
    strncpy(joined + re_len + 1, im, im_len);
    joined[re_len + im_len + 1] = 'i';
    joined[re_len + im_len + 2] = '\0';
    free(re);
    free(im);
    return joined;
}
