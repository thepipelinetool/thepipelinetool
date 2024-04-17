cargo install --path thepipelinetool --force --debug
cargo install --path thepipelinetool_server --bin tpt_executor --force --debug
./scripts/refresh_cache.sh 
./scripts/run_local_server_dev.sh 