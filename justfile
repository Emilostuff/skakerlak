build:
    cargo b -r

run:
    just build
    cd target/release && ./runner
