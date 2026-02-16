build:
    cargo b -r

run:
    just build
    cd target/release && ./runner

watch:
    @tail -f target/release/engine.log

test arg="":
    cargo nextest run -r {{ arg }}

run-raw:
    cargo r -r --bin skakarlak

bench name:
    cargo bench --bench {{ name }};
