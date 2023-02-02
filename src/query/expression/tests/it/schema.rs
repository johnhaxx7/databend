// Copyright 2023 Datafuse Labs.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::BTreeMap;

use common_exception::Result;
use common_expression::create_test_complex_schema;
use common_expression::types::NumberDataType;
use common_expression::TableDataType;
use common_expression::TableField;
use common_expression::TableSchema;
use pretty_assertions::assert_eq;

#[test]
fn test_project_schema_from_tuple() -> Result<()> {
    let b1 = TableDataType::Tuple {
        fields_name: vec!["b11".to_string(), "b12".to_string()],
        fields_type: vec![TableDataType::Boolean, TableDataType::String],
    };
    let b = TableDataType::Tuple {
        fields_name: vec!["b1".to_string(), "b2".to_string()],
        fields_type: vec![b1.clone(), TableDataType::Number(NumberDataType::Int64)],
    };
    let fields = vec![
        TableField::new("a", TableDataType::Number(NumberDataType::UInt64)),
        TableField::new("b", b.clone()),
        TableField::new("c", TableDataType::Number(NumberDataType::UInt64)),
    ];
    let mut schema = TableSchema::new(fields);

    // project schema
    {
        let expect_fields = vec![
            TableField::new_from_column_id("a", TableDataType::Number(NumberDataType::UInt64), 0),
            TableField::new_from_column_id("b:b1:b11", TableDataType::Boolean, 1),
            TableField::new_from_column_id("b:b1:b12", TableDataType::String, 2),
            TableField::new_from_column_id("b:b2", TableDataType::Number(NumberDataType::Int64), 3),
            TableField::new_from_column_id("b:b1", b1.clone(), 1),
            TableField::new_from_column_id("b", b.clone(), 1),
        ];

        let mut path_indices = BTreeMap::new();
        path_indices.insert(0, vec![0]); // a
        path_indices.insert(1, vec![1, 0, 0]); // b:b1:b11
        path_indices.insert(2, vec![1, 0, 1]); // b:b1:b12
        path_indices.insert(3, vec![1, 1]); // b:b2
        path_indices.insert(4, vec![1, 0]); // b:b1
        path_indices.insert(5, vec![1]); // b
        let project_schema = schema.inner_project(&path_indices);

        for (i, field) in project_schema.fields().iter().enumerate() {
            assert_eq!(*field, expect_fields[i]);
        }
        assert_eq!(project_schema.next_column_id(), schema.next_column_id());

        // check leaf fields
        {
            let expected_column_id_field = vec![
                (0, "a"),
                (1, "b:b1:b11"),
                (2, "b:b1:b12"),
                (3, "b:b2"),
                (1, "b11"),
                (2, "b12"),
                (1, "b11"),
                (2, "b12"),
                (3, "b2"),
            ];
            let (column_ids, leaf_fields) = project_schema.leaf_fields();
            for (i, (column_id, leaf_field)) in column_ids.iter().zip(leaf_fields).enumerate() {
                assert_eq!(expected_column_id_field[i].0, *column_id);
                assert_eq!(expected_column_id_field[i].1, leaf_field.name());
            }
        }
    };

    // drop column
    {
        schema.drop_column("b")?;
        let mut path_indices = BTreeMap::new();
        path_indices.insert(0, vec![0]);
        path_indices.insert(1, vec![1]);
        let project_schema = schema.inner_project(&path_indices);

        let expect_fields = vec![
            TableField::new_from_column_id("a", TableDataType::Number(NumberDataType::UInt64), 0),
            TableField::new_from_column_id("c", TableDataType::Number(NumberDataType::UInt64), 4),
        ];

        for (i, field) in project_schema.fields().iter().enumerate() {
            assert_eq!(*field, expect_fields[i]);
        }
        assert_eq!(project_schema.next_column_id(), schema.next_column_id());
    }

    // add column
    {
        schema.add_columns(&[TableField::new("b", b.clone())])?;

        let mut path_indices = BTreeMap::new();
        path_indices.insert(0, vec![0]);
        path_indices.insert(1, vec![1]);
        path_indices.insert(2, vec![2, 0, 0]);
        path_indices.insert(3, vec![2, 0, 1]);
        path_indices.insert(4, vec![2, 1]);
        path_indices.insert(5, vec![2, 0]);
        path_indices.insert(6, vec![2]);

        let expect_fields = vec![
            TableField::new_from_column_id("a", TableDataType::Number(NumberDataType::UInt64), 0),
            TableField::new_from_column_id("c", TableDataType::Number(NumberDataType::UInt64), 4),
            TableField::new_from_column_id("b:b1:b11", TableDataType::Boolean, 5),
            TableField::new_from_column_id("b:b1:b12", TableDataType::String, 6),
            TableField::new_from_column_id("b:b2", TableDataType::Number(NumberDataType::Int64), 7),
            TableField::new_from_column_id("b:b1", b1, 5),
            TableField::new_from_column_id("b", b, 5),
        ];
        let project_schema = schema.inner_project(&path_indices);

        for (i, field) in project_schema.fields().iter().enumerate() {
            assert_eq!(*field, expect_fields[i]);
        }
        assert_eq!(project_schema.next_column_id(), schema.next_column_id());
    }

    Ok(())
}

