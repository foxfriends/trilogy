#include "trilogy_string.h"
#include "internal.h"
#include "trilogy_array.h"
#include "trilogy_character.h"
#include "trilogy_value.h"
#include "types.h"
#include <assert.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

trilogy_string_value*
trilogy_string_init(trilogy_value* tv, trilogy_string_value* str) {
    assert(tv->tag == TAG_UNDEFINED);
    tv->tag = TAG_STRING;
    tv->payload = (uint64_t)str;
    return str;
}

trilogy_string_value*
trilogy_string_init_new(trilogy_value* tv, size_t len, char* s) {
    trilogy_string_value* str = malloc_safe(sizeof(trilogy_string_value));
    str->len = len;
    str->contents = malloc_safe(sizeof(char) * len);
    strncpy(str->contents, s, len);
    return trilogy_string_init(tv, str);
}

trilogy_string_value*
trilogy_string_clone_into(trilogy_value* tv, const trilogy_string_value* orig) {
    trilogy_string_value* str = malloc_safe(sizeof(trilogy_string_value));
    str->len = orig->len;
    str->contents = malloc_safe(sizeof(char) * orig->len);
    strncpy(str->contents, orig->contents, orig->len);
    return trilogy_string_init(tv, str);
}

trilogy_string_value* trilogy_string_init_from_c(trilogy_value* tv, char* s) {
    size_t len = (size_t)strlen(s);
    trilogy_string_value* str = malloc_safe(sizeof(trilogy_string_value));
    str->len = len;
    str->contents = malloc_safe(sizeof(char) * len);
    strncpy(str->contents, s, len);
    return trilogy_string_init(tv, str);
}

char* trilogy_string_as_c(trilogy_string_value* str) {
    char* ptr = malloc_safe(sizeof(char) * (str->len + 1));
    strncpy(ptr, str->contents, str->len);
    ptr[str->len] = '\0';
    return ptr;
}

size_t trilogy_string_len(trilogy_string_value* str) { return str->len; }

uint32_t trilogy_string_at(trilogy_string_value* str, size_t index) {
    assert(index < str->len);
    // TODO: properly support Unicode characters, instead of doing this on bytes
    return (uint32_t)str->contents[index];
}

int trilogy_string_compare(
    trilogy_string_value* lhs, trilogy_string_value* rhs
) {
    size_t len = lhs->len < rhs->len ? lhs->len : rhs->len;
    int cmp = strncmp(lhs->contents, rhs->contents, len);
    if (cmp != 0) return cmp;
    if (lhs->len < rhs->len) return -1;
    if (lhs->len > rhs->len) return 1;
    return 0;
}

trilogy_string_value* trilogy_string_untag(trilogy_value* val) {
    if (val->tag != TAG_STRING) rte("string", val->tag);
    return trilogy_string_assume(val);
}

trilogy_string_value* trilogy_string_assume(trilogy_value* val) {
    assert(val->tag == TAG_STRING);
    return (trilogy_string_value*)val->payload;
}

trilogy_string_value* trilogy_string_concat(
    trilogy_value* rt, trilogy_string_value* lhs, trilogy_string_value* rhs
) {
    size_t room = SIZE_MAX - lhs->len;
    if (rhs->len > room) internal_panic("string length limit\n");
    size_t len = lhs->len + rhs->len;
    char* ptr = malloc_safe(sizeof(char) * len);
    strncpy(ptr, lhs->contents, lhs->len);
    strncpy(ptr + lhs->len, rhs->contents, rhs->len);
    return trilogy_string_init_new(rt, len, ptr);
}

bool trilogy_string_unglue_start(
    trilogy_value* rt, trilogy_string_value* lhs, trilogy_string_value* rhs
) {
    if (rhs->len < lhs->len) return false;
    if (strncmp(lhs->contents, rhs->contents, lhs->len) != 0) return false;
    trilogy_string_init_new(rt, rhs->len - lhs->len, rhs->contents + lhs->len);
    return true;
}

bool trilogy_string_unglue_end(
    trilogy_value* rt, trilogy_string_value* lhs, trilogy_string_value* rhs
) {
    if (lhs->len < rhs->len) return false;
    size_t keep_len = lhs->len - rhs->len;
    if (strncmp(lhs->contents + keep_len, rhs->contents, rhs->len) != 0)
        return false;
    trilogy_string_init_new(rt, keep_len, lhs->contents);
    return true;
}

void trilogy_string_destroy(trilogy_string_value* val) { free(val->contents); }

void trilogy_string_to_array(trilogy_value* rt, trilogy_string_value* str) {
    trilogy_array_value* arr = trilogy_array_init_cap(rt, str->len);
    for (uint64_t i = 0; i < str->len; ++i) {
        trilogy_value val = trilogy_undefined;
        trilogy_character_init(&val, str->contents[i]);
        trilogy_array_push(arr, &val);
    }
}
