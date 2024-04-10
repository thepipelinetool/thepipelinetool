./scripts/build_examples.sh
cargo install --path thepipelinetool --force

for f in $(ls ./bin)
do
    echo "running example: $f"

    if [[ "$f" == "params"* ]]; then
        tpt ./bin/$f run in_memory --params '{"data": "hello world"}' --max_parallelism $1
        retVal=$?
    elif [[ "$f" == "fail"* ]]; then
        tpt ./bin/$f run in_memory --max_parallelism $1
        retVal=$?
        if [ $retVal -eq 0 ]; then
            echo "error: $f should have failed"
            exit 1
        fi
    else
        tpt ./bin/$f run in_memory --max_parallelism $1
        retVal=$?
        if [ $retVal -ne 0 ]; then
            echo "error: $f failed"
            exit 1
        fi
    fi
done