#!/usr/bin/env python3
"""Bidirectional TCP forwarder so the OHOS emulator can reach a
host-localhost-bound service.

The emulator reaches the host at 10.0.2.2, but services bound to 127.0.0.1
(like octos on 9401) are not reachable from the guest. This listens on
0.0.0.0:<listen_port> and forwards to 127.0.0.1:<target_port>, so the app can
hit http://10.0.2.2:<listen_port>.

Usage: ohos_tcp_forward.py <listen_port> <target_port>
"""
import socket
import sys
import threading


def pipe(src, dst):
    try:
        while True:
            data = src.recv(65536)
            if not data:
                break
            dst.sendall(data)
    except OSError:
        pass
    finally:
        for s in (src, dst):
            try:
                s.shutdown(socket.SHUT_RDWR)
            except OSError:
                pass


def handle(client, target_host, target_port):
    try:
        upstream = socket.create_connection((target_host, target_port))
    except OSError as e:
        sys.stderr.write(f"[forward] upstream connect failed: {e}\n")
        client.close()
        return
    threading.Thread(target=pipe, args=(client, upstream), daemon=True).start()
    threading.Thread(target=pipe, args=(upstream, client), daemon=True).start()


def main():
    listen_port = int(sys.argv[1]) if len(sys.argv) > 1 else 19401
    target_port = int(sys.argv[2]) if len(sys.argv) > 2 else 9401
    target_host = "127.0.0.1"
    srv = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    srv.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    srv.bind(("0.0.0.0", listen_port))
    srv.listen(64)
    sys.stderr.write(
        f"[forward] 0.0.0.0:{listen_port} -> {target_host}:{target_port}\n"
    )
    while True:
        client, _ = srv.accept()
        threading.Thread(
            target=handle, args=(client, target_host, target_port), daemon=True
        ).start()


if __name__ == "__main__":
    main()
