use crate::types::*;

pub(crate) fn unfold(record: &Record) -> Record {
    Record::new(
        record
            .elements()
            .iter()
            .map(|type_| canonicalize(&unfold_with_root(type_, record, 0)))
            .collect(),
        record.is_boxed(),
    )
}

fn unfold_with_root(type_: &Type, root_record: &Record, root_index: usize) -> Type {
    let unfold = |type_| unfold_with_root(type_, root_record, root_index);

    match type_ {
        Type::Function(function) => {
            Function::new(unfold(function.argument()), unfold(function.result())).into()
        }
        Type::Index(index) => {
            if *index == root_index {
                root_record.clone().into()
            } else {
                Type::Index(*index)
            }
        }
        Type::Record(record) => Record::new(
            record
                .elements()
                .iter()
                .map(|type_| unfold_with_root(type_, root_record, root_index + 1))
                .collect(),
            record.is_boxed(),
        )
        .into(),
        Type::Primitive(_) | Type::Variant => type_.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn unfold_() {
        for (type_, root_type) in &[
            (
                Record::new(vec![Type::Index(0)], true),
                Record::new(vec![Record::new(vec![Type::Index(0)], true).into()], true),
            ),
            (
                Record::new(vec![Record::new(vec![Type::Index(1)], true).into()], false),
                Record::new(
                    vec![
                        Record::new(vec![Record::new(vec![Type::Index(1)], false).into()], true)
                            .into(),
                    ],
                    false,
                ),
            ),
            (
                Record::new(
                    vec![
                        Record::new(vec![Record::new(vec![Type::Index(2)], true).into()], false)
                            .into(),
                    ],
                    true,
                ),
                Record::new(
                    vec![Record::new(
                        vec![Record::new(
                            vec![Record::new(vec![Type::Index(2)], true).into()],
                            true,
                        )
                        .into()],
                        false,
                    )
                    .into()],
                    true,
                ),
            ),
        ] {
            assert_eq!(&unfold(type_), root_type);
        }
    }
}
