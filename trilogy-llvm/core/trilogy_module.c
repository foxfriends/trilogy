#include "trilogy_module.h"
#include "internal.h"
#include "trilogy_array.h"
#include "trilogy_callable.h"
#include "trilogy_value.h"
#include "types.h"
#include <assert.h>
#include <stdint.h>
#include <stdlib.h>

trilogy_module* trilogy_module_init(trilogy_value* tv, trilogy_module* module) {
    tv->tag = TAG_MODULE;
    tv->payload = (uint64_t)module;
    return module;
}

trilogy_module*
trilogy_module_init_new(trilogy_value* tv, trilogy_module_data* module_data) {
    trilogy_module* module = malloc_safe(sizeof(trilogy_module));
    module->rc = 1;
    module->module_data = module_data;
    module->closure = NO_CLOSURE;
    return trilogy_module_init(tv, module);
}

trilogy_module* trilogy_module_init_new_closure(
    trilogy_value* tv, trilogy_module_data* module_data, trilogy_value* closure
) {
    assert(closure->tag == TAG_ARRAY);
    trilogy_module* module = malloc_safe(sizeof(trilogy_module));
    module->rc = 1;
    module->module_data = module_data;
    module->closure = trilogy_array_assume(closure);
    return trilogy_module_init(tv, module);
}

trilogy_module*
trilogy_module_clone_into(trilogy_value* tv, trilogy_module* module) {
    assert(module->rc != 0);
    module->rc++;
    return trilogy_module_init(tv, module);
}

trilogy_module* trilogy_module_untag(trilogy_value* val) {
    if (val->tag != TAG_MODULE) rte("module", val->tag);
    return trilogy_module_assume(val);
}

trilogy_module* trilogy_module_assume(trilogy_value* val) {
    assert(val->tag == TAG_MODULE);
    return (trilogy_module*)val->payload;
}

void trilogy_module_destroy(trilogy_module* module) {
    if (--module->rc == 0) {
        // NOTE: module->member_data is not destroyed because it is a global
        // constant.
        trilogy_array_destroy(module->closure);
        free(module);
    }
}

typedef trilogy_value* (*accessor)(trilogy_value*);
typedef trilogy_value* (*closure_accessor)(trilogy_value*, trilogy_value*);

void trilogy_module_find(
    trilogy_value* tv, trilogy_module* module, uint64_t id
) {
    trilogy_module_data* module_data = module->module_data;
    // NOTE: modules are typically quite small, so linear search is usually
    // going to be just fine, but if someone makes a pathological module we
    // might do much better to binary search this.
    for (size_t i = 0; i < module_data->len; ++i) {
        if (module_data->member_ids[i] == id) {
            size_t byte_index = i / 8;
            uint8_t bit_index = i % 8;
            uint8_t is_exported =
                module_data->member_exports[byte_index] & (1 << bit_index);
            if (!is_exported) break;
            if (module->closure == NO_CLOSURE) {
                ((accessor)module_data->members[i])(tv);
            } else {
                trilogy_value* closure = malloc_safe(sizeof(trilogy_value));
                *closure = trilogy_undefined;
                trilogy_array_clone_into(closure, module->closure);
                ((closure_accessor)module_data->members[i])(tv, closure);
            }
            return;
        }
    }
    // TODO: consider that this maybe should not be a panic...
    return internal_panic("module does not contain requested member\n");
}
