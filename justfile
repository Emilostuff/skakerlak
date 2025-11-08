build:
    cargo b -r

run:
    just build
    cd target/release && ./runner

watch:
    @tail -f target/release/engine.log
