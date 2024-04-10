rm -rf ./bin/*
cargo install --path examples --examples --root . --force --no-track
cp examples/examples/yaml/**.yaml bin/

