#include "bigint.h"
#include "internal.h"
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

const bigint bigint_zero = BIGINT_ZERO;
const bigint bigint_one = BIGINT_ONE;

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
    val->contents.digits = digits;
}

void bigint_init_small(bigint* val, digit_t digit) {
    val->capacity = 0;
    val->length = 1;
    val->contents.value = digit;
}

void bigint_init_const(bigint* val, size_t length, const digit_t* digits) {
    if (length == 0) {
        *val = bigint_zero;
        return;
    }
    if (length == 1) {
        val->capacity = 0;
        val->length = 1;
        val->contents.value = digits[0];
        return;
    }
    val->capacity = length;
    val->length = length;
    val->contents.digits = malloc_safe(length * sizeof(digit_t));
    memcpy(val->contents.digits, digits, length * sizeof(digit_t));
}

void bigint_init_from_u64(bigint* val, uint64_t u64) {
    if (u64 <= DIGIT_MAX) {
        bigint_init_small(val, (digit_t)u64);
    } else {
        digit_t* digits = malloc_safe(2 * sizeof(digit_t));
        digits[0] = (digit_t)u64;
        digits[1] = (digit_t)(u64 >> 32);
        bigint_init(val, 2, digits);
    }
}

void bigint_clone(bigint* clone, const bigint* value) {
    if (value->length == 1) {
        assert(value->capacity == 0);
        *clone = *value;
        return;
    }
    clone->contents.digits = malloc_safe(value->length * sizeof(digit_t));
    clone->capacity = value->length;
    clone->length = value->length;
    memcpy(
        clone->contents.digits, value->contents.digits,
        value->length * sizeof(digit_t)
    );
}

void bigint_destroy(bigint* v) {
    if (v->capacity != 0) free(v->contents.digits);
    v->capacity = 0;
    v->contents.digits = NULL;
}

bool add_digit(digit_t* out, digit_t lhs, digit_t rhs, bool carry) {
    digit_t space = DIGIT_MAX - lhs;
    if (space > rhs) {
        *out = lhs + rhs + carry;
        return false;
    }
    if (space == rhs) {
        *out = carry ? 0 : DIGIT_MAX;
        return carry;
    }
    *out = rhs - space - 1;
    return true;
}

static void ensure_capacity(bigint* val, size_t capacity) {
    if (capacity <= 1) return;
    if (val->length == 1) {
        assert(val->capacity == 0);
        digit_t value = val->contents.value;
        val->contents.digits = malloc_safe(sizeof(digit_t) * capacity);
        val->contents.digits[0] = value;
        val->capacity = capacity;
    } else if (val->capacity < capacity) {
        val->contents.digits =
            realloc_safe(val->contents.digits, sizeof(digit_t) * capacity);
        val->capacity = capacity;
    }
}

static digit_t digit_at(const bigint* val, size_t i) {
    if (val->length == 1 && val->capacity == 0) {
        return i == 0 ? val->contents.value : 0;
    }
    return i < val->length ? val->contents.digits[i] : 0;
}

void bigint_add(bigint* lhs, const bigint* rhs) {
    size_t capacity = max(lhs->length, rhs->length);
    if (capacity == 1) {
        if (lhs->contents.value <= DIGIT_MAX - rhs->contents.value) {
            lhs->contents.value += rhs->contents.value;
            return;
        }
    } else if (capacity == SIZE_MAX) {
        internal_panic("bigint capacity limit\n");
    }
    capacity += 1;
    ensure_capacity(lhs, capacity);
    bool carry = false;
    for (size_t i = 0; i < capacity; ++i) {
        digit_t r = digit_at(rhs, i);
        if (i >= lhs->length) {
            lhs->contents.digits[i] = 0;
        }
        carry = add_digit(
            &lhs->contents.digits[i], lhs->contents.digits[i], r, carry
        );
    }
    lhs->length =
        lhs->contents.digits[capacity - 1] == 0 ? capacity - 1 : capacity;
}

static bool sub_digit(digit_t* out, digit_t lhs, digit_t rhs, bool borrow) {
    if (lhs > rhs) {
        *out = lhs - rhs - borrow;
        return false;
    }
    if (lhs == rhs) {
        *out = borrow ? DIGIT_MAX : 0;
        return borrow;
    }
    if (lhs == 0 && borrow) {
        *out = DIGIT_MAX - rhs;
        return true;
    }
    digit_t absdiff = rhs - (lhs - borrow);
    *out = DIGIT_MAX - absdiff + 1;
    return true;
}

