#pragma once
#include "bigint.h"
#include "types.h"
#include <stdbool.h>
#include <stddef.h>

trilogy_number_value*
trilogy_number_init(trilogy_value* tv, trilogy_number_value* n);
trilogy_number_value* trilogy_number_init_new(
    trilogy_value* tv, bool re_is_negative, size_t re_numer_length,
    unsigned long* re_numer, size_t re_denom_length, unsigned long* re_denom,
    bool im_is_negative, size_t im_numer_length, unsigned long* im_numer,
    size_t im_denom_length, unsigned long* im_denom
);
trilogy_number_value* trilogy_number_init_bigint(
    trilogy_value* tv, bool is_negative, bigint* i /* moved */
);
trilogy_number_value*
trilogy_number_init_ulong(trilogy_value* tv, unsigned long i);

trilogy_number_value*
trilogy_number_clone_into(trilogy_value* tv, trilogy_number_value* num);

unsigned long trilogy_number_to_ulong(trilogy_number_value* tv);

trilogy_number_value* trilogy_number_untag(trilogy_value* val);
trilogy_number_value* trilogy_number_assume(trilogy_value* val);
void trilogy_number_destroy(trilogy_number_value* val);

int trilogy_number_compare(
    trilogy_number_value* lhs, trilogy_number_value* rhs
);
bool trilogy_number_eq(trilogy_number_value* lhs, trilogy_number_value* rhs);
