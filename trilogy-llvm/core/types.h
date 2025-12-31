#pragma once
#include "rational.h"
#include <stddef.h>
#include <stdint.h>

typedef enum trilogy_value_tag : uint8_t {
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
     * I guess module will be a runtime type, since no avoiding one being
     * referenced dynamically, and they follow different rules than other
     * values.
     */
    TAG_MODULE = 14,
    /**
     * Not an observable value in a Trilogy program, but the reference counted
     * reference to a heap allocated variable is a distinguished type at runtime
     * level...
     */
    TAG_REFERENCE = 15
} trilogy_value_tag;

typedef enum trilogy_callable_tag : uint8_t {
    CALLABLE_FUNCTION = 1,
    CALLABLE_RULE = 2,
    CALLABLE_CONTINUATION = 3,
} trilogy_callable_tag;

typedef struct trilogy_value {
    trilogy_value_tag tag;
    /**
     * The payload is held in `uint64_t` to be a 64 bit container for
     * anything. The actual value type of the payload depends on the tag.
     *
     * We do not use a union because the field should be left-padded, which
     * would not be the default in a union, but does happen correctly if we cast
     * the payload around manually.
     */
    uint64_t payload;
} trilogy_value;

typedef struct trilogy_number_value {
    rational re;
    rational im;
} trilogy_number_value;

typedef struct trilogy_string_value {
    /**
     * The number of bytes in the string.
     */
    size_t len;
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
    size_t len;
    /**
     * The raw bytes of the bits value.
     *
     * This is a bytearray of `len/8` bytes. The value of any excess padding
     * bits is undefined.
     */
    uint8_t* contents;
} trilogy_bits_value;

typedef struct trilogy_struct_value {
    /**
     * The unwrapped atom ID that tags this struct.
     */
    uint64_t atom;
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
    uint32_t rc;
    /**
     * The number of elements in this array.
     */
    size_t len;
    /**
     * The capacity of this array; values in cells past the len contain
     * unspecified data.
     */
    size_t cap;
    /**
     * An array of length `cap` containing the values of this array.
     */
    trilogy_value* contents;
} trilogy_array_value;

typedef struct trilogy_set_value {
    /**
     * The reference count for this set.
     */
    uint32_t rc;
    /**
     * The number of elements in this set.
     */
    size_t len;
    /**
     * The capacity of this set; values in cells past the len contain
     * unspecified data.
     */
    size_t cap;
    /**
     * An array of length `cap` containing the values of this set. This is a
     * tuple, same as record as a set is just a record with `unit` for every
     * value.
     */
    trilogy_tuple_value* contents;
} trilogy_set_value;

typedef struct trilogy_record_value {
    /**
     * The reference count for this record.
     */
    uint32_t rc;
    /**
     * The number of elements in this record.
     */
    size_t len;
    /**
     * The capacity of this record; values in cells past the len contain
     * unspecified data.
     */
    size_t cap;
    /**
     * An array of length `cap` containing the key-value pairs of this record.
     */
    trilogy_tuple_value* contents;
} trilogy_record_value;

typedef struct source_pos {
    size_t line;
    size_t column;
} source_pos;

typedef struct source_span {
    source_pos start;
    source_pos end;
} source_span;

/**
 * Static metadata about each callable.
 */
typedef struct trilogy_callable_data {
    const char* name;
    const char* path;
    source_span span;
    const struct trilogy_callable_data* parent;
} trilogy_callable_data;

typedef struct trilogy_callable_value {
    /**
     * The reference count for this callable.
     */
    uint32_t rc;
    /**
     * Determines which type of call this callable requires
     */
    trilogy_callable_tag tag;
    /**
     * The number of parameters to this callable. Functions must have arity 1.
     * Procedures may have any arity. Continuations arity is not checked, but
     * in most cases (and any case that the end programmer can get ahold of it
     * directly) is 1.
     */
    uint32_t arity;
    /**
     * For captured continuations, the possible exit directions are captured.
     **/
    struct trilogy_callable_value* return_to;
    struct trilogy_callable_value* yield_to;
    struct trilogy_callable_value* end_to;
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
    const trilogy_callable_data* metadata;
} trilogy_callable_value;

/**
 * A shared variable reference, represented as an upvalue.
 */
typedef struct trilogy_reference {
    /**
     * The reference count for this reference.
     */
    uint32_t rc;
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

/**
 * Static metadata about each module.
 */
typedef struct trilogy_module_data {
    // const char* name;
    // const char* path;
    size_t len;
    uint64_t* member_ids;
    uint8_t* member_exports;
    void** members;
} trilogy_module_data;

/**
 * A module instance.
 */
typedef struct trilogy_module {
    /**
     * The reference count for this module instance.
     */
    uint32_t rc;
    /**
     * Pointer to the module data table
     **/
    const trilogy_module_data* module_data;
    /**
     * The closure containing the module parameters and storage for constants.
     */
    trilogy_array_value* closure;
} trilogy_module;

/**
 * The foreign object works/appears as a module in Trilogy, but internally is
 * backed by some native stuff instead. The contents takes the place of the
 * closure, and is received as the last parameter when calling methods.
 *
 * RAII is not a thing, the contents are possibly unclosed resources,
 * the Trilogy code should correctly open/close them if needed.
 */
typedef struct trilogy_foreign_object {
    /**
     * The reference count for this foreign object instance.
     */
    uint32_t rc;
    /**
     * The number of members in this foreign object.
     */
    size_t len;
    /**
     * The sorted list of IDs for members in this foreign object.
     */
    uint64_t* member_ids;
    /**
     * The actual pointers to contained member accessors.
     */
    void** members;
    /**
     * Pointer to the contained foreign object.
     */
    void* contents;
} trilogy_foreign_object;

char* type_name(trilogy_value_tag tag);
