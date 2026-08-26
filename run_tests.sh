#!/bin/bash

# It has one limitation for now, it only goes one directory up to search for the
# tests folder and inside the test folder if it found one
# 
# It doesn't go inside any directory to search for test scripts
# unless its name is tests/

PASS_COUNTER=0
FAIL_COUNTER=0

test_folder="tests"
server_folder="backend"

server_commands=(
    "cargo run"
    "cargo run -p server"
)

BASE_URL='http://127.0.0.1:8080'
RESPONSE_FILE=$(mktemp) || exit 1
server_pid=""
START_DIR="$PWD"

stop_started_server() {
    if [[ -n "$server_pid" ]] && kill -0 "$server_pid" 2>/dev/null; then
        kill -TERM -- "-$server_pid" 2>/dev/null || kill "$server_pid" 2>/dev/null
        wait "$server_pid" 2>/dev/null
    fi

    server_pid=""
}

cleanup() {
    stop_started_server
    rm -f -- "$RESPONSE_FILE"
}

trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

request_status() {
    curl --silent --show-error \
    --connect-timeout 1 \
    --max-time 2 \
    --output "$RESPONSE_FILE" \
    --write-out "%{http_code}" \
    "$1"
}

start_server() {
    local attempt
    local cmd
    local curl_status
    local health_status
    local status

    for cmd in "${server_commands[@]}"; do
        echo "Running: $cmd"
        setsid bash -c "$cmd" &
        server_pid=$!

        # Starting or compiling the server can take a moment, so wait for it.
        for ((attempt = 1; attempt <= 60; attempt++)); do
            health_status=$(request_status "$BASE_URL/health" 2>/dev/null)
            curl_status=$?

            if [[ $curl_status -eq 0 && $health_status -eq 200 ]]; then
                echo "Server is running and is healthy!"
                return 0
            fi

            if ! kill -0 "$server_pid" 2>/dev/null; then
                wait "$server_pid"
                status=$?
                echo "Command $cmd failed with status $status"
                server_pid=""
                break
            fi

            sleep 1
        done

        if [[ -n "$server_pid" ]]; then
            echo "Command $cmd started, but the server did not become healthy in time."
            stop_started_server
        fi
    done

    echo "All commands exhausted but server isn't running.."
    echo "Aborting now.."
    return 1
}

kill_server() {
    local attempt
    local cmd
    local cwd
    local failed=0
    local found=0
    local health_status
    local pid

    if [[ -n "$server_pid" ]]; then
        stop_started_server
        return 0
    fi

    for cmd in "${server_commands[@]}"; do
        for pid in $(pgrep -f "$cmd"); do
            cwd=$(readlink -f "/proc/$pid/cwd")

            if [[ "$cwd" == "$SERVER_DIR" ]]; then
                found=1
                kill "$pid" || failed=1
            fi
        done
    done

    if [[ $found -eq 0 || $failed -ne 0 ]]; then
        return 1
    fi

    # Do not restart until the old server has actually released the endpoint.
    for ((attempt = 1; attempt <= 20; attempt++)); do
        health_status=$(request_status "$BASE_URL/health" 2>/dev/null)
        if [[ $? -ne 0 ]]; then
            return 0
        fi
        sleep 0.25
    done

    return 1
}

run_tests() {
    local fail
    local original_dir="$PWD"
    local output
    local pass
    local result=0
    local script
    local test_status

    if [[ ! -d "$test_folder" ]]; then
        cd .. || return 1
    fi

    if [[ ! -d "$test_folder" ]]; then
        printf "%s\n\n" "No tests directory found in $PWD"
        cd "$original_dir" || return 1
        return 1
    fi

    printf "%s\n" "Found '$test_folder' folder in:"
    printf "\e[1;33m%s\e[0m\n" "$(ls)"
    if ! cd "$test_folder"; then
        cd "$original_dir" || return 1
        return 1
    fi
    printf "\n%s\n\n" "Right now in $PWD"

    local scripts=(./*.sh)
    if [[ ! -e "${scripts[0]}" ]]; then
        printf "%s\n" "No test scripts found in $PWD"
        cd "$original_dir" || return 1
        return 1
    fi

    for script in "${scripts[@]}"; do
        chmod +x "$script"

        # Run all the scripts and then cleanup.sh only at last
        if [[ ! $script == "cleanup.sh" ]]; then 
            output=$("$script" 2>&1)
            test_status=$?
            printf "%s\n" "$output"
    
            pass=$(echo "$output" | grep -ow "PASS" | wc -l)
            ((PASS_COUNTER += pass))
            fail=$(echo "$output" | grep -ow "FAIL" | wc -l)
            ((FAIL_COUNTER += fail))
    
            if [[ $test_status -ne 0 ]]; then
                result=1
                if [[ $fail -eq 0 ]]; then
                    ((FAIL_COUNTER += 1))
                fi
            elif [[ $fail -gt 0 ]]; then
                result=1
            fi
        fi
    done

     # Remembering syntax fo this operation..
     output=$(bash "./cleanup.sh")
     test_status=$?
     printf "%s\n" "$output"

     pass=$(echo "$output" | grep -ow "PASS" | wc -l)
     ((PASS_COUNTER += pass))
     fail=$(echo "$output" | grep -ow "FAIL" | wc -l)
     ((FAIL_COUNTER += fail))

     if [[ $test_status -ne 0 ]]; then
         result=1
         if [[ $fail -eq 0 ]]; then
             ((FAIL_COUNTER += 1))
         fi
     elif [[ $fail -gt 0 ]]; then
         result=1
     fi

    cd "$original_dir" || return 1
    return "$result"
}

if [[ ! -d "$server_folder" ]]; then
    cd .. || exit 1
fi

if [[ ! -d "$server_folder" ]]; then
    printf "%s\n" "No '$server_folder' directory found in $START_DIR or its parent."
    exit 1
fi

printf "%s\n" "Found '$server_folder' folder in:"
printf "\e[1;33m%s\e[0m\n" "$(ls)"

cd "$server_folder" || {
    echo "Couldn't go to $server_folder due to some reason"
    echo "Aborting.."
    exit 1
}
SERVER_DIR="$PWD"
printf "\n%s\n\n" "Right now in $PWD"

# First check if a server is already running.
health_status=$(request_status "$BASE_URL/health" 2>/dev/null)
curl_status=$?

if [[ $curl_status -eq 0 && $health_status -eq 200 ]]; then
    echo "Server is already running and healthy."
    echo "Re-running it to make sure all changes have been compiled.."

    if kill_server; then
        echo "Server killed successfully!"
    else
        echo "Couldn't find or stop the existing server safely."
        exit 1
    fi
elif [[ $curl_status -eq 0 ]]; then
    echo "Server is reachable but returned HTTP $health_status instead of 200."
    exit 1
else
    echo "Server is not already running."
fi

echo "Starting the server now.."
start_server || exit 1

echo "Trying to run tests now.."
run_tests
test_status=$?

stop_started_server
cd "$START_DIR" || exit 1
    

printf "\n\e[1;33m%s\e[0m\n" "===RESULTS==="
printf "\e[1;33m%s\e[0m" "PASSED: "
echo "$PASS_COUNTER"
printf "\e[1;33m%s\e[0m" "FAILED: "
echo "$FAIL_COUNTER"
printf "\n"

exit "$test_status"
