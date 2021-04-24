use crate::types::*;

pub fn equal_records(one: &Record, other: &Record, parents: &[&Record]) -> bool {
    equal_records_with_pairs(
        one,
        other,
        &parents
            .iter()
            .copied()
            .zip(parents.iter().copied())
            .collect::<Vec<_>>(),
    )
}

fn equal_records_with_pairs(one: &Record, other: &Record, pairs: &[(&Record, &Record)]) -> bool {
    pairs.contains(&(one, other)) || {
        let pairs = [(one, other)]
            .iter()
            .chain(pairs)
            .copied()
            .collect::<Vec<_>>();

        one.elements().len() == other.elements().len()
            && one.is_boxed() == other.is_boxed()
            && one
                .elements()
                .iter()
                .zip(other.elements())
                .all(|(one, other)| equal_with_pairs(one, other, &pairs))
    }
}

fn equal_with_pairs(one: &Type, other: &Type, pairs: &[(&Record, &Record)]) -> bool {
    let equal = |one, other| equal_with_pairs(one, other, pairs);
    let equal_records = |one, other| equal_records_with_pairs(one, other, pairs);

    match (one, other) {
        (Type::Function(one), Type::Function(other)) => {
            equal(one.argument(), other.argument()) && equal(one.result(), other.result())
        }
        (Type::Primitive(one), Type::Primitive(other)) => one == other,
        (Type::Record(one), Type::Record(other)) => equal_records_with_pairs(one, other, pairs),
        (Type::Index(index), Type::Record(record)) => equal_records(pairs[*index].0, record),
        (Type::Record(record), Type::Index(index)) => equal_records(record, pairs[*index].1),
        (Type::Index(one), Type::Index(other)) => equal_records(pairs[*one].0, pairs[*other].1),
        (Type::Variant, Type::Variant) => true,
        (_, _) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal() {
        for (one, other) in &[
            (Primitive::Float64.into(), Primitive::Float64.into()),
            (
                Function::new(Primitive::Float64, Primitive::Float64).into(),
                Function::new(Primitive::Float64, Primitive::Float64).into(),
            ),
            (
                Record::new(vec![Primitive::Float64.into()], true).into(),
                Record::new(vec![Primitive::Float64.into()], true).into(),
            ),
            (
                Record::new(vec![Type::Index(0)], true).into(),
                Record::new(vec![Record::new(vec![Type::Index(0)], true).into()], true).into(),
            ),
            (
                Record::new(vec![Type::Index(0)], true).into(),
                Record::new(vec![Record::new(vec![Type::Index(1)], true).into()], true).into(),
            ),
            (
                Record::new(vec![Record::new(vec![Type::Index(0)], true).into()], true).into(),
                Record::new(vec![Record::new(vec![Type::Index(1)], true).into()], true).into(),
            ),
            (
                Record::new(
                    vec![Function::new(Primitive::Float64, Type::Index(0)).into()],
                    true,
                )
                .into(),
                Record::new(
                    vec![Function::new(
                        Primitive::Float64,
                        Record::new(
                            vec![Function::new(Primitive::Float64, Type::Index(0)).into()],
                            true,
                        ),
                    )
                    .into()],
                    true,
                )
                .into(),
            ),
            (
                Record::new(
                    vec![Function::new(Primitive::Float64, Type::Index(0)).into()],
                    true,
                )
                .into(),
                Record::new(
                    vec![Function::new(
                        Primitive::Float64,
                        Record::new(
                            vec![Function::new(Primitive::Float64, Type::Index(1)).into()],
                            true,
                        ),
                    )
                    .into()],
                    true,
                )
                .into(),
            ),
        ] {
            assert!(equal_with_pairs(one, other, &[]));
        }
    }

    #[test]
    fn not_equal() {
        for (one, other) in &[
            (
                Primitive::Float64.into(),
                Function::new(Primitive::Float64, Primitive::Float64).into(),
            ),
            (
                Function::new(
                    Primitive::Float64,
                    Function::new(Primitive::Float64, Primitive::Float64),
                )
                .into(),
                Function::new(Primitive::Float64, Primitive::Float64).into(),
            ),
            (
                Record::new(vec![Primitive::Float64.into()], true).into(),
                Record::new(
                    vec![Primitive::Float64.into(), Primitive::Float64.into()],
                    true,
                )
                .into(),
            ),
            (
                Record::new(vec![Primitive::Float64.into()], true).into(),
                Record::new(vec![Primitive::Float64.into()], false).into(),
            ),
        ] {
            assert!(!equal_with_pairs(one, other, &[]));
        }
    }
}
