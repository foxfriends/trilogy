#pragma once
#include "bigint.h"
#include "types.h"
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

trilogy_number_value*
trilogy_number_init(trilogy_value* tv, trilogy_number_value* n);
trilogy_number_value* trilogy_number_init_new(
    trilogy_value* tv, bool re_is_negative, size_t re_numer_length,
    digit_t* re_numer, size_t re_denom_length, digit_t* re_denom,
    bool im_is_negative, size_t im_numer_length, digit_t* im_numer,
    size_t im_denom_length, digit_t* im_denom
);
trilogy_number_value* trilogy_number_init_u64(trilogy_value* tv, uint64_t i);

trilogy_number_value*
trilogy_number_clone_into(trilogy_value* tv, const trilogy_number_value* num);

uint64_t trilogy_number_to_u64(trilogy_number_value* tv);

trilogy_number_value* trilogy_number_untag(trilogy_value* val);
trilogy_number_value* trilogy_number_assume(trilogy_value* val);
void trilogy_number_destroy(trilogy_number_value* val);

int trilogy_number_compare(
    trilogy_number_value* lhs, trilogy_number_value* rhs
);
bool trilogy_number_eq(trilogy_number_value* lhs, trilogy_number_value* rhs);

void trilogy_number_add(
    trilogy_value* tv, const trilogy_number_value* lhs,
    const trilogy_number_value* rhs
);
void trilogy_number_sub(
    trilogy_value* tv, const trilogy_number_value* lhs,
    const trilogy_number_value* rhs
);
void trilogy_number_mul(
    trilogy_value* tv, const trilogy_number_value* lhs,
    const trilogy_number_value* rhs
);
void trilogy_number_div(
    trilogy_value* tv, const trilogy_number_value* lhs,
    const trilogy_number_value* rhs
);
void trilogy_number_rem(
    trilogy_value* tv, const trilogy_number_value* lhs,
    const trilogy_number_value* rhs
);

char* trilogy_number_to_string(const trilogy_number_value* tv);
