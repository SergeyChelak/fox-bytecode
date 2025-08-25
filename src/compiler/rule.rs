pub(crate) type ParseFn<T> = fn(&mut T, bool);

pub struct ParseRule<T> {
    pub(crate) prefix: Option<ParseFn<T>>,
    pub(crate) infix: Option<ParseFn<T>>,
    pub(crate) precedence: Precedence,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

impl Precedence {
    pub(crate) fn increased(&self) -> Self {
        use Precedence::*;
        match self {
            None => Assignment,
            Assignment => Or,
            Or => And,
            And => Equality,
            Equality => Comparison,
            Comparison => Term,
            Term => Factor,
            Factor => Unary,
            Unary => Call,
            Call => Primary,
            Primary => Primary, //unreachable!("undefined behavior by the book"),
        }
    }

    pub(crate) fn le(&self, other: &Self) -> bool {
        *self as u8 <= *other as u8
    }
}

impl<T> ParseRule<T> {
    pub(crate) fn new(
        prefix: Option<ParseFn<T>>,
        infix: Option<ParseFn<T>>,
        precedence: Precedence,
    ) -> Self {
        Self {
            prefix,
            infix,
            precedence,
        }
    }
}

impl<T> Default for ParseRule<T> {
    fn default() -> Self {
        Self {
            precedence: Precedence::None,
            prefix: Default::default(),
            infix: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn increase_less_equal() {
        use Precedence::*;
        let precedence = [
            None, Assignment, Or, And, Equality, Comparison, Term, Factor, Unary, Call, Primary,
        ];

        for (i, item) in precedence.iter().enumerate() {
            let next = item.increased();
            assert!(item.le(item));
            assert!(item.le(&next));
            let next_val = precedence.get(i + 1).unwrap_or(&Precedence::Primary);
            assert_eq!(next, *next_val);
        }
    }
}
