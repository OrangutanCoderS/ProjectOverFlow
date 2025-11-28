use criterion::{criterion_group, criterion_main, Criterion};
use action_spawn_mirror::{ActionSpawnMirror, MirrorRequest};

pub fn bench_spawn_mirror(c: &mut Criterion) {
    let mgr = ActionSpawnMirror::default();
    let pid = std::process::id() as i32;
    let req = MirrorRequest { pid, sandboxed: false, context: Some("bench".into()) };

    c.bench_function("mirror_spawn", |b| b.iter(|| mgr.execute(req.clone())));
}

criterion_group!(benches, bench_spawn_mirror);
criterion_main!(benches);