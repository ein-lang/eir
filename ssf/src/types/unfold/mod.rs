mod type_unfolder;

use crate::types::*;
use type_unfolder::TypeUnfolder;

pub(crate) fn unfold(record: &Record) -> Record {
    Record::new(
        record
            .elements()
            .iter()
            .map(|type_| canonicalize(&TypeUnfolder::new(record).unfold(type_)))
            .collect(),
        record.is_boxed(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn unfold_() {
        for (type_, unfolded_type) in &[
            (
                Algebraic::new(vec![Constructor::boxed(vec![Type::Index(0)])]),
                Algebraic::new(vec![Constructor::boxed(vec![Algebraic::new(vec![
                    Constructor::boxed(vec![Type::Index(0)]),
                ])
                .into()])]),
            ),
            (
                Algebraic::new(vec![Constructor::unboxed(vec![Algebraic::new(vec![
                    Constructor::boxed(vec![Type::Index(1)]),
                ])
                .into()])]),
                Algebraic::new(vec![Constructor::unboxed(vec![Algebraic::new(vec![
                    Constructor::boxed(vec![Algebraic::new(vec![Constructor::unboxed(vec![
                        Type::Index(1),
                    ])])
                    .into()]),
                ])
                .into()])]),
            ),
            (
                Algebraic::new(vec![Constructor::boxed(vec![Algebraic::new(vec![
                    Constructor::unboxed(vec![]),
                    Constructor::unboxed(vec![Algebraic::new(vec![Constructor::boxed(vec![
                        Type::Index(2),
                    ])])
                    .into()]),
                ])
                .into()])]),
                Algebraic::new(vec![Constructor::boxed(vec![Algebraic::new(vec![
                    Constructor::unboxed(vec![]),
                    Constructor::unboxed(vec![Algebraic::new(vec![Constructor::boxed(vec![
                        Algebraic::new(vec![Constructor::boxed(vec![Type::Index(2)])]).into(),
                    ])])
                    .into()]),
                ])
                .into()])]),
            ),
        ] {
            assert_eq!(&unfold(type_), unfolded_type);
        }
    }
}
