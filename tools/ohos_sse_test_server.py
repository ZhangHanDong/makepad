#!/usr/bin/env python3
"""Minimal SSE server for the OHOS simulator streaming smoke.

Serves GET /sse with five server-sent events, one per second, then closes.
The OHOS emulator reaches the host at 10.0.2.2, so run this on the host and
point the streaming smoke at http://10.0.2.2:8765/sse.
"""
import http.server
import sys
import time


class SseHandler(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def do_GET(self):
        if self.path != "/sse":
            self.send_response(404)
            self.send_header("Content-Length", "0")
            self.end_headers()
            return
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Cache-Control", "no-cache")
        self.send_header("Connection", "close")
        self.end_headers()
        for i in range(5):
            chunk = f"data: ohos sse event {i}\n\n".encode()
            self.wfile.write(chunk)
            self.wfile.flush()
            self.log_message("sent event %d (%d bytes)", i, len(chunk))
            time.sleep(1)

    def log_message(self, fmt, *args):
        sys.stderr.write("[sse-server] " + (fmt % args) + "\n")


if __name__ == "__main__":
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8765
    server = http.server.HTTPServer(("0.0.0.0", port), SseHandler)
    sys.stderr.write(f"[sse-server] listening on 0.0.0.0:{port}\n")
    server.serve_forever()
