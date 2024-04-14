cargo install --path thepipelinetool --force
cargo install --path thepipelinetool_server --bin tpt_executor --force
./scripts/refresh_cache.sh 
./scripts/run_local_server_release.sh 