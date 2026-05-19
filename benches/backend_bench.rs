//! Backend performance benchmark: C++ vs pure-Rust.
//!
//! This file is intentionally backend-agnostic: it drives only the safe
//! `rs_abieos::Abieos` public API, so the *exact same code path* is measured
//! regardless of which backend is compiled in. Pick the backend with feature
//! flags and use Criterion baselines to compare:
//!
//! ```text
//! cargo bench --no-default-features --features cpp-backend  -- --save-baseline cpp
//! cargo bench --no-default-features --features rust-backend -- --baseline cpp
//! ```
//!
//! The second run prints the Rust backend's change relative to the saved C++
//! baseline for every benchmark id.

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use rs_abieos::Abieos;

// eosio.token ABI in hex form (canonical fixture, mirrors tests::samples).
const EOSIO_TOKEN_HEX_ABI: &str = "0e656f73696f3a3a6162692f312e30010c6163636f756e745f6e616d65046e616d6505087472616e7366657200040466726f6d0c6163636f756e745f6e616d6502746f0c6163636f756e745f6e616d65087175616e74697479056173736574046d656d6f06737472696e67066372656174650002066973737565720c6163636f756e745f6e616d650e6d6178696d756d5f737570706c79056173736574056973737565000302746f0c6163636f756e745f6e616d65087175616e74697479056173736574046d656d6f06737472696e67076163636f756e7400010762616c616e63650561737365740e63757272656e63795f7374617473000306737570706c790561737365740a6d61785f737570706c79056173736574066973737565720c6163636f756e745f6e616d6503000000572d3ccdcd087472616e73666572000000000000a531760569737375650000000000a86cd445066372656174650002000000384f4d113203693634010863757272656e6379010675696e743634076163636f756e740000000000904dc603693634010863757272656e6379010675696e7436340e63757272656e63795f7374617473000000";

const EOSIO_TOKEN_U64: u64 = 6138663591592764928;

// One eosio.token `transfer` action, in the three interchange forms.
const TRANSFER_JSON: &str =
    r#"{"from":"alice","to":"bob","quantity":"1.0000 EOS","memo":"Hello!"}"#;
const TRANSFER_HEX: &str =
    "0000000000855C340000000000000E3D102700000000000004454F53000000000648656C6C6F21";
const TRANSFER_BIN: &[u8] = &[
    0, 0, 0, 0, 0, 133, 92, 52, 0, 0, 0, 0, 0, 0, 14, 61, 16, 39, 0, 0, 0, 0, 0, 0, 4, 69, 79, 83,
    0, 0, 0, 0, 6, 72, 101, 108, 108, 111, 33,
];

// The large real-world eosio system ABI, JSON and packed-binary forms.
const EOSIO_ABI_JSON: &str = include_str!("../abis/eosio.abi");
const EOSIO_ABI_BIN: &[u8] = include_bytes!("../abis/eosio.abi.bin");

/// Backend label baked in at compile time, so saved baselines/reports are
/// self-describing even when the benchmark id is identical across runs.
const BACKEND: &str = if cfg!(feature = "rust-backend") {
    "rust"
} else {
    "cpp"
};

fn token_ctx() -> Abieos {
    let a = Abieos::new();
    a.set_abi_hex("eosio.token", EOSIO_TOKEN_HEX_ABI)
        .expect("load eosio.token ABI");
    a
}

fn bench_context(c: &mut Criterion) {
    let mut g = c.benchmark_group("context");
    g.bench_function("create_destroy", |b| {
        b.iter(|| {
            let a = Abieos::new();
            black_box(a.as_ptr());
            // dropped here -> abieos_destroy
        });
    });
    g.finish();
}

fn bench_name(c: &mut Criterion) {
    let a = Abieos::new();
    let mut g = c.benchmark_group("name");
    g.bench_function("string_to_name", |b| {
        b.iter(|| black_box(a.string_to_name(black_box("eosio.token")).unwrap()));
    });
    g.bench_function("name_to_string", |b| {
        b.iter(|| black_box(a.name_to_string(black_box(EOSIO_TOKEN_U64)).unwrap()));
    });
    g.finish();
}

