use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{
    Token,
    TokenType::{self, *},
};

/// An assignment statement. An assignment may have one of many "strategies".
///
/// [`FunctionAssignment`][] is parsed separately.
///
/// ```trilogy
/// lhs = rhs
/// ```
#[derive(Clone, Debug)]
pub struct AssignmentStatement {
    pub lhs: Expression,
    pub strategy: AssignmentStrategy,
    pub rhs: Expression,
    pub span: Span,
}

impl Spanned for AssignmentStatement {
    fn span(&self) -> Span {
        self.span
    }
}

impl AssignmentStatement {
    pub(crate) const ASSIGNMENT_OPERATOR: [TokenType; 24] = [
        OpEq,
        OpAmpAmpEq,
        OpPipePipeEq,
        OpAmpEq,
        OpPipeEq,
        OpCaretEq,
        OpShrEq,
        OpShlEq,
        OpShrExEq,
        OpShlExEq,
        OpShrConEq,
        OpShlConEq,
        OpGlueEq,
        OpPlusEq,
        OpMinusEq,
        OpStarEq,
        OpSlashEq,
        OpSlashSlashEq,
        OpPercentEq,
        OpStarStarEq,
        OpLtLtEq,
        OpGtGtEq,
        OpColonEq,
        OpDotEq,
    ];

    pub(crate) fn parse(parser: &mut Parser, lhs: Expression) -> SyntaxResult<Self> {
        let strategy = AssignmentStrategy::parse(parser)?;
        let rhs = Expression::parse(parser)?;
        Ok(Self {
            span: lhs.span().union(rhs.span()),
            lhs,
            strategy,
            rhs,
        })
    }
}

/// The strategy of an assignment statement.
#[derive(Clone, Debug, Spanned)]
pub enum AssignmentStrategy {
    Direct(Token),
    And(Token),
    Or(Token),
    Add(Token),
    Subtract(Token),
    Multiply(Token),
    Divide(Token),
    Remainder(Token),
    Power(Token),
    IntDivide(Token),
    BitwiseAnd(Token),
    BitwiseOr(Token),
    BitwiseXor(Token),
    LeftShift(Token),
    RightShift(Token),
    LeftShiftExtend(Token),
    RightShiftExtend(Token),
    LeftShiftContract(Token),
    RightShiftContract(Token),
    Glue(Token),
    Compose(Token),
    RCompose(Token),
    Access(Token),
    Cons(Token),
}