#[test]
fn test_schema_from_simple_type() -> Result<()> {
    let field1 = TableField::new("a", TableDataType::Number(NumberDataType::UInt64));
    let field2 = TableField::new("b", TableDataType::Number(NumberDataType::UInt64));
    let field3 = TableField::new(
        "c",
        TableDataType::Nullable(Box::new(TableDataType::Number(NumberDataType::UInt64))),
    );

    let schema = TableSchema::new(vec![field1, field2, field3]);
    assert_eq!(schema.to_column_ids(), vec![0, 1, 2]);
    assert_eq!(schema.to_leaf_column_ids(), vec![0, 1, 2]);
    assert_eq!(schema.next_column_id(), 3);

    let (leaf_column_ids, leaf_fields) = schema.leaf_fields();
    assert_eq!(leaf_column_ids, schema.to_leaf_column_ids());
    let leaf_field_names = vec!["a", "b", "c"];
    for (i, field) in leaf_fields.iter().enumerate() {
        assert_eq!(field.name(), leaf_field_names[i])
    }

    Ok(())
}

#[test]
fn test_schema_from_struct() -> Result<()> {
    let schema = create_test_complex_schema();
    let flat_column_ids = schema.to_leaf_column_ids();

    let (leaf_column_ids, leaf_fields) = schema.leaf_fields();
    assert_eq!(leaf_column_ids, schema.to_leaf_column_ids());
    let expected_fields = vec![
        ("u64", TableDataType::Number(NumberDataType::UInt64)),
        ("0", TableDataType::Number(NumberDataType::UInt64)),
        ("1", TableDataType::Number(NumberDataType::UInt64)),
        ("1:0", TableDataType::Number(NumberDataType::UInt64)),
        ("0", TableDataType::Number(NumberDataType::UInt64)),
        ("1", TableDataType::Number(NumberDataType::UInt64)),
        (
            "nullarray",
            TableDataType::Nullable(Box::new(TableDataType::Array(Box::new(
                TableDataType::Number(NumberDataType::UInt64),
            )))),
        ),
        (
            "maparray",
            TableDataType::Map(Box::new(TableDataType::Array(Box::new(
                TableDataType::Number(NumberDataType::UInt64),
            )))),
        ),
        (
            "nullu64",
            TableDataType::Nullable(Box::new(TableDataType::Number(NumberDataType::UInt64))),
        ),
        ("u64array:0", TableDataType::Number(NumberDataType::UInt64)),
        ("a", TableDataType::Number(NumberDataType::Int32)),
        ("b", TableDataType::Number(NumberDataType::Int32)),
    ];
    for (i, field) in leaf_fields.iter().enumerate() {
        let expected_field = &expected_fields[i];
        assert_eq!(field.name(), expected_field.0);
        assert_eq!(field.data_type().to_owned(), expected_field.1);
    }

    let expeted_column_ids = vec![
        ("u64", vec![0]),
        ("tuplearray", vec![1, 1, 1, 2, 3, 3]),
        ("arraytuple", vec![4, 4, 4, 5]),
        ("nullarray", vec![6]),
        ("maparray", vec![7]),
        ("nullu64", vec![8]),
        ("u64array", vec![9, 9]),
        ("tuplesimple", vec![10, 10, 11]),
    ];

    for (i, column_id) in schema.field_column_ids().iter().enumerate() {
        let expeted_column_id = &expeted_column_ids[i];
        assert_eq!(
            expeted_column_id.0.to_string(),
            schema.fields()[i].name().to_string()
        );
        assert_eq!(expeted_column_id.1, *column_id);
    }

    let expeted_flat_column_ids = vec![
        ("u64", vec![0]),
        ("tuplearray", vec![1, 2, 3]),
        ("arraytuple", vec![4, 5]),
        ("nullarray", vec![6]),
        ("maparray", vec![7]),
        ("nullu64", vec![8]),
        ("u64array", vec![9]),
        ("tuplesimple", vec![10, 11]),
    ];

    for (i, field) in schema.fields().iter().enumerate() {
        let expeted_column_id = &expeted_flat_column_ids[i];
        assert_eq!(expeted_column_id.0.to_string(), field.name().to_string());
        assert_eq!(expeted_column_id.1, field.leaf_column_ids());
    }

    assert_eq!(schema.next_column_id(), 12);

    // make sure column ids is adjacent integers(in case there is no add or drop column operations)
    assert_eq!(flat_column_ids.len(), schema.next_column_id() as usize);
    for i in 1..flat_column_ids.len() {
        assert_eq!(flat_column_ids[i], flat_column_ids[i - 1] + 1);
    }

    // check leaf fields
    {
        let expected_column_id_field = vec![
            (0, "u64"),
            (1, "0"),
            (2, "1"),
            (3, "1:0"),
            (4, "0"),
            (5, "1"),
            (6, "nullarray"),
            (7, "maparray"),
            (8, "nullu64"),
            (9, "u64array:0"),
            (10, "a"),
            (11, "b"),
        ];
        let (column_ids, leaf_fields) = schema.leaf_fields();
        for (i, (column_id, leaf_field)) in column_ids.iter().zip(leaf_fields).enumerate() {
            assert_eq!(expected_column_id_field[i].0, *column_id);
            assert_eq!(expected_column_id_field[i].1, leaf_field.name());
        }
    }

    Ok(())
}

