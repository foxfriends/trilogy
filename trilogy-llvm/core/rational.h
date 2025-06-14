#pragma once
#include "bigint.h"
#include <stdbool.h>
#include <stddef.h>

typedef struct rational {
    bool is_negative;
    bigint numer;
    bigint denom;
} rational;

// clang-format off
#define RATIONAL_ZERO {.is_negative = false, .numer = BIGINT_ZERO, .denom = BIGINT_ONE}
#define RATIONAL_ONE {.is_negative = false, .numer = BIGINT_ONE, .denom = BIGINT_ONE}
// clang-format on

extern const rational rational_zero;
extern const rational rational_one;

void rational_init_const(
    rational*, bool is_negative, size_t numer_length, const digit_t* numer,
    size_t denom_length, const digit_t* denom
);
void rational_clone(rational* into, const rational* from);
void rational_destroy(rational* val);

bool rational_is_zero(const rational* val);
bool rational_is_whole(const rational* val);

int rational_cmp(const rational* lhs, const rational* rhs);
bool rational_eq(const rational* lhs, const rational* rhs);

void rational_reduce(rational* val);
void rational_negate(rational* val);
void rational_truncate(rational* val);

void rational_add(rational* lhs, const rational* rhs);
void rational_sub(rational* lhs, const rational* rhs);
void rational_mul(rational* lhs, const rational* rhs);
void rational_div(rational* lhs, const rational* rhs);
void rational_rem(rational* lhs, const rational* rhs);

char* rational_to_string(const rational* val);
char* rational_to_string_unsigned(const rational* val);
