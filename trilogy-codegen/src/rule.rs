use crate::context::Context;
use crate::prelude::*;
use std::collections::HashSet;
use trilogy_ir::ir;
use trilogy_ir::visitor::{HasBindings, HasCanEvaluate};
use trilogy_vm::Instruction;

pub(crate) fn write_rule(context: &mut Context, rule: &ir::Rule, on_fail: &str) {
    let setup = context.labeler.unique_hint("setup");
    let end = context.labeler.unique_hint("end");
    let precall = context.labeler.unique_hint("precall");
    let call = context.labeler.unique_hint("call");

    // On calling a rule, check if the state is `unit`. If it is, that means it's the
    // first time into this branch of the rule, so we have to run setup.
    context
        .instruction(Instruction::Copy)
        .constant(())
        .instruction(Instruction::ValNeq)
        .cond_jump(&setup)
        // Once setup is complete, we end up here where the rule's iterator is actually
        // called. The state (that is not unit) is that iterator.
        .label(&call)
        .instruction(Instruction::Copy)
        .instruction(Instruction::Call(0))
        // After calling this overload's iterator, flatten failures into the parent.
        // The parent expects this to work more like a query than an iterator for
        // failures.
        .instruction(Instruction::Copy)
        .atom("done")
        .instruction(Instruction::ValEq)
        .cond_jump(&end)
        .instruction(Instruction::Pop) // The 'done
        .instruction(Instruction::Pop) // The closure itself
        .jump(on_fail);

    // Begin setup by throwing away the `unit` placeholder state
    context.label(setup).instruction(Instruction::Pop);
    // Set up the initial bindset for this rule. It starts empty, but gets
    // filled right away from the parameters.
    context.constant(HashSet::new());
    // Just using temp register to help keep the bindset on top of stack.
    // Probably could have done this any number of ways... but temp
    // register is easy
    context.instruction(Instruction::SetRegister(3));
    // First check all the parameters, make sure they work. If they don't
    // match, we can fail without even constructing the state.
    let mut cleanup = vec![];
    // Track how many variables get declared in the parameters so that they
    // can properly be cleaned up at the end. These parameters only need to
    // exist inside the closure, and must not exist on stack outside the closure.
    let mut total_declared = 0;
    for (i, parameter) in rule.parameters.iter().enumerate() {
        let skip = context.labeler.unique_hint("skip");
        cleanup.push(context.labeler.unique_hint("cleanup"));
        // Variables of this binding must be declared, whether they are about to
        // be set or not.
        total_declared += context.declare_variables(parameter.bindings());
        // Then we only set those bindings if the parameter was passed.
        context
            .instruction(Instruction::IsSetLocal(1 + i as u32))
            .cond_jump(&skip);
        // Parameter *was* passed, so update the bindset and the bindings together.
        context.instruction(Instruction::LoadRegister(3));
        for var in parameter.bindings() {
            let index = context.scope.lookup(&var).unwrap().unwrap_local();
            context.constant(index).instruction(Instruction::Insert);
        }
        context.scope.intermediate(); // Bindset
        context.instruction(Instruction::LoadLocal(1 + i as u32));
        write_pattern_match(context, parameter, &cleanup[i]);
        context.scope.end_intermediate();
        context.instruction(Instruction::SetRegister(3));
        context.label(skip);
    }
    // Happy path: we continue by writing the query state down, and then
    // encapsulating all that into a closure which will be the iterator for
    // this overload of the rule.
    total_declared += context.declare_variables(rule.body.bindings());
    // Put the final bindset down here. Register 3 no longer matters after
    // this.
    context.instruction(Instruction::LoadRegister(3));
    write_continued_query_state(context, &rule.body);
    let actual_state = context.scope.intermediate();
    context.close(&precall);

    // The actual body of the rule involves running the query, then
    // returning the return value in 'next. We convert failure to
    // returning 'done, as in a regular iterator.
    let on_done = context.labeler.unique_hint("on_done");
    context.instruction(Instruction::LoadLocal(actual_state));
    write_query(context, &rule.body, &on_done);
    context.instruction(Instruction::SetLocal(actual_state));
    context.scope.end_intermediate();
    // The query is normal, then the value is computed by evaluating
    // the parameter patterns now as expressions.
    context.scope.intermediate(); // At this point, the query state is an intermediate
    for (i, param) in rule.parameters.iter().enumerate() {
        let eval = context.labeler.unique_hint("eval");
        let next = context.labeler.unique_hint("next");
        context
            .instruction(Instruction::IsSetLocal(1 + i as u32))
            // Previously unset parameters get evaluated into
            .cond_jump(&eval)
            // Previously set parameters are just loaded back up directly
            .instruction(Instruction::LoadLocal(1 + i as u32))
            .jump(&next);
        context.label(eval);
        if param.can_evaluate() {
            write_expression(context, param);
        } else {
            context.instruction(Instruction::Fizzle);
        }
        context.scope.intermediate(); // As is each subsequent parameter value
        context.label(next);
    }
    // The return value is a (backwards) list
    context.constant(());
    for _ in &rule.parameters {
        context
            .instruction(Instruction::Swap)
            .instruction(Instruction::Cons);
        context.scope.end_intermediate();
    }
    // Finally, put the return value into 'next()
    context
        .atom("next")
        .instruction(Instruction::Construct)
        .instruction(Instruction::Return);
    // This ends with 2 expected values on the stack ([state, retval])
    // so they are no longer intemediate
    context.scope.end_intermediate();

    // On failure, just return 'done
    context
        .label(on_done)
        .atom("done")
        .instruction(Instruction::Return);

    // Sad path: we have to undeclare all the variables that have been
    // declared so far, then fail as regular.
    for parameter in rule.parameters.iter().rev() {
        context.label(cleanup.pop().unwrap());
        context.undeclare_variables(parameter.bindings(), true);
    }
    context
        .instruction(Instruction::Pop) // The bindset is still on the stack during cleanup!
        .jump(on_fail)
        // Following setup, we go here, clearing all the closed up state that was put
        // on the stack before going to `call`.
        //
        // That state is the query state + all the query's parameters + all the query's body variables
        .label(&precall)
        .instruction(Instruction::Slide(total_declared as u32 + 1))
        .instruction(Instruction::Pop);
    for _ in 0..total_declared {
        // The query's parameters and body variables are popped manually,
        // as the parameters were already undeclared above, so using
        // undeclare on those is not going to work. :shrug:
        context.instruction(Instruction::Pop);
    }
    context.undeclare_variables(rule.body.bindings(), false);
    context.jump(call).label(end);
}
