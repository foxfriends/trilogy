#include "bigint.h"
#include "internal.h"
#include <limits.h>
#include <string.h>

static size_t max(size_t lhs, size_t rhs) { return lhs > rhs ? lhs : rhs; }

bigint bigint_zero = {.capacity = 0, .length = 0, .digits = NULL};

void bigint_init(bigint* val, size_t length, unsigned long* digits) {
    val->capacity = length;
    val->length = length;
    val->digits = digits;
}

void bigint_init_const(
    bigint* val, size_t length, const unsigned long* digits
) {
    val->capacity = length;
    val->length = length;
    val->digits = calloc_safe(length, sizeof(unsigned long));
    memcpy(val->digits, digits, length * sizeof(unsigned long) / 8);
}

void bigint_init_from_ulong(bigint* val, unsigned long u64) {
    if (u64 == 0) {
        bigint_init(val, 0, NULL);
        return;
    }
    unsigned long* digits = calloc_safe(1, sizeof(unsigned long));
    digits[0] = u64;
    bigint_init(val, 1, digits);
}

void bigint_clone(bigint* clone, const bigint* value) {
    if (clone->capacity < value->length) {
        bigint_destroy(clone);
        clone->capacity = value->capacity;
        clone->digits = calloc_safe(value->length, sizeof(unsigned long));
    }
    clone->length = value->length;
    if (value->length > 0) {
        memcpy(
            clone->digits, value->digits,
            value->length * sizeof(unsigned long) / 8
        );
    }
}

void bigint_destroy(bigint* v) {
    if (v->digits != NULL) free(v->digits);
    v->capacity = 0;
    v->digits = NULL;
}

bool add_digit(unsigned long* lhs, unsigned long rhs, bool carry) {
    unsigned long space = ULONG_MAX - *lhs;
    if (space > rhs) {
        *lhs = *lhs + rhs + carry;
        return false;
    } else if (space == rhs) {
        *lhs = carry ? 0 : ULONG_MAX;
        return carry;
    } else {
        *lhs = rhs - space;
        return true;
    }
}

void bigint_add(bigint* lhs, const bigint* rhs) {
    size_t capacity = max(lhs->length, rhs->length) + 1;
    if (lhs->capacity < capacity) {
        lhs->digits = realloc_safe(lhs->digits, capacity);
        lhs->capacity = capacity;
    }
    bool carry = false;
    for (size_t i = 0; i < capacity; ++i) {
        unsigned long r = i < rhs->length ? rhs->digits[i] : 0;
        if (i >= lhs->length) {
            lhs->digits[i] = 0;
        }
        carry = add_digit(&lhs->digits[i], r, carry);
    }
    lhs->length = lhs->digits[capacity - 1] == 0 ? capacity - 1 : capacity;
}

static size_t
bigint_sub_from(unsigned long* out, const bigint* lhs, const bigint* rhs) {
    bool borrow = false;
    for (size_t i = 0; i < rhs->length; ++i) {
        if (lhs->digits[i] > rhs->digits[i]) {
            out[i] = lhs->digits[i] - rhs->digits[i] - borrow;
            borrow = false;
        } else if (lhs->digits[i] == rhs->digits[i]) {
            out[i] = borrow ? ULONG_MAX : 0;
        } else {
            unsigned long absdiff = rhs->digits[i] - lhs->digits[i];
            out[i] = ULONG_MAX - absdiff + 1;
            borrow = true;
        }
    }
    for (size_t i = rhs->length; i > 0; --i) {
        if (out[i - 1] != 0) return i;
    }
    return 0;
}

bool bigint_sub(bigint* lhs, const bigint* rhs) {
    if (bigint_cmp(lhs, rhs) == -1) {
        unsigned long* out = calloc_safe(rhs->length, sizeof(unsigned long));
        size_t length = bigint_sub_from(out, rhs, lhs);
        lhs->capacity = rhs->length;
        lhs->length = length;
        free(lhs->digits);
        lhs->digits = out;
        return true;
    }
    lhs->length = bigint_sub_from(lhs->digits, lhs, rhs);
    return false;
}

void bigint_mul(bigint* lhs, const bigint* rhs);
void bigint_div(bigint* lhs, const bigint* rhs);
void bigint_rem(bigint* lhs, const bigint* rhs);
void bigint_pow(bigint* lhs, const bigint* rhs);

int bigint_cmp(const bigint* lhs, const bigint* rhs) {
    if (lhs->length > rhs->length) return 1;
    if (rhs->length > lhs->length) return -1;
    if (lhs->length == 0) return 0;
    size_t i = lhs->length;
    while (i-- > 0) {
        if (lhs->digits[i] > rhs->digits[i]) return 1;
        if (rhs->digits[i] > lhs->digits[i]) return -1;
    }
    return 0;
}

bool bigint_eq(const bigint* lhs, const bigint* rhs) {
    return bigint_cmp(lhs, rhs) == 0;
}

bool bigint_is_zero(const bigint* val) { return val->length == 0; }

char* bigint_to_string(const bigint* val);

unsigned long bigint_to_ulong(const bigint* val) {
    if (val->length > 1)
        internal_panic("expected unsigned long, but number is too large");
    if (val->length == 0) return 0;
    return val->digits[0];
}
