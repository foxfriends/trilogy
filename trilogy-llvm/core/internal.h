#pragma once
#include <stdbool.h>
#include "types.h"

void panic(char* msg);
char* type_name(unsigned char tag);
char* tocstr(struct trilogy_value* val);
void rte(char* expected, unsigned char tag);

void untag_unit(struct trilogy_value* val);
bool untag_bool(struct trilogy_value* val);
unsigned long untag_atom(struct trilogy_value* val);
unsigned int untag_char(struct trilogy_value* val);
struct trilogy_string_value* untag_string(struct trilogy_value* val);
long untag_integer(struct trilogy_value* val);
struct trilogy_bits_value* untag_bits(struct trilogy_value* val);
struct trilogy_struct_value* untag_struct(struct trilogy_value* val);
struct trilogy_tuple_value* untag_tuple(struct trilogy_value* val);
struct trilogy_array_value* untag_array(struct trilogy_value* val);
struct trilogy_set_value* untag_set(struct trilogy_value* val);
struct trilogy_record_value* untag_record(struct trilogy_value* val);
void* untag_callable(struct trilogy_value* val);
