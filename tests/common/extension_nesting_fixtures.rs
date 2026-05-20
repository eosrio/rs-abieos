#[derive(Clone, Copy)]
pub struct ExtensionNestingCase {
    pub label: &'static str,
    pub ty: &'static str,
    pub json: &'static str,
    pub expected_json: &'static str,
}

impl ExtensionNestingCase {
    const fn new(label: &'static str, ty: &'static str, json: &'static str) -> Self {
        Self {
            label,
            ty,
            json,
            expected_json: json,
        }
    }

    const fn with_expected(
        label: &'static str,
        ty: &'static str,
        json: &'static str,
        expected_json: &'static str,
    ) -> Self {
        Self {
            label,
            ty,
            json,
            expected_json,
        }
    }
}

pub const EXTENSION_NESTING_ABI: &str = r#"{
    "version": "eosio::abi/1.1",
    "structs": [
        {
            "name": "s1",
            "fields": [
                { "name": "x1", "type": "int8" }
            ]
        },
        {
            "name": "s2",
            "fields": [
                { "name": "y1", "type": "int8$" },
                { "name": "y2", "type": "int8$" }
            ]
        },
        {
            "name": "s3",
            "fields": [
                { "name": "z1", "type": "int8$" },
                { "name": "z2", "type": "v1$" },
                { "name": "z3", "type": "s2$" }
            ]
        },
        {
            "name": "s4",
            "fields": [
                { "name": "a1", "type": "int8?$" },
                { "name": "b1", "type": "int8[]$" }
            ]
        },
        {
            "name": "ext_struct",
            "fields": [
                {"name": "f1", "type": "int32"},
                {"name": "f2", "type": "int32$"},
                {"name": "f3", "type": "int32$"}
            ]
        },
        {
            "name": "nested_ext_struct",
            "fields": [
                {"name": "n1", "type": "int32"},
                {"name": "n2", "type": "ext_struct$"}
            ]
        },
        {
            "name": "array_of_ext_struct",
            "fields": [
                {"name": "a1", "type": "ext_struct[]"}
            ]
        },
        {
            "name": "ext_array_of_ext_struct",
            "fields": [
                {"name": "a1", "type": "ext_struct[]$"}
            ]
        },
        {
            "name": "variant_of_ext_struct",
            "fields": [
                {"name": "v1", "type": "v_ext"}
            ]
        },
        {
            "name": "complex_nesting",
            "fields": [
                {"name": "c1", "type": "int32"},
                {"name": "c2", "type": "nested_ext_struct$"},
                {"name": "c3", "type": "array_of_ext_struct$"},
                {"name": "c4", "type": "variant_of_ext_struct$"}
            ]
        },
        {
            "name": "s7",
            "fields": [
                { "name": "bs", "type": "bitset" }
            ]
        },
        {
            "name": "s8",
            "fields": [
                { "name": "a1", "type": "int8[2]" }
            ]
        },
        {
            "name": "s9",
            "fields": [
                { "name": "a1", "type": "s1[2]" }
            ]
        }
    ],
    "variants": [
        {
            "name": "v1",
            "types": ["int8","s1","s2","s7"]
        },
        {
            "name": "v_ext",
            "types": ["int32", "ext_struct"]
        }
    ]
}"#;

