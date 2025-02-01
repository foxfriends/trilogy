#pragma once

typedef enum trilogy_value_tag : unsigned char {
    TAG_UNDEFINED = 0,
    TAG_UNIT = 1,
    TAG_BOOL = 2,
    TAG_ATOM = 3,
    TAG_CHAR = 4,
    TAG_STRING = 5,
    TAG_NUMBER = 6,
    TAG_BITS = 7,
    TAG_STRUCT = 8,
    TAG_TUPLE = 9,
    TAG_ARRAY = 10,
    TAG_SET = 11,
    TAG_RECORD = 12,
    TAG_CALLABLE = 13,
    /**
     * Not an observable value in a Trilogy program, but the reference counted
     * reference to a heap allocated variable is a distinguished type at runtime
     * level...
     */
    TAG_REFERENCE = 14
} trilogy_value_tag;

typedef enum trilogy_callable_tag : unsigned char {
    CALLABLE_FUNCTION = 1,
    CALLABLE_PROCEDURE = 2,
    CALLABLE_RULE = 3,
    CALLABLE_CONTINUATION = 4
} trilogy_callable_tag;

typedef struct trilogy_value {
    trilogy_value_tag tag;
    /**
     * The payload is held in `unsigned long` to be a 64 bit container for
     * anything. The actual value type of the payload depends on the tag.
     *
     * We do not use a union because the field should be left-padded, which
     * would not be the default in a union, but does happen correctly if we cast
     * the payload around manually.
     */
    unsigned long payload;
} trilogy_value;

typedef struct trilogy_string_value {
    /**
     * The number of bytes in the string.
     */
    unsigned long len;
    /**
     * The raw byte contents of this string. This data is ASSUMED to be UTF-8,
     * and is not null terminated.
     */
    char* contents;
} trilogy_string_value;

typedef struct trilogy_bits_value {
    /**
     * The number of relevant bits in this value.
     */
    unsigned long len;
    /**
     * The raw bytes of the bits value.
     *
     * This is a bytearray of `len/8` bytes. The value of any excess padding
     * bits is undefined.
     */
    unsigned char* contents;
} trilogy_bits_value;

typedef struct trilogy_struct_value {
    /**
     * The unwrapped atom ID that tags this struct.
     */
    unsigned long atom;
    /**
     * The value of this struct.
     */
    trilogy_value contents;
} trilogy_struct_value;

typedef struct trilogy_tuple_value {
    /**
     * The first value of this tuple.
     */
    trilogy_value fst;
    /**
     * The second value of this tuple.
     */
    trilogy_value snd;
} trilogy_tuple_value;

typedef struct trilogy_array_value {
    /**
     * The reference count for this array.
     */
    unsigned int rc;
    /**
     * The number of elements in this array.
     */
    unsigned long len;
    /**
     * The capacity of this array; values in cells past the len contain
     * unspecified data.
     */
    unsigned long cap;
    /**
     * An array of length `cap` containing the values of this array.
     */
    trilogy_value* contents;
} trilogy_array_value;

typedef struct trilogy_set_value {
    /**
     * The reference count for this set.
     */
    unsigned int rc;
    /**
     * The number of elements in this set.
     */
    unsigned long len;
    /**
     * The capacity of this set; values in cells past the len contain
     * unspecified data.
     */
    unsigned long cap;
    /**
     * An array of length `cap` containing the values of this set.
     */
    trilogy_value* contents;
} trilogy_set_value;

typedef struct trilogy_record_value {
    /**
     * The reference count for this record.
     */
    unsigned int rc;
    /**
     * The number of elements in this record.
     */
    unsigned long len;
    /**
     * The capacity of this record; values in cells past the len contain
     * unspecified data.
     */
    unsigned long cap;
    /**
     * An array of length `cap` containing the key-value pairs of this record.
     */
    trilogy_tuple_value* contents;
} trilogy_record_value;

typedef struct trilogy_callable_value {
    /**
     * The reference count for this callable.
     */
    unsigned int rc;
    /**
     * Determines which type of call this callable requires
     */
    trilogy_callable_tag tag;
    /**
     * The number of parameters to this callable. Functions must have arity 1.
     * Other types may have any arity.
     */
    unsigned int arity;
    /**
     * For captured continuations, the return and yield points are stored rather
     * than provided. (The `end` pointer is still provided)
     **/
    struct trilogy_callable_value* return_to;
    struct trilogy_callable_value* yield_to;
    /**
     * Context captured from the closure of this callable. This is an array of
     * trilogy values (all of which would should be references?). The array is
     * owned by the callable, and uses the array struct mostly as a convenience.
     *
     * The identity and population of each field is a static analysis concern.
     *
     * NOTE: there is the inherent risk of circular references here,
     * which should likely be solved weak references of some sort...
     */
    trilogy_array_value* closure;
    /**
     * Pointer to the function itself.
     */
    void* function;
} trilogy_callable_value;

/**
 * A shared variable reference, represented as an upvalue.
 */
typedef struct trilogy_reference {
    /**
     * The reference count for this reference.
     */
    unsigned int rc;
    /**
     * Pointer to the location of the variable. Will be pointing to the value
     * field if the referenced value has been moved to the heap, or to the
     * original stack location if not.
     */
    trilogy_value* location;
    /**
     * The actual value of this variable, if it is on the heap. Will be
     * undefined if the value remains on the stack.
     */
    trilogy_value closed;
} trilogy_reference;

char* type_name(trilogy_value_tag tag);
