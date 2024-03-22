#!/bin/bash

# Make sure this file is clean before running dir iteration
> list

if [[ -z $1 ]]; then
    echo "Please provide a regex to filter out false positivies"
    exit 1
fi

for dir in $(ls pallets); do 
    if [[ $dir == "mock" ]]; then 
        continue; 
    fi;

    echo pallets/$dir >> list
done

for dir in $(ls runtime); do 
    if [[ $dir == "mock" ]]; then 
        continue; 
    fi;

    echo runtime/$dir >> list
done

ERRORS=false

echo 🔎 Subalfred feature checks
for dir in $(cat list); do 
    echo 
    RESULT=$(subalfred check features $dir)
    CHECK_RESULT=$? # 0 if it's good, anything else is bad 

    # If subalfred fails with 130||1, then we dont want to proceed with the check
    # Its probably cargo error
    if [[ $CHECK_RESULT == 130 || $CHECK_RESULT == 1 ]]; then
        echo "❌ Subalfred failed to run check features in $dir"
        echo "$RESULT"
        ERRORS=true 

        continue
    fi

    # Sanitizing subalfred output
    # First line is always "checking: $PATH/Cargo.toml"
    RESULT=$(echo "$RESULT" | tail -n+2)

    # Filter out false positives
    RESULT_OUTPUT=$(echo "$RESULT" | grep -vE "($1)")
    # Trim whitespaces
    RESULT_OUTPUT=${RESULT_OUTPUT##*( )}

    # We are checking here if there is anything left in the output after filtering out false positives
    if [[ "$RESULT_OUTPUT" == "" ]]; then
        echo "✅ $dir"
        continue
    fi

    echo "$RESULT_OUTPUT" | grep '`std`' > /dev/null
    GREP_RESULT=$? # 0 if it's bad, 1 if it's good

    # If result is non empty and there are no std features, then we're yellow
    if [[ "$GREP_RESULT" == 1 && "$CHECK_RESULT" != 0 && "$RESULT_OUTPUT" != "" ]]; then
        echo "🟡 $dir"
        echo -e "$RESULT_OUTPUT"

    # If there are std errors, then we're red
    else
        echo "❌ $dir"
        echo -e "$RESULT_OUTPUT"
        ERRORS=true
    fi
done

if [[ $ERRORS == true ]]; then
    exit 1
fi

rm list