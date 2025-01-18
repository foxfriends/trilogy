#include <stdlib.h>
#include <string.h>
#include "trilogy_string.h"
#include "internal.h"

trilogy_value trilogy_string_new(size_t len, char* s) {
    trilogy_string_value* str = malloc(sizeof(trilogy_string_value));
    str->len = len;
    str->contents = malloc(sizeof(char) * len);
    strncpy(str->contents, s, len);
    trilogy_value t = { .tag = TAG_STRING, .payload = (unsigned long)str };
    return t;
}

trilogy_value trilogy_string_clone(trilogy_string_value* orig) {
    trilogy_string_value* str = malloc(sizeof(trilogy_string_value));
    str->len = orig->len;
    str->contents = malloc(sizeof(char) * orig->len);
    strncpy(str->contents, orig->contents, orig->len);
    trilogy_value t = { .tag = TAG_STRING, .payload = (unsigned long)str };
    return t;
}

trilogy_value trilogy_string_from_c(char* s) {
    unsigned long len = (unsigned long)strlen(s);
    trilogy_string_value* str = malloc(sizeof(trilogy_string_value));
    str->len = len;
    str->contents = malloc(sizeof(char) * len);
    strncpy(str->contents, s, len);
    trilogy_value t = { .tag = TAG_STRING, .payload = (unsigned long)str };
    return t;
}

char* trilogy_string_to_c(trilogy_string_value* str) {
    char* ptr = malloc(sizeof(char) * (str->len + 1));
    strncpy(ptr, str->contents, str->len);
    ptr[str->len] = '\0';
    return ptr;
}

trilogy_string_value* trilogy_string_untag(trilogy_value* val) {
    if (val->tag != TAG_STRING) rte("string", val->tag);
    return trilogy_string_assume(val);
}

trilogy_string_value* trilogy_string_assume(trilogy_value* val) {
    return (trilogy_string_value*)val->payload;
}

void trilogy_string_destroy(trilogy_string_value* val) {
    free(val->contents);
}
