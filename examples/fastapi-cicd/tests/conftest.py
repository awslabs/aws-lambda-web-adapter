import time
from collections.abc import Callable
from contextlib import contextmanager
from http.client import HTTPConnection
from subprocess import Popen
from typing import Any


def gen_background_server_ctxmanager(
    cmd: list | None = None,
    cwd: str = ".",
    port: int = 8000,
    healthendpoint: str = "/",
    wait_seconds: int = 5,
    **kwargs: Any,
) -> Callable:
    if cmd is None:
        cmd = ["python", "-m", "http.server"]

    @contextmanager
    def server():
        print(f"opening server {cmd=} at {cwd=}")
        retries = 10
        process = Popen(cmd, cwd=cwd, **kwargs)
        # give it 1 second right off the bat
        time.sleep(1)
        while retries > 0:
            conn = HTTPConnection(f"localhost:{port}", timeout=10)
            try:
                conn.request("HEAD", healthendpoint)
                response = conn.getresponse()
                if response is not None:
                    print(f"health check for {cmd=} got a response")
                    conn.close()
                    time.sleep(1)  # Give server time to stabilize
                    yield process
                    break
            except (ConnectionRefusedError, ConnectionResetError, OSError) as e:
                # ConnectionRefusedError: server not listening yet
                # ConnectionResetError: server accepted connection but reset it
                # OSError: other network errors (e.g., connection aborted)
                print(f"failed health check for {cmd=} ({e!r}), waiting {wait_seconds=}")
                conn.close()
                time.sleep(wait_seconds)
                retries -= 1

        if not retries:
            raise RuntimeError(f"Failed to start server at {port=}")
        else:
            print(f"terminating process after {retries}")
            # do it twice for good measure
            process.terminate()
            time.sleep(1)
            process.terminate()
            time.sleep(1)

    return server
