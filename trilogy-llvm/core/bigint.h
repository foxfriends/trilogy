#pragma once
#include <stdbool.h>
#include <stdlib.h>

typedef struct bigint {
    /**
     * The amount of space available in the digits array. The value of an unused
     * digit is undefind.
     */
    size_t capacity;
    /**
     * The number of significant digits in the digits array.
     */
    size_t length;
    /**
     * Though I don't like it, I admit that in this case it makes sense: these
     * are the base (2^64-1) digits of the number in little endian order.
     */
    unsigned long* digits;
} bigint;

void bigint_init(bigint* v, size_t digit_length, unsigned long* digits);
void bigint_init_const(
    bigint* v, size_t digit_length, const unsigned long* digits
);
void bigint_init_from_ulong(bigint* v, unsigned long u64);
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
unsigned long bigint_to_ulong(const bigint* val);
