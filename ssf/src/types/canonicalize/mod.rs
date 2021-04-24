mod record_equality;

use crate::types::*;
use record_equality::equal_records;

pub(crate) fn canonicalize(type_: &Type) -> Type {
    canonicalize_with_records(type_, &[])
}

fn canonicalize_with_records(type_: &Type, records: &[&Record]) -> Type {
    let canonicalize = |type_| canonicalize_with_records(type_, records);

    match type_ {
        Type::Function(function) => Function::new(
            canonicalize(function.argument()),
            canonicalize(function.result()),
        )
        .into(),
        Type::Record(record) => {
            for (index, parent_type) in records.iter().enumerate() {
                if equal_records(record, parent_type, records) {
                    return Type::Index(index);
                }
            }

            let records = vec![record]
                .into_iter()
                .chain(records.iter().copied())
                .collect::<Vec<_>>();

            Record::new(
                record
                    .elements()
                    .iter()
                    .map(|element| canonicalize_with_records(element, &records))
                    .collect(),
                record.is_boxed(),
            )
            .into()
        }
        _ => type_.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn canonicalize_() {
        for (type_, canonical_type) in &[
            (Primitive::Float64.into(), Primitive::Float64.into()),
            (
                Function::new(Primitive::Float64, Primitive::Float64).into(),
                Function::new(Primitive::Float64, Primitive::Float64).into(),
            ),
            (
                Algebraic::new(vec![Constructor::boxed(vec![Primitive::Float64.into()])]).into(),
                Algebraic::new(vec![Constructor::boxed(vec![Primitive::Float64.into()])]).into(),
            ),
            (
                Algebraic::new(vec![Constructor::boxed(vec![Algebraic::new(vec![
                    Constructor::boxed(vec![Type::Index(0)]),
                ])
                .into()])])
                .into(),
                Algebraic::new(vec![Constructor::boxed(vec![Type::Index(0)])]).into(),
            ),
            (
                Algebraic::new(vec![Constructor::boxed(vec![Algebraic::new(vec![
                    Constructor::boxed(vec![Type::Index(1)]),
                ])
                .into()])])
                .into(),
                Algebraic::new(vec![Constructor::boxed(vec![Type::Index(0)])]).into(),
            ),
            (
                Algebraic::new(vec![Constructor::boxed(vec![Function::new(
                    Primitive::Float64,
                    Algebraic::new(vec![Constructor::boxed(vec![Function::new(
                        Primitive::Float64,
                        Type::Index(0),
                    )
                    .into()])]),
                )
                .into()])])
                .into(),
                Algebraic::new(vec![Constructor::boxed(vec![Function::new(
                    Primitive::Float64,
                    Type::Index(0),
                )
                .into()])])
                .into(),
            ),
            (
                Algebraic::new(vec![Constructor::boxed(vec![Function::new(
                    Primitive::Float64,
                    Algebraic::new(vec![Constructor::boxed(vec![Function::new(
                        Primitive::Float64,
                        Type::Index(1),
                    )
                    .into()])]),
                )
                .into()])])
                .into(),
                Algebraic::new(vec![Constructor::boxed(vec![Function::new(
                    Primitive::Float64,
                    Type::Index(0),
                )
                .into()])])
                .into(),
            ),
            (
                Algebraic::new(vec![Constructor::boxed(vec![Algebraic::new(vec![
                    Constructor::unboxed(vec![Algebraic::new(vec![Constructor::boxed(vec![
                        Type::Index(2),
                    ])])
                    .into()]),
                ])
                .into()])])
                .into(),
                Algebraic::new(vec![Constructor::boxed(vec![Algebraic::new(vec![
                    Constructor::unboxed(vec![Algebraic::new(vec![Constructor::boxed(vec![
                        Type::Index(2),
                    ])])
                    .into()]),
                ])
                .into()])])
                .into(),
            ),
            (
                Algebraic::new(vec![Constructor::boxed(vec![Algebraic::new(vec![
                    Constructor::unboxed(vec![Algebraic::new(vec![Constructor::boxed(vec![
                        Algebraic::new(vec![Constructor::boxed(vec![Algebraic::new(vec![
                            Constructor::unboxed(vec![Algebraic::new(vec![Constructor::boxed(
                                vec![Type::Index(2)],
                            )])
                            .into()]),
                        ])
                        .into()])])
                        .into(),
                    ])])
                    .into()]),
                ])
                .into()])])
                .into(),
                Algebraic::new(vec![Constructor::boxed(vec![Algebraic::new(vec![
                    Constructor::unboxed(vec![Algebraic::new(vec![Constructor::boxed(vec![
                        Type::Index(2),
                    ])])
                    .into()]),
                ])
                .into()])])
                .into(),
            ),
            (
                Algebraic::new(vec![Constructor::boxed(vec![Algebraic::new(vec![
                    Constructor::boxed(vec![Algebraic::new(vec![
                        Constructor::boxed(vec![Algebraic::new(vec![Constructor::boxed(vec![
                            Algebraic::new(vec![Constructor::boxed(vec![Algebraic::new(vec![
                                Constructor::boxed(vec![Type::Index(2)]),
                                Constructor::boxed(vec![Type::Index(2)]),
                            ])
                            .into()])])
                            .into(),
                        ])])
                        .into()]),
                        Constructor::boxed(vec![Type::Index(2)]),
                    ])
                    .into()]),
                ])
                .into()])])
                .into(),
                Algebraic::new(vec![Constructor::boxed(vec![Algebraic::new(vec![
                    Constructor::boxed(vec![Algebraic::new(vec![
                        Constructor::boxed(vec![Type::Index(2)]),
                        Constructor::boxed(vec![Type::Index(2)]),
                    ])
                    .into()]),
                ])
                .into()])])
                .into(),
            ),
            (
                Algebraic::new(vec![Constructor::boxed(vec![Algebraic::new(vec![
                    Constructor::boxed(vec![Algebraic::new(vec![
                        Constructor::boxed(vec![Algebraic::new(vec![Constructor::boxed(vec![
                            Type::Index(2),
                        ])])
                        .into()]),
                        Constructor::boxed(vec![Type::Index(2)]),
                    ])
                    .into()]),
                ])
                .into()])])
                .into(),
                Algebraic::new(vec![Constructor::boxed(vec![Algebraic::new(vec![
                    Constructor::boxed(vec![Algebraic::new(vec![
                        Constructor::boxed(vec![Type::Index(2)]),
                        Constructor::boxed(vec![Type::Index(2)]),
                    ])
                    .into()]),
                ])
                .into()])])
                .into(),
            ),
            (
                Algebraic::new(vec![Constructor::boxed(vec![Algebraic::new(vec![
                    Constructor::unboxed(vec![]),
                    Constructor::unboxed(vec![Algebraic::new(vec![Constructor::boxed(vec![
                        Algebraic::new(vec![Constructor::boxed(vec![Algebraic::new(vec![
                            Constructor::unboxed(vec![]),
                            Constructor::unboxed(vec![Algebraic::new(vec![Constructor::boxed(
                                vec![Type::Index(2)],
                            )])
                            .into()]),
                        ])
                        .into()])])
                        .into(),
                    ])])
                    .into()]),
                ])
                .into()])])
                .into(),
                Algebraic::new(vec![Constructor::boxed(vec![Algebraic::new(vec![
                    Constructor::unboxed(vec![]),
                    Constructor::unboxed(vec![Algebraic::new(vec![Constructor::boxed(vec![
                        Type::Index(2),
                    ])])
                    .into()]),
                ])
                .into()])])
                .into(),
            ),
        ] {
            assert_eq!(&canonicalize(type_), canonical_type);
        }
    }
}
