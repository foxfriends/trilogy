#include "bigint.h"
#include "internal.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>

const bigint bigint_zero = {.capacity = 0, .length = 0, .digits = NULL};
const digit_t DIGIT_MAX = UINT32_MAX;
static const uint64_t BASE = ((uint64_t)UINT32_MAX + 1);
static size_t max(size_t lhs, size_t rhs) { return lhs > rhs ? lhs : rhs; }

static int digit_cmp(const digit_t* lhs, const digit_t* rhs, size_t i) {
    while (i-- > 0) {
        if (lhs[i] > rhs[i]) return 1;
        if (rhs[i] > lhs[i]) return -1;
    }
    return 0;
}

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

bool add_digit(digit_t* out, digit_t lhs, digit_t rhs, bool carry) {
    digit_t space = DIGIT_MAX - lhs;
    if (space > rhs) {
        *out = lhs + rhs + carry;
        return false;
    } else if (space == rhs) {
        *out = carry ? 0 : DIGIT_MAX;
        return carry;
    } else {
        *out = rhs - space - 1;
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
        carry = add_digit(&lhs->digits[i], lhs->digits[i], r, carry);
    }
    lhs->length = lhs->digits[capacity - 1] == 0 ? capacity - 1 : capacity;
}

static bool sub_digit(digit_t* out, digit_t lhs, digit_t rhs, bool borrow) {
    if (lhs > rhs) {
        *out = lhs - rhs - borrow;
        return false;
    } else if (lhs == rhs) {
        *out = borrow ? DIGIT_MAX : 0;
        return borrow;
    } else if (lhs == 0 && borrow) {
        *out = DIGIT_MAX - rhs;
        return true;
    } else {
        digit_t absdiff = rhs - (lhs - borrow);
        *out = DIGIT_MAX - absdiff + 1;
        return true;
    }
}

// 3 1
// 2

static size_t
bigint_sub_into(digit_t* out, const bigint* lhs, const bigint* rhs) {
    bool borrow = false;
    for (size_t i = 0; i < rhs->length; ++i) {
        borrow = sub_digit(&out[i], lhs->digits[i], rhs->digits[i], borrow);
    }
    if (borrow) {
        sub_digit(&out[rhs->length], lhs->digits[rhs->length], 0, borrow);
    }
    for (size_t i = lhs->length; i > 0; --i) {
        if (out[i - 1] != 0) return i;
    }
    return 0;
}

bool bigint_sub(bigint* lhs, const bigint* rhs) {
    if (bigint_cmp(lhs, rhs) == -1) {
        digit_t* out = malloc_safe(rhs->length * sizeof(digit_t));
        size_t length = bigint_sub_into(out, rhs, lhs);
        lhs->capacity = rhs->length;
        lhs->length = length;
        free(lhs->digits);
        lhs->digits = out;
        return true;
    }
    lhs->length = bigint_sub_into(lhs->digits, lhs, rhs);
    return false;
}

static void
digits_mul_by(digit_t* output, const digit_t* lhs, digit_t rhs, size_t len) {
    uint64_t carry = 0;
    for (size_t j = 0; j < len; j++) {
        uint64_t product = rhs * (uint64_t)lhs[j];
        uint64_t sum = (uint64_t)output[j] + carry + product;
        output[j] = (digit_t)sum;
        carry = product >> 32;
    }
    output[len] = (digit_t)carry;
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
        digits_mul_by(output + i, rhs->digits, lhs->digits[i], rhs->length);
    }
    free(lhs->digits);
    lhs->digits = output;
    lhs->capacity = capacity;
    lhs->length = capacity;
    while (lhs->length > 0 && lhs->digits[lhs->length - 1] == 0) {
        --lhs->length;
    }
}

static void digits_lsh(size_t length, digit_t* digits, unsigned int offset) {
    assert(offset < 32);
    for (size_t i = length; i > 0; --i) {
        digits[i - 1] <<= offset;
        if (i > 1) digits[i - 1] |= digits[i - 2] >> (32 - offset);
    }
}

static void digits_rsh(size_t length, digit_t* digits, unsigned int offset) {
    assert(offset < 32);
    for (size_t i = 0; i < length; ++i) {
        digits[i] >>= offset;
        if (i < length - 1) digits[i] |= digits[i + 1] << (32 - offset);
    }
}

static digit_t digits_div(digit_t* out, digit_t* lhs, digit_t rhs, size_t len) {
    uint64_t r = 0;
    size_t j = len - 1;
    do {
        uint64_t u = lhs[j];
        out[j] = (digit_t)((r * BASE + u) / rhs);
        r = (r * BASE + u) % rhs;
    } while (j-- > 0);
    return r;
}

