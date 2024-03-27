use crate::context::Context;
use crate::prelude::*;
use std::collections::HashSet;
use trilogy_ir::ir;
use trilogy_ir::visitor::{HasBindings, HasCanEvaluate, HasReferences};
use trilogy_vm::{Instruction, Offset};

pub(crate) fn write_rule(context: &mut Context, rule: &ir::Rule, on_fail: &str) {
    let setup = context.make_label("setup_rule");
    let end = context.make_label("end");
    let precall = context.make_label("precall");
    let call = context.make_label("call");

    // On calling a rule, check if the state is `unit`. If it is, that means it's the
    // first time into this branch of the rule, so we have to run setup.
    context
        .instruction(Instruction::Copy)
        .instruction(Instruction::Unit)
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
    context.instruction(Instruction::SetRegister(TEMPORARY));
    // First check all the parameters, make sure they work. If they don't
    // match, we can fail without even constructing the state.
    let cleanup = context.make_label("cleanup");
    // Track how many variables get declared in the parameters so that they
    // can properly be cleaned up at the end. These parameters only need to
    // exist inside the closure, and must not exist on stack outside the closure.
    let all_bindings = rule
        .parameters
        .iter()
        .flat_map(|param| param.bindings())
        .collect::<HashSet<_>>();
    // Variables of every binding are declared up front.
    let mut total_declared = context.declare_variables(all_bindings.clone());
    for (i, parameter) in rule.parameters.iter().enumerate() {
        let skip = context.make_label("skip");
        // Then we only set those bindings if the parameter was passed.
        context
            .instruction(Instruction::IsSetLocal(1 + i as Offset))
            .cond_jump(&skip);
        // Parameter *was* passed, so update the bindset and the bindings together.
        context.instruction(Instruction::LoadRegister(TEMPORARY));
        for var in parameter.bindings() {
            let index = context.scope.lookup(&var).unwrap().unwrap_local();
            context.constant(index).instruction(Instruction::Insert);
        }
        context.intermediate(); // bindset
        context
            .instruction(Instruction::LoadLocal(1 + i as Offset))
            .pattern_match(parameter, &cleanup)
            .end_intermediate() // bindset
            .instruction(Instruction::SetRegister(TEMPORARY))
            .label(skip);
    }
    // Happy path: we continue by writing the query state down, and then
    // encapsulating all that into a closure which will be the iterator for
    // this overload of the rule.
    total_declared += context.declare_variables(rule.body.bindings());
    // Put the final bindset down here. Register 3 no longer matters after
    // this.
    let actual_state = context
        .instruction(Instruction::LoadRegister(TEMPORARY))
        .extend_query_state(&rule.body)
        .intermediate();
    context.close(&precall);

    // The actual body of the rule involves running the query, then
    // returning the return value in 'next. We convert failure to
    // returning 'done, as in a regular iterator.
    let on_done = context.make_label("on_done");
    let start = context.make_label("start");
    context
        .label(&start)
        .instruction(Instruction::LoadLocal(actual_state))
        .execute_query(&rule.body, &on_done)
        .instruction(Instruction::SetLocal(actual_state))
        .end_intermediate();
    // The query is normal, then the value is computed by evaluating
    // the parameter patterns now as expressions.
    context.scope.intermediate(); // At this point, the query state is an intermediate

    // Every parameter must be fully evaluated in order for a rule to return those values.
    // This is probably a limitation right now, and maybe can be relaxed allowing returning
    // still-unbound variables, but that is unecessary right now.
    //
    // Instead, just ensure that every variable referenced in every parameter is bound before
    // returning. If any are not, then call the rule again.
    for id in rule
        .parameters
        .iter()
        .flat_map(|param| param.references())
        .collect::<HashSet<_>>()
    {
        if let Binding::Variable(index) = context.scope.lookup(&id).unwrap() {
            context
                .instruction(Instruction::IsSetLocal(index))
                .cond_jump(&start);
        }
    }
    // Stack these up in reverse so that when the caller starts pattern matching they are
    // doing it left to right, as expected.
    for (i, param) in rule.parameters.iter().enumerate().rev() {
        let eval = context.make_label("eval");
        let next = context.make_label("next");
        context
            .instruction(Instruction::IsSetLocal(1 + i as Offset))
            // Previously unset parameters get evaluated into
            .cond_jump(&eval)
            // Previously set parameters are just loaded back up directly
            .instruction(Instruction::LoadLocal(1 + i as Offset))
            .jump(&next);
        context.label(eval);
        if param.can_evaluate() {
            context.evaluate(param);
        } else {
            context.instruction(Instruction::Fizzle);
        }
        context.scope.intermediate(); // As is each subsequent parameter value
        context.label(next);
    }
    // The return value is a (backwards) list
    context.instruction(Instruction::Unit);
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
    context.label(&cleanup);
    context.undeclare_variables(all_bindings, true);
    context
        .instruction(Instruction::Pop) // The bindset is still on the stack during cleanup!
        .jump(on_fail)
        // Following setup, we go here, clearing all the closed up state that was put
        // on the stack before going to `call`.
        //
        // That state is the query state + all the query's parameters + all the query's body variables
        .label(&precall)
        .instruction(Instruction::Slide(total_declared as Offset + 1))
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