static size_t
bigint_sub_into(digit_t* out, const bigint* lhs, const bigint* rhs) {
    bool borrow = false;
    for (size_t i = 0; i < rhs->length; ++i) {
        borrow = sub_digit(
            &out[i], lhs->contents.digits[i], digit_at(rhs, i), borrow
        );
    }
    if (borrow) {
        sub_digit(
            &out[rhs->length], lhs->contents.digits[rhs->length], 0, borrow
        );
    }
    for (size_t i = lhs->length; i > 0; --i) {
        if (out[i - 1] != 0) return i;
    }
    return 0;
}

static void inline_contents(bigint* val) {
    assert(val->length == 1);
    assert(val->capacity != 0);
    digit_t value = val->contents.digits[0];
    free(val->contents.digits);
    val->capacity = 0;
    val->contents.value = value;
}

bool bigint_sub(bigint* lhs, const bigint* rhs) {
    if (lhs->length == 1 && rhs->length == 1) {
        if (lhs->contents.value > rhs->contents.value) {
            lhs->contents.value -= rhs->contents.value;
            return false;
        }
        lhs->contents.value = rhs->contents.value - lhs->contents.value;
        return true;
    }
    if (bigint_cmp(lhs, rhs) == -1) {
        digit_t* out = malloc_safe(rhs->length * sizeof(digit_t));
        size_t length = bigint_sub_into(out, rhs, lhs);
        if (lhs->length) free(lhs->contents.digits);
        lhs->capacity = rhs->length;
        lhs->length = length;
        lhs->contents.digits = out;
        if (lhs->length == 1) inline_contents(lhs);
        return true;
    }
    lhs->length = bigint_sub_into(lhs->contents.digits, lhs, rhs);
    if (lhs->length == 1) inline_contents(lhs);
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

static const digit_t* digits_ptr(const bigint* val) {
    return val->capacity == 0 ? &val->contents.value : val->contents.digits;
}

static digit_t* digits_ptr_mut(bigint* val) {
    return val->capacity == 0 ? &val->contents.value : val->contents.digits;
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
        digits_mul_by(
            output + i, digits_ptr(rhs), digit_at(lhs, i), rhs->length
        );
    }
    if (lhs->capacity) free(lhs->contents.digits);
    lhs->contents.digits = output;
    lhs->capacity = capacity;
    lhs->length = capacity;
    while (lhs->length > 1 && lhs->contents.digits[lhs->length - 1] == 0) {
        --lhs->length;
    }
    if (lhs->length == 1) inline_contents(lhs);
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

void bigint_half(bigint* val) {
    digits_rsh(val->length, digits_ptr_mut(val), 1);
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

void bigint_div_rem(bigint* lhs, const bigint* rhs, bigint* rem_out) {
    // REF: The Art of Computer Programming, Volume 2, Section 4.3.1, Algorithm
    // D (page 272)
    assert(lhs->length >= rhs->length);
    assert(rhs->length != 1 || rhs->contents.value != 0);

    if (rhs->length == 1) {
        digit_t r = digits_div(
            digits_ptr_mut(lhs), digits_ptr_mut(lhs), rhs->contents.value,
            lhs->length
        );
        while (lhs->length > 1 && lhs->contents.digits[lhs->length - 1] == 0) {
            --lhs->length;
        }
        if (lhs->length == 1 && lhs->capacity != 0) inline_contents(lhs);
        if (rem_out != NULL) bigint_init_small(rem_out, r);
        return;
    }

    const size_t m = lhs->length - rhs->length;
    const size_t n = rhs->length;

    // Normalize
    uint64_t offset = 1;
    while (rhs->contents.digits[n - 1] << offset < BASE / 2) {
        offset += 1;
    }

    digit_t* u = malloc_safe((n + m + 1) * sizeof(digit_t));
    memcpy(u, digits_ptr(lhs), (n + m) * sizeof(digit_t));
    u[n + m] = 0;
    digits_lsh(n + m + 1, u, offset);

    digit_t* v = malloc_safe(n * sizeof(digit_t));
    memcpy(v, digits_ptr(rhs), n * sizeof(digit_t));
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
    free(lhs->contents.digits);
    lhs->contents.digits = q;
    lhs->capacity = m + 1;
    lhs->length = m + 1;
    while (lhs->length > 1 && lhs->contents.digits[lhs->length - 1] == 0) {
        --lhs->length;
    }
    if (lhs->length == 1) inline_contents(lhs);

    if (rem_out != NULL) {
        digits_rsh(n + m + 1, u, offset);
        rem_out->contents.digits = u;
        rem_out->capacity = n + m + 1;
        rem_out->length = n + m + 1;
        while (rem_out->length > 1 &&
               rem_out->contents.digits[rem_out->length - 1] == 0) {
            --rem_out->length;
        }
        if (rem_out->length == 1) inline_contents(rem_out);
    } else {
        free(u);
    }
    free(qv);
    free(v);
}

void bigint_div(bigint* lhs, const bigint* rhs) {
    bigint_div_rem(lhs, rhs, NULL);
}

void bigint_rem(bigint* lhs, const bigint* rhs) {
    bigint out = bigint_zero;
    bigint_div_rem(lhs, rhs, &out);
    bigint_destroy(lhs);
    *lhs = out;
}

int bigint_cmp(const bigint* lhs, const bigint* rhs) {
    if (lhs->length > rhs->length) return 1;
    if (rhs->length > lhs->length) return -1;
    if (lhs->length == 1) {
        assert(lhs->capacity == 0);
        assert(rhs->capacity == 0);
        if (lhs->contents.value > rhs->contents.value) return 1;
        if (lhs->contents.value < rhs->contents.value) return -1;
        return 0;
    }
    return digit_cmp(lhs->contents.digits, rhs->contents.digits, lhs->length);
}

bool bigint_eq(const bigint* lhs, const bigint* rhs) {
    return bigint_cmp(lhs, rhs) == 0;
}

bool bigint_is_zero(const bigint* val) {
    return val->length == 1 && val->contents.value == 0;
}

bool bigint_is_one(const bigint* val) {
    return val->length == 1 && val->contents.value == 1;
}

bool bigint_is_odd(const bigint* val) {
    return val->length == 1 ? val->contents.value & 1
                            : val->contents.digits[val->length - 1] & 1;
}

char* bigint_to_string(const bigint* val) {
    if (val->length == 1 && val->capacity == 0) {
        int len = snprintf(NULL, 0, "%u", val->contents.value);
        char* str = malloc_safe((len + 1) * sizeof(char) + 1);
        snprintf(str, len + 1, "%u", val->contents.value);
        return str;
    }

    bigint n;
    bigint_clone(&n, val);

    size_t len = 0;
    char* str = malloc_safe(20 * val->length * sizeof(char));

    while (n.length > 0) {
        digit_t digit =
            digits_div(digits_ptr_mut(&n), digits_ptr_mut(&n), 10, n.length);
        str[len++] = '0' + digit;
        while (n.length > 0 && n.contents.digits[n.length - 1] == 0) {
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
    assert(val->length != 0);
    switch (val->length) {
    case 1:
        return (uint64_t)val->contents.value;
    case 2:
        return (uint64_t)val->contents.digits[0] +
               ((uint64_t)val->contents.digits[1] << 32);
    default:
        internal_panic("expected u64, but number is too large");
    }
}

bigint* bigint_gcd(const bigint* lhs, const bigint* rhs) {
    bigint* a = malloc_safe(sizeof(bigint));
    bigint b;
    if (bigint_cmp(lhs, rhs) == 1) {
        bigint_clone(a, lhs);
        bigint_clone(&b, rhs);
    } else {
        bigint_clone(a, rhs);
        bigint_clone(&b, lhs);
    }
    while (!bigint_is_zero(&b)) {
        bigint c;
        bigint_clone(&c, &b);
        bigint_rem(a, &b);
        bigint_destroy(&b);
        b = *a;
        *a = c;
    }
    bigint_destroy(&b);
    return a;
}

bigint* bigint_lcm(const bigint* lhs, const bigint* rhs) {
    bigint* lcm = malloc_safe(sizeof(bigint));
    if (bigint_is_zero(lhs) && bigint_is_zero(rhs)) {
        *lcm = bigint_zero;
        return lcm;
    }
    bigint* gcd = bigint_gcd(lhs, rhs);
    bigint_clone(lcm, lhs);
    bigint_div(lcm, gcd);
    bigint_mul(lcm, rhs);
    bigint_destroy(gcd);
    free(gcd);
    return lcm;
}
