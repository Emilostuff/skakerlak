build:
    cargo b -r

run:
    just build
    cd target/release && ./runner

debug:
    @tail -f target/release/engine.log
