#!/bin/bash

PASS_COUNTER=0
FAIL_COUNTER=0

cd tests || exit 1;

for script in ./*.sh; do 
	chmod +x "$script"

	output=$("$script")

	pass=$(echo "$output" | grep -ow "PASS" | wc -l)
	((PASS_COUNTER += pass))
	fail=$(echo "$output" | grep -ow "FAIL" | wc -l)
	((FAIL_COUNTER += fail)) 
done

echo "==RESULTS=="
printf '%s\n' "PASSED: $PASS_COUNTER"
printf '%s\n' "FAILED: $FAIL_COUNTER"
