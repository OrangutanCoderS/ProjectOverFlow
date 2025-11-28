use criterion::{criterion_group, criterion_main, Criterion};
use hashbrown::HashSet;
use plugin_response_chain::{load_from_json_str, PluginResponseChain};

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

fn bench_load_only(c: &mut Criterion) {
    let allowed = allowed_action_set();

    c.bench_function("plugin_response_chain_load_only", |b| {
        b.iter(|| {
            let chain: PluginResponseChain =
                load_from_json_str(EXAMPLE_JSON, Some(&allowed)).unwrap();
            assert!(!chain.is_empty());
        });
    });
}

fn bench_resolve_hot_path(c: &mut Criterion) {
    let allowed = allowed_action_set();
    let chain: PluginResponseChain =
        load_from_json_str(EXAMPLE_JSON, Some(&allowed)).expect("valid chain");

    c.bench_function("plugin_response_chain_resolve_hot", |b| {
        b.iter(|| {
            let actions = chain
                .resolve("fake_net", "critical", "persistent")
                .unwrap();
            // Ensure the compiler can't fully optimize the call away
            assert_eq!(actions.len(), 2);
        });
    });
}

criterion_group!(benches, bench_load_only, bench_resolve_hot_path);
criterion_main!(benches);