void bigint_div(bigint* lhs, const bigint* rhs) {
    // REF: The Art of Computer Programming, Volume 2, Section 4.3.1, Algorithm
    // D (page 272)
    assert(lhs->length >= rhs->length);
    assert(rhs->length != 0);

    if (rhs->length == 1) {
        digits_div(lhs->digits, lhs->digits, rhs->digits[0], lhs->length);
        while (lhs->length > 0 && lhs->digits[lhs->length - 1] == 0) {
            --lhs->length;
        }
        return;
    }

    const size_t m = lhs->length - rhs->length;
    const size_t n = rhs->length;

    // Normalize
    uint64_t offset = 1;
    while (rhs->digits[n - 1] << offset < BASE / 2) {
        offset += 1;
    }

    digit_t* u = malloc_safe((n + m + 1) * sizeof(digit_t));
    memcpy(u, lhs->digits, (n + m) * sizeof(digit_t));
    u[n + m] = 0;
    digits_lsh(n + m + 1, u, offset);

    digit_t* v = malloc_safe(n * sizeof(digit_t));
    memcpy(v, rhs->digits, n * sizeof(digit_t));
    digits_lsh(n, v, offset);

    digit_t* q = malloc_safe((m + 1) * sizeof(digit_t));
    memset(q, 0, (m + 1) * sizeof(digit_t));

    digit_t* qv = malloc_safe((n + 1) * sizeof(digit_t));

    // Initialize j
    size_t j = m;
    do {
        // Calculate q^
        uint64_t u_head =
            (((uint64_t)u[n + j] * BASE) + (uint64_t)u[n + j - 1]);
        digit_t q_guess = u_head / v[n - 1];
        digit_t r_guess = u_head - q_guess * v[n - 1];
        while (q_guess >= BASE ||
               q_guess * v[n - 2] > BASE * r_guess + u[j + n - 2]) {
            q_guess -= 1;
            r_guess += v[n - 1];
            if (r_guess >= BASE) break;
        }

        // Multiply
        memset(qv, 0, (n + 1) * sizeof(digit_t));
        digits_mul_by(qv, v, q_guess, n);

        // Test remainder
        int cmp = digit_cmp(u + j, qv, n + 1);
        if (cmp == -1) {
            // Add back
            q_guess -= 1;
            memset(qv, 0, (n + 1) * sizeof(digit_t));
            digits_mul_by(qv, v, q_guess, n);
            assert(digit_cmp(u + j, qv, n + 1) != -1);
        }

        // Subtract
        bool borrow = false;
        for (size_t i = 0; i <= n; ++i) {
            borrow = sub_digit(&u[j + i], u[j + i], qv[i], borrow);
        }
        assert(!borrow);

        q[j] = q_guess;
        // Loop on j
    } while (j-- > 0);

    // Unnormalize
    free(qv);
    free(v);
    free(u);
    free(lhs->digits);
    lhs->digits = q;
    lhs->capacity = m + 1;
    lhs->length = m + 1;
    while (lhs->length > 0 && lhs->digits[lhs->length - 1] == 0) {
        --lhs->length;
    }
}

void bigint_rem(bigint* lhs, const bigint* rhs);
void bigint_pow(bigint* lhs, const bigint* rhs);

int bigint_cmp(const bigint* lhs, const bigint* rhs) {
    if (lhs->length > rhs->length) return 1;
    if (rhs->length > lhs->length) return -1;
    if (lhs->length == 0) return 0;
    return digit_cmp(lhs->digits, rhs->digits, lhs->length);
}

bool bigint_eq(const bigint* lhs, const bigint* rhs) {
    return bigint_cmp(lhs, rhs) == 0;
}

bool bigint_is_zero(const bigint* val) { return val->length == 0; }

char* bigint_to_string(const bigint* val) {
    if (val->length == 0) {
        char* str = malloc_safe(2 * sizeof(char));
        str[0] = '0';
        str[1] = '\0';
        return str;
    }
    if (val->length == 1) {
        int len = snprintf(NULL, 0, "%u", val->digits[0]);
        char* str = malloc_safe((len + 1) * sizeof(char) + 1);
        snprintf(str, len + 1, "%u", val->digits[0]);
        return str;
    }

    bigint n = bigint_zero;
    bigint_clone(&n, val);

    size_t len = 0;
    char* str = malloc_safe(20 * val->length * sizeof(char));

    while (!bigint_is_zero(&n)) {
        digit_t digit = digits_div(n.digits, n.digits, 10, n.length);
        str[len++] = '0' + digit;
        while (n.length > 0 && n.digits[n.length - 1] == 0) {
            --n.length;
        }
    }
    str[len] = '\0';
    bigint_destroy(&n);
    str = realloc_safe(str, (len + 1) * sizeof(char));
    for (size_t i = 0; i < len / 2; ++i) {
        char t = str[i];
        str[i] = str[len - i - 1];
        str[len - i - 1] = t;
    }
    return str;
}

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
