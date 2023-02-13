use super::{Identifier, *};
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct Lookup {
    pub path: Path,
    pub patterns: Vec<Pattern>,
    end: Token,
}

impl Lookup {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let mut segments = vec![];
        loop {
            let identifier = Identifier::parse(parser)?;

            enum Argument {
                Module(ModuleReference),
                Pattern(Pattern),
            }

            if let Ok(start) = parser.expect(OParen) {
                // We don't know if this is the last segment of the path or not yet.
                // Each identifier with arguments may be a module reference or the
                // lookup itself, so we parse it as if it could be either. If all
                // patterns are module references and we eventually find the next path
                // segment, then it's a module reference, otherwise it's the lookup
                // and we can coerce any identifiers into patterns.
                let mut arguments = vec![];
                let end = loop {
                    if let Ok(end) = parser.expect(CParen) {
                        break end;
                    }
                    arguments.push(match ModuleReference::parse_or_pattern(parser)? {
                        Ok(module_reference) => Argument::Module(module_reference),
                        Err(pattern) => Argument::Pattern(pattern),
                    });
                    if parser.expect(OpComma).is_ok() {
                        continue;
                    }
                    break parser.expect(CParen).map_err(|token| {
                        parser.expected(token, "expected `,` or `)` in argument list")
                    })?;
                };
                if arguments
                    .iter()
                    .all(|arg| matches!(arg, Argument::Module(..)))
                    && parser.expect(OpColonColon).is_ok()
                {
                    segments.push(ModuleReference::new(
                        identifier,
                        Some(ModuleArguments::new(
                            start,
                            arguments
                                .into_iter()
                                .map(|arg| match arg {
                                    Argument::Module(module) => module,
                                    _ => unreachable!(),
                                })
                                .collect(),
                            end,
                        )),
                    ));
                } else {
                    let patterns = arguments
                        .into_iter()
                        .map(|arg| match arg {
                            Argument::Module(module) => Pattern::try_from(module),
                            Argument::Pattern(pattern) => Ok(pattern),
                        })
                        .filter_map(|pattern| match pattern {
                            Ok(pattern) => Some(pattern),
                            Err(error) => {
                                parser.error(error);
                                None
                            }
                        })
                        .collect();
                    return Ok(Self {
                        path: Path::new(segments, identifier),
                        patterns,
                        end,
                    });
                }
            } else {
                segments.push(ModuleReference::new(identifier, None));
                parser.expect(OpColonColon).map_err(|token| {
                    parser.expected(
                        token,
                        "expected `::` to continue path, path must end with a rule lookup",
                    )
                })?;
            }
        }
    }
}
