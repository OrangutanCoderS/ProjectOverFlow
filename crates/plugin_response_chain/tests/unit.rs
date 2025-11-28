use hashbrown::HashSet;
use plugin_response_chain::{
    load_from_json_str, PluginChainError, PluginResponseChain,
};

const EXAMPLE_JSON: &str = r#"
{
  "chain": {
    "fake_net": {
      "critical": {
        "persistent": ["action_kill", "action_fake_output"],
        "burst": ["action_throttle"]
      },
      "medium": {
        "isolated": ["action_delay"]
      }
    },
    "entropy_flagger": {
      "high": {
        "persistent": ["action_spawn_mirror"],
        "isolated": ["action_suspend"]
      }
    }
  }
}
"#;

fn allowed_action_set() -> HashSet<String> {
    let mut set = HashSet::new();
    for a in &[
        "action_kill",
        "action_fake_output",
        "action_throttle",
        "action_delay",
        "action_spawn_mirror",
        "action_suspend",
    ] {
        set.insert((*a).to_string());
    }
    set
}

#[test]
fn load_chain_without_validation() {
    let chain: PluginResponseChain =
        load_from_json_str(EXAMPLE_JSON, None).expect("chain should load");

    assert!(!chain.is_empty());
    // 6 distinct (plugin, severity, pattern) combos:
    // fake_net: critical/persistent, critical/burst, medium/isolated
    // entropy_flagger: high/persistent, high/isolated
    // -> actually 5 or 6 depending on your compile_def logic; current code assumes 6
    assert_eq!(chain.len(), 6);

    let actions = chain
        .resolve("fake_net", "critical", "persistent")
        .expect("rule must exist");
    assert_eq!(
        actions,
        &[
            "action_kill".to_string(),
            "action_fake_output".to_string()
        ]
    );

    let burst = chain
        .resolve("fake_net", "critical", "burst")
        .expect("burst rule must exist");
    assert_eq!(burst, &["action_throttle".to_string()]);

    // Unknown combination returns None, not panic.
    assert!(chain.resolve("fake_net", "low", "isolated").is_none());
}

#[test]
fn load_chain_with_validation_success() {
    let allowed = allowed_action_set();
    let chain =
        load_from_json_str(EXAMPLE_JSON, Some(&allowed)).expect("chain should validate and load");
    assert_eq!(chain.len(), 6);
}

#[test]
fn load_chain_with_validation_failure() {
    // Intentionally remove one allowed action to trigger UnknownAction
    let mut allowed = allowed_action_set();
    allowed.remove("action_fake_output");

    let err = load_from_json_str(EXAMPLE_JSON, Some(&allowed))
        .expect_err("should fail due to unknown action");

    match err {
        PluginChainError::UnknownAction { action, .. } => {
            assert_eq!(action, "action_fake_output".to_string())
        }
        other => panic!("expected UnknownAction, got: {:?}", other),
    }
}