#pragma once
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef uint32_t digit_t;
extern const digit_t DIGIT_MAX;

typedef struct bigint {
    /**
     * The amount of space available in the digits array. The value of an unused
     * digit is undefind.
     */
    size_t capacity;
    /**
     * The number of significant digits in the digits array. Length is 0 iff the
     * value is 0.
     */
    size_t length;
    /**
     * Though I don't like it, I admit that in this case it makes sense: these
     * are the base (2^64-1) digits of the number in little endian order.
     */
    union {
        digit_t* digits;
        digit_t value;
    } contents;
} bigint;

extern const bigint bigint_zero;
extern const bigint bigint_one;

void bigint_init(bigint* v, size_t digit_length, digit_t* digits);
void bigint_init_const(bigint* v, size_t digit_length, const digit_t* digits);
void bigint_init_from_u64(bigint* v, uint64_t u64);
void bigint_destroy(bigint* v);
void bigint_clone(bigint* clone, const bigint* value);

/**
 * Add rhs to lhs in place. May reallocate lhs to be larger.
 */
void bigint_add(bigint* lhs, const bigint* rhs);

/**
 * Subtract rhs from lhs in place. Returns true if the result is negative.
 * May reallocate lhs to be larger.
 */
bool bigint_sub(bigint* lhs, const bigint* rhs);

void bigint_mul(bigint* lhs, const bigint* rhs);
void bigint_div(bigint* lhs, const bigint* rhs);
void bigint_rem(bigint* lhs, const bigint* rhs);
void bigint_pow(bigint* lhs, const bigint* rhs);

int bigint_cmp(const bigint* lhs, const bigint* rhs);
bool bigint_eq(const bigint* lhs, const bigint* rhs);
bool bigint_is_zero(const bigint* val);

char* bigint_to_string(const bigint* val);
uint64_t bigint_to_u64(const bigint* val);
