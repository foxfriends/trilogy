#include "bigint.h"
#include "internal.h"
#include <string.h>

const digit_t DIGIT_MAX = UINT32_MAX;

static size_t max(size_t lhs, size_t rhs) { return lhs > rhs ? lhs : rhs; }

bigint bigint_zero = {.capacity = 0, .length = 0, .digits = NULL};

void bigint_init(bigint* val, size_t length, digit_t* digits) {
    val->capacity = length;
    val->length = length;
    val->digits = digits;
}

void bigint_init_const(bigint* val, size_t length, const digit_t* digits) {
    val->capacity = length;
    val->length = length;
    val->digits = malloc_safe(length * sizeof(digit_t));
    memcpy(val->digits, digits, length * sizeof(digit_t));
}

void bigint_init_from_u64(bigint* val, uint64_t u64) {
    if (u64 == 0) {
        bigint_init(val, 0, NULL);
    } else if (u64 <= DIGIT_MAX) {
        digit_t* digits = malloc_safe(1 * sizeof(digit_t));
        digits[0] = (digit_t)u64;
        bigint_init(val, 1, digits);
    } else {
        digit_t* digits = malloc_safe(2 * sizeof(digit_t));
        digits[0] = (digit_t)u64;
        digits[1] = (digit_t)(u64 >> 32);
        bigint_init(val, 2, digits);
    }
}

void bigint_clone(bigint* clone, const bigint* value) {
    if (clone->capacity < value->length) {
        bigint_destroy(clone);
        clone->digits = malloc_safe(value->length * sizeof(digit_t));
        clone->capacity = value->length;
    }
    clone->length = value->length;
    if (value->length > 0) {
        memcpy(clone->digits, value->digits, value->length * sizeof(digit_t));
    }
}

void bigint_destroy(bigint* v) {
    if (v->digits != NULL) free(v->digits);
    v->capacity = 0;
    v->digits = NULL;
}

bool add_digit(digit_t* lhs, digit_t rhs, bool carry) {
    digit_t space = DIGIT_MAX - *lhs;
    if (space > rhs) {
        *lhs = *lhs + rhs + carry;
        return false;
    } else if (space == rhs) {
        *lhs = carry ? 0 : DIGIT_MAX;
        return carry;
    } else {
        *lhs = rhs - space;
        return true;
    }
}

void bigint_add(bigint* lhs, const bigint* rhs) {
    size_t capacity = max(lhs->length, rhs->length);
    if (capacity == SIZE_MAX) {
        internal_panic("bigint capacity limit\n");
    }
    capacity += 1;
    if (lhs->capacity < capacity) {
        lhs->digits = realloc_safe(lhs->digits, sizeof(digit_t) * capacity);
        lhs->capacity = capacity;
    }
    bool carry = false;
    for (size_t i = 0; i < capacity; ++i) {
        digit_t r = i < rhs->length ? rhs->digits[i] : 0;
        if (i >= lhs->length) {
            lhs->digits[i] = 0;
        }
        carry = add_digit(&lhs->digits[i], r, carry);
    }
    lhs->length = lhs->digits[capacity - 1] == 0 ? capacity - 1 : capacity;
}

static size_t
bigint_sub_from(digit_t* out, const bigint* lhs, const bigint* rhs) {
    bool borrow = false;
    for (size_t i = 0; i < rhs->length; ++i) {
        if (lhs->digits[i] > rhs->digits[i]) {
            out[i] = lhs->digits[i] - rhs->digits[i] - borrow;
            borrow = false;
        } else if (lhs->digits[i] == rhs->digits[i]) {
            out[i] = borrow ? DIGIT_MAX : 0;
        } else {
            digit_t absdiff = rhs->digits[i] - lhs->digits[i];
            out[i] = DIGIT_MAX - absdiff + 1;
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
        digit_t* out = malloc_safe(rhs->length * sizeof(digit_t));
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

void bigint_mul(bigint* lhs, const bigint* rhs) {
    size_t available = SIZE_MAX - lhs->length;
    if (available < rhs->length) {
        internal_panic("bigint capacity limit\n");
    }
    size_t capacity = lhs->length + rhs->length;
    digit_t* output = malloc_safe(sizeof(digit_t) * capacity);
    memset(output, 0, sizeof(digit_t) * capacity);
    for (size_t i = 0; i < lhs->length; i++) {
        uint64_t carry = 0;
        for (size_t j = 0; j < rhs->length; j++) {
            uint64_t product =
                (uint64_t)lhs->digits[i] * (uint64_t)rhs->digits[j];
            uint64_t sum = (uint64_t)output[i + j] + carry + product;
            output[i + j] = (digit_t)sum;
            carry = product >> 32;
        }
        output[rhs->length + i] = (digit_t)carry;
    }
    free(lhs->digits);
    lhs->digits = output;
    lhs->capacity = capacity;
    lhs->length = capacity;
    while (lhs->length > 0 && lhs->digits[lhs->length - 1] == 0) {
        --lhs->length;
    }
}

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

uint64_t bigint_to_u64(const bigint* val) {
    switch (val->length) {
    case 0:
        return 0;
    case 1:
        return (uint64_t)val->digits[0];
    case 2:
        return (uint64_t)val->digits[0] + ((uint64_t)val->digits[1] << 32);
    default:
        internal_panic("expected u64, but number is too large");
    }
}
