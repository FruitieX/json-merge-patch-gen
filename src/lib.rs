/**
 * Generates a JSON Merge Patch (RFC 7386)
 * <https://datatracker.ietf.org/doc/html/rfc7386>
 *
 * Ported from <https://github.com/pierreinglebert/json-merge-patch/blob/master/lib/generate.js>
 */
pub fn generate(
    before: &serde_json::Value,
    after: &serde_json::Value,
) -> Option<serde_json::Value> {
    if before.is_null()
        || after.is_null()
        || (!before.is_object() && !before.is_array())
        || (!after.is_object() && !after.is_array())
        || before.is_array() != after.is_array()
    {
        return Some(after.clone());
    }

    if before.is_array() {
        if before != after {
            return Some(after.clone());
        }
        return None;
    }

    let mut patch = serde_json::json!({});

    // The .unwrap() calls are safe because we previously checked that the keys are objects
    let before = before.as_object().unwrap();
    let after = after.as_object().unwrap();

    // New elements
    for (key, value) in after.iter() {
        if !before.contains_key(key) {
            patch[key] = value.clone();
        }
    }

    // Removed & modified elements
    for (key, before_value) in before.iter() {
        match after.get(key) {
            None => {
                patch[key] = serde_json::Value::Null;
            }
            Some(after_value) => {
                if before_value.is_object() {
                    let sub_patch = generate(before_value, after_value);
                    if let Some(sub_patch) = sub_patch {
                        patch[key] = sub_patch;
                    }
                } else if before_value != after_value {
                    patch[key] = after_value.clone();
                }
            }
        }
    }

    if patch.as_object().unwrap().is_empty() {
        None
    } else {
        Some(patch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_replace_attr() {
        let before = json!({ "a": "b" });
        let after = json!({ "a": "c" });
        let patch = generate(&before, &after).unwrap();
        assert_eq!(patch, json!({ "a": "c" }));
    }

    #[test]
    fn test_add_attr() {
        let before = json!({ "a": "b" });
        let after = json!({ "a": "b", "b": "c" });
        let patch = generate(&before, &after).unwrap();
        assert_eq!(patch, json!({ "b": "c" }));
    }

    #[test]
    fn test_del_attr() {
        let before = json!({ "a": "b" });
        let after = json!({});
        let patch = generate(&before, &after).unwrap();
        assert_eq!(patch, json!({ "a": null }));
    }

    #[test]
    fn test_del_attr_no_affect_others() {
        let before = json!({ "a": "b", "b": "c" });
        let after = json!({ "b": "c" });
        let patch = generate(&before, &after).unwrap();
        assert_eq!(patch, json!({ "a": null }));
    }

    #[test]
    fn test_replace_array_with_attr() {
        let before = json!({ "a": ["b"] });
        let after = json!({ "a": "c" });
        let patch = generate(&before, &after).unwrap();
        assert_eq!(patch, json!({ "a": "c" }));
    }

    #[test]
    fn test_replace_attr_with_array() {
        let before = json!({ "a": "c" });
        let after = json!({ "a": ["b"] });
        let patch = generate(&before, &after).unwrap();
        assert_eq!(patch, json!({ "a": ["b"] }));
    }

    #[test]
    fn test_replace_obj_array_with_num_array() {
        let before = json!({ "a": [{ "b": "c" }] });
        let after = json!({ "a": [1] });
        let patch = generate(&before, &after).unwrap();
        assert_eq!(patch, json!({ "a": [1] }));
    }

    #[test]
    fn test_replace_arr_if_changed() {
        let before = json!(["a", "b"]);
        let after = json!(["c", "d"]);
        let patch = generate(&before, &after).unwrap();
        assert_eq!(patch, json!(["c", "d"]));
    }

    #[test]
    fn test_replace_arr_if_elem_deleted() {
        let before = json!(["a", "b"]);
        let after = json!(["a"]);
        let patch = generate(&before, &after).unwrap();
        assert_eq!(patch, json!(["a"]));
    }

    #[test]
    fn test_replace_obj_with_array() {
        let before = json!({ "a": "b" });
        let after = json!(["c"]);
        let patch = generate(&before, &after).unwrap();
        assert_eq!(patch, json!(["c"]));
    }

    #[test]
    fn test_replace_with_null() {
        let before = json!({ "a": "b" });
        let after = json!(null);
        let patch = generate(&before, &after).unwrap();
        assert_eq!(patch, json!(null));
    }

    #[test]
    fn test_replace_with_string() {
        let before = json!({ "a": "foo" });
        let after = json!("bar");
        let patch = generate(&before, &after).unwrap();
        assert_eq!(patch, json!("bar"));
    }

    #[test]
    fn test_keep_null_attrs() {
        let before = json!({ "e": null });
        let after = json!({ "e": null, "a": 1 });
        let patch = generate(&before, &after).unwrap();
        assert_eq!(patch, json!({ "a": 1 }));
    }

    #[test]
    fn test_recursive() {
        let before = json!({});
        let after = json!({ "a": { "bb": {} } });
        let patch = generate(&before, &after).unwrap();
        assert_eq!(patch, json!({ "a": { "bb": {} } }));
    }

    #[test]
    fn test_recursive_replace() {
        let before = json!({ "a": { "b": "c" } });
        let after = json!({ "a": { "b": "d" } });
        let patch = generate(&before, &after).unwrap();
        assert_eq!(patch, json!({ "a": { "b": "d" } }));
    }

    #[test]
    fn test_recursive_add() {
        let before = json!({ "a": { "b": "c" } });
        let after = json!({ "a": { "b": "c", "d": "e" } });
        let patch = generate(&before, &after).unwrap();
        assert_eq!(patch, json!({ "a": { "d": "e" } }));
    }

    #[test]
    fn test_unchanged() {
        let before = json!({ "a": "a" });
        let after = json!({ "a": "a" });
        let patch = generate(&before, &after);
        assert_eq!(patch, None);
    }

    #[test]
    fn test_recursive_unchanged() {
        let before = json!({ "a": { "b": "c" } });
        let after = json!({ "a": { "b": "c" } });
        let patch = generate(&before, &after);
        assert_eq!(patch, None);
    }

    #[test]
    fn test_unchanged_array() {
        let before = json!([1, 2, 3]);
        let after = json!([1, 2, 3]);
        let patch = generate(&before, &after);
        assert_eq!(patch, None);
    }
}
