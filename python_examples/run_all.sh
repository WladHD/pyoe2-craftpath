#!/bin/bash
for file in *.py; do
    echo "Running $file..."
    if ! python "$file"; then
        echo "❌ $file failed. Exiting."
        exit 1
    fi
done
