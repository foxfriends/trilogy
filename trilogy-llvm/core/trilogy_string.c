#include "trilogy_string.h"
#include "internal.h"
#include <assert.h>
#include <stdlib.h>
#include <string.h>

trilogy_string_value*
trilogy_string_init(trilogy_value* tv, trilogy_string_value* str) {
    assert(tv->tag == TAG_UNDEFINED);
    tv->tag = TAG_STRING;
    tv->payload = (unsigned long)str;
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
trilogy_string_clone_into(trilogy_value* tv, trilogy_string_value* orig) {
    trilogy_string_value* str = malloc_safe(sizeof(trilogy_string_value));
    str->len = orig->len;
    str->contents = malloc_safe(sizeof(char) * orig->len);
    strncpy(str->contents, orig->contents, orig->len);
    return trilogy_string_init(tv, str);
}

trilogy_string_value* trilogy_string_init_from_c(trilogy_value* tv, char* s) {
    unsigned long len = (unsigned long)strlen(s);
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

unsigned long trilogy_string_len(trilogy_string_value* str) { return str->len; }

unsigned int trilogy_string_at(trilogy_string_value* str, unsigned long index) {
    assert(index < str->len);
    // TODO: properly support Unicode characters, instead of doing this on bytes
    return (unsigned int)str->contents[index];
}

trilogy_string_value* trilogy_string_untag(trilogy_value* val) {
    if (val->tag != TAG_STRING) rte("string", val->tag);
    return trilogy_string_assume(val);
}

trilogy_string_value* trilogy_string_assume(trilogy_value* val) {
    assert(val->tag == TAG_STRING);
    return (trilogy_string_value*)val->payload;
}

void trilogy_string_destroy(trilogy_string_value* val) { free(val->contents); }