pub const EXTENSION_NESTING_CASES: &[ExtensionNestingCase] = &[
    // Ported from test.cpp
    ExtensionNestingCase::new("variant int8", "v1", r#"["int8",7]"#),
    ExtensionNestingCase::new("variant struct", "v1", r#"["s1",{"x1":6}]"#),
    ExtensionNestingCase::new(
        "variant extension struct",
        "v1",
        r#"["s2",{"y1":5,"y2":4}]"#,
    ),
    ExtensionNestingCase::new("extension empty", "s3", r#"{}"#),
    ExtensionNestingCase::new("extension first field", "s3", r#"{"z1":7}"#),
    ExtensionNestingCase::new("extension variant", "s3", r#"{"z1":7,"z2":["int8",6]}"#),
    ExtensionNestingCase::with_expected(
        "extension trailing empty skipped",
        "s3",
        r#"{"z1":7,"z2":["int8",6],"z3":{}}"#,
        r#"{"z1":7,"z2":["int8",6]}"#,
    ),
    ExtensionNestingCase::new(
        "extension nested one field",
        "s3",
        r#"{"z1":7,"z2":["int8",6],"z3":{"y1":9}}"#,
    ),
    ExtensionNestingCase::new(
        "extension nested two fields",
        "s3",
        r#"{"z1":7,"z2":["int8",6],"z3":{"y1":9,"y2":10}}"#,
    ),
    ExtensionNestingCase::new("optional extension empty", "s4", r#"{}"#),
    ExtensionNestingCase::new("optional extension null", "s4", r#"{"a1":null}"#),
    ExtensionNestingCase::new("optional extension value", "s4", r#"{"a1":7}"#),
    ExtensionNestingCase::new(
        "optional extension array empty",
        "s4",
        r#"{"a1":null,"b1":[]}"#,
    ),
    ExtensionNestingCase::new(
        "optional extension array values",
        "s4",
        r#"{"a1":null,"b1":[5,6,7]}"#,
    ),
    // Additional nesting shapes
    ExtensionNestingCase::new(
        "extension struct required only",
        "ext_struct",
        r#"{"f1":1}"#,
    ),
    ExtensionNestingCase::new(
        "extension struct one extension",
        "ext_struct",
        r#"{"f1":1,"f2":2}"#,
    ),
    ExtensionNestingCase::new(
        "extension struct all extensions",
        "ext_struct",
        r#"{"f1":1,"f2":2,"f3":3}"#,
    ),
    ExtensionNestingCase::new(
        "nested extension struct empty",
        "nested_ext_struct",
        r#"{"n1":10}"#,
    ),
    ExtensionNestingCase::new(
        "nested extension struct required nested",
        "nested_ext_struct",
        r#"{"n1":10,"n2":{"f1":1}}"#,
    ),
    ExtensionNestingCase::new(
        "nested extension struct one nested extension",
        "nested_ext_struct",
        r#"{"n1":10,"n2":{"f1":1,"f2":2}}"#,
    ),
    ExtensionNestingCase::new(
        "nested extension struct all nested extensions",
        "nested_ext_struct",
        r#"{"n1":10,"n2":{"f1":1,"f2":2,"f3":3}}"#,
    ),
    ExtensionNestingCase::new(
        "array of extension structs empty",
        "array_of_ext_struct",
        r#"{"a1":[]}"#,
    ),
    ExtensionNestingCase::new(
        "array of extension structs one",
        "array_of_ext_struct",
        r#"{"a1":[{"f1":1,"f2":2,"f3":3}]}"#,
    ),
    ExtensionNestingCase::new(
        "array of extension structs two",
        "array_of_ext_struct",
        r#"{"a1":[{"f1":1,"f2":2,"f3":3},{"f1":4,"f2":5,"f3":6}]}"#,
    ),
    ExtensionNestingCase::new("extension array absent", "ext_array_of_ext_struct", r#"{}"#),
    ExtensionNestingCase::new(
        "extension array empty",
        "ext_array_of_ext_struct",
        r#"{"a1":[]}"#,
    ),
    ExtensionNestingCase::new(
        "extension array one",
        "ext_array_of_ext_struct",
        r#"{"a1":[{"f1":1,"f2":2,"f3":3}]}"#,
    ),
    ExtensionNestingCase::new(
        "variant containing int32",
        "variant_of_ext_struct",
        r#"{"v1":["int32",100]}"#,
    ),
    ExtensionNestingCase::new(
        "variant containing extension struct required only",
        "variant_of_ext_struct",
        r#"{"v1":["ext_struct",{"f1":10}]}"#,
    ),
    ExtensionNestingCase::new(
        "variant containing extension struct one extension",
        "variant_of_ext_struct",
        r#"{"v1":["ext_struct",{"f1":10,"f2":20}]}"#,
    ),
    ExtensionNestingCase::new(
        "variant containing extension struct all extensions",
        "variant_of_ext_struct",
        r#"{"v1":["ext_struct",{"f1":10,"f2":20,"f3":30}]}"#,
    ),
    ExtensionNestingCase::new(
        "complex nesting base only",
        "complex_nesting",
        r#"{"c1":1000}"#,
    ),
    ExtensionNestingCase::new(
        "complex nesting through c2",
        "complex_nesting",
        r#"{"c1":1000,"c2":{"n1":10,"n2":{"f1":1,"f2":2,"f3":3}}}"#,
    ),
    ExtensionNestingCase::new(
        "complex nesting through c3",
        "complex_nesting",
        r#"{"c1":1000,"c2":{"n1":10,"n2":{"f1":1,"f2":2,"f3":3}},"c3":{"a1":[{"f1":1,"f2":2,"f3":3}]}}"#,
    ),
    ExtensionNestingCase::new(
        "complex nesting through c4",
        "complex_nesting",
        r#"{"c1":1000,"c2":{"n1":10,"n2":{"f1":1,"f2":2,"f3":3}},"c3":{"a1":[{"f1":1,"f2":2,"f3":3}]},"c4":{"v1":["ext_struct",{"f1":10}]}}"#,
    ),
    // Additional cases from test.cpp that share the same ABI.
    ExtensionNestingCase::new("fixed int8 array", "s8", r#"{"a1":[1,27]}"#),
    ExtensionNestingCase::new("fixed struct array", "s9", r#"{"a1":[{"x1":6},{"x1":16}]}"#),
    ExtensionNestingCase::new("bitset empty", "s7", r#"{"bs":""}"#),
    ExtensionNestingCase::new("bitset zeros", "s7", r#"{"bs":"00000000"}"#),
    ExtensionNestingCase::new("bitset mixed", "s7", r#"{"bs":"1011001"}"#),
];