#[test]
fn test_schema_modify_field() -> Result<()> {
    let field1 = TableField::new("a", TableDataType::Number(NumberDataType::UInt64));
    let field2 = TableField::new("b", TableDataType::Number(NumberDataType::UInt64));
    let field3 = TableField::new("c", TableDataType::Number(NumberDataType::UInt64));

    let mut schema = TableSchema::new(vec![field1.clone()]);

    let expected_field1 =
        TableField::new_from_column_id("a", TableDataType::Number(NumberDataType::UInt64), 0);
    let expected_field2 =
        TableField::new_from_column_id("b", TableDataType::Number(NumberDataType::UInt64), 1);
    let expected_field3 =
        TableField::new_from_column_id("c", TableDataType::Number(NumberDataType::UInt64), 2);

    assert_eq!(schema.fields().to_owned(), vec![expected_field1.clone()]);
    assert_eq!(schema.column_id_of("a").unwrap(), 0);
    assert_eq!(schema.is_column_deleted(0), false);
    assert_eq!(schema.to_column_ids(), vec![0]);
    assert_eq!(schema.to_leaf_column_ids(), vec![0]);
    assert_eq!(schema.next_column_id(), 1);

    // add column b
    schema.add_columns(&[field2])?;
    assert_eq!(schema.fields().to_owned(), vec![
        expected_field1.clone(),
        expected_field2,
    ]);
    assert_eq!(schema.column_id_of("a").unwrap(), 0);
    assert_eq!(schema.column_id_of("b").unwrap(), 1);
    assert_eq!(schema.is_column_deleted(0), false);
    assert_eq!(schema.is_column_deleted(1), false);
    assert_eq!(schema.to_column_ids(), vec![0, 1]);
    assert_eq!(schema.to_leaf_column_ids(), vec![0, 1]);
    assert_eq!(schema.next_column_id(), 2);

    // drop column b
    schema.drop_column("b")?;
    assert_eq!(schema.fields().to_owned(), vec![expected_field1.clone(),]);
    assert_eq!(schema.column_id_of("a").unwrap(), 0);
    assert_eq!(schema.is_column_deleted(0), false);
    assert_eq!(schema.is_column_deleted(1), true);
    assert_eq!(schema.to_column_ids(), vec![0]);
    assert_eq!(schema.to_leaf_column_ids(), vec![0]);
    assert_eq!(schema.next_column_id(), 2);

    // add column c
    schema.add_columns(&[field3])?;
    assert_eq!(schema.fields().to_owned(), vec![
        expected_field1,
        expected_field3
    ]);
    assert_eq!(schema.column_id_of("a").unwrap(), 0);
    assert_eq!(schema.column_id_of("c").unwrap(), 2);
    assert_eq!(schema.is_column_deleted(0), false);
    assert_eq!(schema.is_column_deleted(1), true);
    assert_eq!(schema.is_column_deleted(2), false);
    assert_eq!(schema.to_column_ids(), vec![0, 2]);
    assert_eq!(schema.to_leaf_column_ids(), vec![0, 2]);
    assert_eq!(schema.next_column_id(), 3);

    // add struct column
    let child_field11 = TableDataType::Number(NumberDataType::UInt64);
    let child_field12 = TableDataType::Number(NumberDataType::UInt64);
    let child_field22 = TableDataType::Number(NumberDataType::UInt64);
    let s = TableDataType::Tuple {
        fields_name: vec!["0".to_string(), "1".to_string()],
        fields_type: vec![child_field11.clone(), child_field12.clone()],
    };
    let s2 = TableDataType::Tuple {
        fields_name: vec!["0".to_string(), "1".to_string()],
        fields_type: vec![s.clone(), child_field22.clone()],
    };
    schema.add_columns(&[TableField::new("s", s2.clone())])?;
    assert_eq!(schema.column_id_of("s").unwrap(), 3);
    assert_eq!(schema.is_column_deleted(0), false);
    assert_eq!(schema.is_column_deleted(1), true);
    assert_eq!(schema.is_column_deleted(2), false);
    assert_eq!(schema.is_column_deleted(3), false);
    assert_eq!(schema.to_column_ids(), vec![0, 2, 3, 3, 3, 4, 5]);
    assert_eq!(schema.to_leaf_column_ids(), vec![0, 2, 3, 4, 5]);
    assert_eq!(schema.next_column_id(), 6);

    // add array column
    let ary = TableDataType::Array(Box::new(TableDataType::Array(Box::new(
        TableDataType::Number(NumberDataType::UInt64),
    ))));
    schema.add_columns(&[TableField::new("ary", ary.clone())])?;
    assert_eq!(schema.column_id_of("ary").unwrap(), 6);
    assert_eq!(schema.is_column_deleted(0), false);
    assert_eq!(schema.is_column_deleted(1), true);
    assert_eq!(schema.is_column_deleted(2), false);
    assert_eq!(schema.is_column_deleted(3), false);
    assert_eq!(schema.is_column_deleted(6), false);
    assert_eq!(schema.to_column_ids(), vec![0, 2, 3, 3, 3, 4, 5, 6, 6, 6]);
    assert_eq!(schema.to_leaf_column_ids(), vec![0, 2, 3, 4, 5, 6]);
    assert_eq!(schema.next_column_id(), 7);

    // check leaf fields
    {
        let expected_column_id_field = vec![
            (0, "a"),
            (2, "c"),
            (3, "0"),
            (4, "1"),
            (5, "1"),
            (6, "ary:0:0"),
        ];
        let (column_ids, leaf_fields) = schema.leaf_fields();
        for (i, (column_id, leaf_field)) in column_ids.iter().zip(leaf_fields).enumerate() {
            assert_eq!(expected_column_id_field[i].0, *column_id);
            assert_eq!(expected_column_id_field[i].1, leaf_field.name());
        }
    }

    // check project fields
    {
        let mut project_fields = BTreeMap::new();
        project_fields.insert(0, field1);
        project_fields.insert(2, TableField::new("s", s2));
        project_fields.insert(3, TableField::new("0", s));
        project_fields.insert(4, TableField::new("0", child_field11));
        project_fields.insert(5, TableField::new("1", child_field12));
        project_fields.insert(6, TableField::new("1", child_field22));
        project_fields.insert(7, TableField::new("ary", ary));
        project_fields.insert(
            8,
            TableField::new(
                "ary:0",
                TableDataType::Array(Box::new(TableDataType::Number(NumberDataType::UInt64))),
            ),
        );
        project_fields.insert(
            9,
            TableField::new("0", TableDataType::Number(NumberDataType::UInt64)),
        );
        let project_schema = schema.project_by_fields(&project_fields);
        let expected_column_ids = vec![
            (0, vec![0]),
            (2, vec![3, 3, 3, 4, 5]),
            (3, vec![3, 3, 4]),
            (4, vec![3]),
            (5, vec![4]),
            (6, vec![5]),
            (7, vec![6, 6, 6]),
            (8, vec![6, 6]),
            (9, vec![6]),
        ];
        for (project_schema_index, (_i, column_ids)) in expected_column_ids.into_iter().enumerate()
        {
            let field = &project_schema.fields()[project_schema_index];
            assert_eq!(field.column_ids(), column_ids);
        }
    }

    // drop tuple column
    schema.drop_column("s")?;
    assert_eq!(schema.is_column_deleted(0), false);
    assert_eq!(schema.is_column_deleted(1), true);
    assert_eq!(schema.is_column_deleted(2), false);
    assert_eq!(schema.is_column_deleted(3), true);
    assert_eq!(schema.is_column_deleted(6), false);
    assert_eq!(schema.to_column_ids(), vec![0, 2, 6, 6, 6]);
    assert_eq!(schema.to_leaf_column_ids(), vec![0, 2, 6]);
    assert!(schema.column_id_of("s").is_err());

    Ok(())
}