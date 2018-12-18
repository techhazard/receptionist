#!/bin/sh

# This script starts both receptionist and nginx
# If either quits, the other is stopped as well.

set -e
set -m

trap myhandler CHLD

myhandler() {
        set +e

        if ! kill -0 "$nginx_pid"; then
                echo "nginx was killed, stopping container" >&2

        elif ! kill -0 "$receptionist_pid"; then
                echo "receptionist was killed, stopping container" >&2

        else
                echo "nothing was killed?" >&2
                return
        fi

		# quit both and wait 3 seconds
		echo "sending sigquit" >&2
        kill -QUIT "$receptionist_pid" "$nginx_pid" >/dev/null 2>&1
        sleep 3

		# kill both and exit 1
		echo "sending sigkill" >&2
        kill -KILL "$receptionist_pid" "$nginx_pid" >/dev/null 2>&1
        exit 1
}


(receptionist) &
receptionist_pid=$!

(nginx -g 'daemon off;') &
nginx_pid=$!

echo "Started receptionist and nginx" >&2

wait
