cargo install --path thepipelinetool --force --debug
cargo install --path thepipelinetool_server --bin tpt_executor --force --debug
sudo ./scripts/refresh_cache.sh 
./scripts/run_local_server.sh 