use trigger_policies::{init_policy_cache, find_match};
use std::collections::HashMap;
use tempfile::NamedTempFile;
use std::io::Write;

#[test]
fn test_policy_load_and_match() {
    let mut tmp = NamedTempFile::new().unwrap();
    write!(
        tmp,
        "policies:
        - id: test1
          trigger_name: high_cpu
          scope: process
          conditions: {{ cpu: \">80\" }}
          actions: [\"throttle\"]
          priority: 10
          cooldown: 5
        "
    )
    .unwrap();

    init_policy_cache(tmp.path().to_str().unwrap()).unwrap();
    let mut ctx = HashMap::new();
    ctx.insert("cpu".to_string(), "90".to_string());
    let match_policy = find_match(&ctx).unwrap().unwrap();
    assert_eq!(match_policy.trigger_name, "high_cpu");
}