fn bench_abi_load(c: &mut Criterion) {
    let mut g = c.benchmark_group("abi_load");

    // Persistent context, ABI overwritten each iteration: isolates parse cost
    // from context lifecycle (which has its own benchmark).
    let a = Abieos::new();
    g.throughput(Throughput::Bytes(EOSIO_TOKEN_HEX_ABI.len() as u64));
    g.bench_function("set_abi_hex_token", |b| {
        b.iter(|| {
            a.set_abi_hex("eosio.token", black_box(EOSIO_TOKEN_HEX_ABI))
                .unwrap()
        });
    });

    g.throughput(Throughput::Bytes(EOSIO_ABI_JSON.len() as u64));
    g.bench_function("set_abi_json_eosio", |b| {
        b.iter(|| a.set_abi_json("eosio", black_box(EOSIO_ABI_JSON)).unwrap());
    });

    g.throughput(Throughput::Bytes(EOSIO_ABI_BIN.len() as u64));
    g.bench_function("set_abi_bin_eosio", |b| {
        b.iter(|| a.set_abi_bin("eosio", black_box(EOSIO_ABI_BIN)).unwrap());
    });
    g.finish();
}

fn bench_abi_convert(c: &mut Criterion) {
    let a = Abieos::new();
    let mut g = c.benchmark_group("abi_convert");

    g.throughput(Throughput::Bytes(EOSIO_ABI_JSON.len() as u64));
    g.bench_function("abi_json_to_bin_eosio", |b| {
        b.iter(|| black_box(a.abi_json_to_bin(black_box(EOSIO_ABI_JSON)).unwrap()));
    });

    g.throughput(Throughput::Bytes(EOSIO_ABI_BIN.len() as u64));
    g.bench_function("abi_bin_to_json_eosio", |b| {
        b.iter(|| black_box(a.abi_bin_to_json(black_box(EOSIO_ABI_BIN)).unwrap()));
    });
    g.finish();
}

fn bench_codec(c: &mut Criterion) {
    let a = token_ctx();
    let mut g = c.benchmark_group("codec");

    g.throughput(Throughput::Bytes(TRANSFER_JSON.len() as u64));
    g.bench_function("json_to_hex_transfer", |b| {
        b.iter(|| {
            black_box(
                a.json_to_hex("eosio.token", "transfer", black_box(TRANSFER_JSON))
                    .unwrap(),
            )
        });
    });
    g.bench_function("json_to_bin_transfer", |b| {
        b.iter(|| {
            black_box(
                a.json_to_bin("eosio.token", "transfer", black_box(TRANSFER_JSON))
                    .unwrap(),
            )
        });
    });

    g.throughput(Throughput::Bytes(TRANSFER_HEX.len() as u64));
    g.bench_function("hex_to_json_transfer", |b| {
        b.iter(|| {
            black_box(
                a.hex_to_json("eosio.token", "transfer", black_box(TRANSFER_HEX))
                    .unwrap(),
            )
        });
    });

    g.throughput(Throughput::Bytes(TRANSFER_BIN.len() as u64));
    g.bench_function("bin_to_json_transfer", |b| {
        b.iter(|| {
            black_box(
                a.bin_to_json("eosio.token", "transfer", black_box(TRANSFER_BIN))
                    .unwrap(),
            )
        });
    });

    // Cold path: fresh context + ABI load + serialize, the realistic
    // "decode one action from scratch" cost.
    g.throughput(Throughput::Bytes(TRANSFER_JSON.len() as u64));
    g.bench_function("cold_load_and_json_to_hex", |b| {
        b.iter_batched(
            Abieos::new,
            |ctx| {
                ctx.set_abi_hex("eosio.token", EOSIO_TOKEN_HEX_ABI).unwrap();
                black_box(
                    ctx.json_to_hex("eosio.token", "transfer", TRANSFER_JSON)
                        .unwrap(),
                )
            },
            BatchSize::SmallInput,
        );
    });
    g.finish();
}

fn label(c: &mut Criterion) {
    // Not a benchmark; just surfaces which backend produced this run in stdout.
    eprintln!("rs_abieos backend under benchmark: {BACKEND}");
    let _ = c;
}

criterion_group!(
    benches,
    label,
    bench_context,
    bench_name,
    bench_abi_load,
    bench_abi_convert,
    bench_codec
);
criterion_main!(benches);
