use std::collections::HashMap;

/// Replace all occurrences of `${key}` with `value` in `src`
pub fn expand_var(src: &str, key: &str, value: &str) -> String {
    let from = format!("${{{}}}", key);
    src.replace(&from, value)
}

/// Return a copy of `src` with all ${var} expanded
pub fn expand_vars(src: &str, vars: &HashMap<String, String>) -> String {
    let mut dst = src.to_string();
    for (name, value) in vars.iter() {
        dst = expand_var(&dst, name, value);
    }
    dst
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _test_expand_var(src: &str, key: &str, value: &str, expected: &str) {
        let result = expand_var(&src, &key, &value);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_expand_var() {
        _test_expand_var("foo${exe}", "exe", ".exe", "foo.exe");
        _test_expand_var("${dir}/bar", "dir", "some/where", "some/where/bar");
    }

    #[test]
    fn test_expand_vars() {
        let map = HashMap::from([
            ("exe".into(), ".exe".into()),
            ("dir".into(), "some/where".into()),
        ]);

        let result = expand_vars("${dir}/foo${exe}", &map);
        assert_eq!(result, "some/where/foo.exe");
    }
}