impl AssignmentStrategy {
    fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect(AssignmentStatement::ASSIGNMENT_OPERATOR)
            .map_err(|token| {
                parser.expected(token, "expected assignment operator (ending with `=`)")
            })?;
        Ok(match token.token_type {
            OpEq => Self::Direct(token),
            OpAmpAmpEq => Self::And(token),
            OpPipePipeEq => Self::Or(token),
            OpAmpEq => Self::BitwiseAnd(token),
            OpPipeEq => Self::BitwiseOr(token),
            OpCaretEq => Self::BitwiseXor(token),
            OpShlEq => Self::LeftShift(token),
            OpShrEq => Self::RightShift(token),
            OpShlExEq => Self::LeftShiftExtend(token),
            OpShrExEq => Self::RightShiftExtend(token),
            OpShlConEq => Self::LeftShiftContract(token),
            OpShrConEq => Self::RightShiftContract(token),
            OpGlueEq => Self::Glue(token),
            OpPlusEq => Self::Add(token),
            OpMinusEq => Self::Subtract(token),
            OpStarEq => Self::Multiply(token),
            OpSlashEq => Self::Divide(token),
            OpSlashSlashEq => Self::IntDivide(token),
            OpPercentEq => Self::Remainder(token),
            OpStarStarEq => Self::Power(token),
            OpLtLtEq => Self::RCompose(token),
            OpGtGtEq => Self::Compose(token),
            OpColonEq => Self::Cons(token),
            OpDotEq => Self::Access(token),
            _ => unreachable!(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(assignment_direct: "x = 5" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Direct(_), .. }));
    test_parse!(assignment_and: "x &&= 5" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::And(_), .. }));
    test_parse!(assignment_or: "x ||= 5" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Or(_), .. }));
    test_parse!(assignment_add: "x += 5" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Add(_), .. }));
    test_parse!(assignment_subtract: "x -= 5" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Subtract(_), .. }));
    test_parse!(assignment_multiply: "x *= 5" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Multiply(_), .. }));
    test_parse!(assignment_divide: "x /= 5" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Divide(_), .. }));
    test_parse!(assignment_remainder: "x %= 5" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Remainder(_), .. }));
    test_parse!(assignment_power: "x **= 5" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Power(_), .. }));
    test_parse!(assignment_int_divide: "x //= 5" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::IntDivide(_), .. }));
    test_parse!(assignment_bitwise_and: "x &= 5" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::BitwiseAnd(_), .. }));
    test_parse!(assignment_bitwise_or: "x |= 5" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::BitwiseOr(_), .. }));
    test_parse!(assignment_bitwise_xor: "x ^= 5" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::BitwiseXor(_), .. }));
    test_parse!(assignment_left_shift: "x <~= 5" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::LeftShift(_), .. }));
    test_parse!(assignment_right_shift: "x ~>= 5" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::RightShift(_), .. }));
    test_parse!(assignment_left_shift_ex: "x <~~= 5" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::LeftShiftExtend(_), .. }));
    test_parse!(assignment_right_shift_ex: "x ~~>= 5" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::RightShiftExtend(_), .. }));
    test_parse!(assignment_left_shift_con: "x <<~= 5" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::LeftShiftContract(_), .. }));
    test_parse!(assignment_right_shift_con: "x ~>>= 5" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::RightShiftContract(_), .. }));
    test_parse!(assignment_glue: "x <>= 5" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Glue(_), .. }));
    test_parse!(assignment_compose: "x >>= 5" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Compose(_), .. }));
    test_parse!(assignment_rcompose: "x <<= 5" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::RCompose(_), .. }));
    test_parse!(assignment_access: "x .= 5" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Access(_), .. }));
    test_parse!(assignment_cons: "x := 5" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Cons(_), .. }));

    test_parse_error!(assignment_not_fn: "a x = 7" => Statement::parse => "cannot assign to an expression that is not a valid assignment target");
    test_parse_error!(assignment_not_proc: "a!() = 7" => Statement::parse => "cannot assign to an expression that is not a valid assignment target");
    test_parse_error!(assignment_contains_not: "[a, a!()] = 7" => Statement::parse => "cannot assign to an expression that is not a valid assignment target");

    test_parse!(assignment_left_access: "a.b = 7" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Direct(_), .. }));
    test_parse!(assignment_proc_but_access: "a!().x = 7" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Direct(_), .. }));
    test_parse!(assignment_left_array: "[a, b, c] = [1, 2, 3]" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Direct(_), .. }));
    test_parse!(assignment_left_array_spread_start: "[..a, b, c] = []" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Direct(_), .. }));
    test_parse!(assignment_left_array_spread_middle: "[a, ..b, c] = []" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Direct(_), .. }));
    test_parse!(assignment_left_array_spread_end: "[a, b, ..c] = []" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Direct(_), .. }));
    test_parse_error!(assignment_left_array_spread_multi: "[..a, b, ..c] = []" => Statement::parse => "cannot assign to an expression that is not a valid assignment target");
    test_parse!(assignment_left_record: "{| \"a\" => a |} = {||}" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Direct(_), .. }));
    test_parse!(assignment_left_record_spread: "{| \"a\" => b, ..c |} = {||}" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Direct(_), .. }));
    test_parse_error!(assignment_left_record_spread_not_last: "{| ..c, \"a\" => b |} = {||}" => Statement::parse => "cannot assign to an expression that is not a valid assignment target");
    test_parse_error!(assignment_left_record_spread_multi: "{| ..a, ..c |} = {||}" => Statement::parse => "cannot assign to an expression that is not a valid assignment target");
    test_parse!(assignment_left_set: "[| a, b |] = [||]" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Direct(_), .. }));
    test_parse!(assignment_left_set_spread: "[| \"a\", ..c |] = [||]" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Direct(_), .. }));
    test_parse_error!(assignment_left_set_spread_not_last: "[| ..c, \"a\" |] = [||]" => Statement::parse => "cannot assign to an expression that is not a valid assignment target");
    test_parse_error!(assignment_left_set_spread_multi: "[| ..a, ..c |] = [||]" => Statement::parse => "cannot assign to an expression that is not a valid assignment target");
    test_parse!(assignment_left_glue: "\"hello \" <> world = 7" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Direct(_), .. }));
    test_parse!(assignment_left_neg: "-world = 7" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Direct(_), .. }));
    test_parse!(assignment_left_cons: "hello : world = 7" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Direct(_), .. }));
    test_parse!(assignment_left_struct: "'hello(x) = 7" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Direct(_), .. }));
    test_parse!(assignment_left_paren: "(x) = 7" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Direct(_), .. }));
    test_parse!(assignment_left_lit_false: "false = 7" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Direct(_), .. }));
    test_parse!(assignment_left_lit_true: "true = 7" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Direct(_), .. }));
    test_parse!(assignment_left_lit_unit: "unit = 7" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Direct(_), .. }));
    test_parse!(assignment_left_lit_atom: "'atom = 7" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Direct(_), .. }));
    test_parse!(assignment_left_lit_num: "7 = 7" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Direct(_), .. }));
    test_parse!(assignment_left_lit_char: "'7' = 7" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Direct(_), .. }));
    test_parse!(assignment_left_lit_str: "\"7\" = 7" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Direct(_), .. }));
    test_parse!(assignment_left_lit_bits: "0bb0101 = 7" => Statement::parse => Statement::Assignment(AssignmentStatement { strategy: AssignmentStrategy::Direct(_), .. }));

    test_parse_error!(assignment_left_block: "{ call!() } = 7" => Statement::parse);
    test_parse_error!(assignment_left_block_empty: "{} = 7" => Statement::parse);
    test_parse_error!(assignment_left_record_no_paren: "{ x => y } = 7" => Statement::parse);
}
