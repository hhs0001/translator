"""Minimal OpenAI-compatible server for manually exercising the app.

Answers `/v1/models` and `/v1/chat/completions` (streaming and not), echoing
each `INDEX|TEXT` line back as `INDEX|[pt] TEXT` after a short delay so the
parallel batches and the live progress are visible in the UI.

Usage: python scripts/mock_api.py [port]
"""

import json
import re
import sys
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

DELAY_SECONDS = 1.2
LINE = re.compile(r"^(\d+)\|(.*)$")


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, fmt, *args):  # noqa: A002 - silence default logging
        sys.stderr.write("[mock] " + fmt % args + "\n")

    def do_GET(self):
        if self.path.rstrip("/").endswith("/models"):
            models = {
                "data": [
                    {"id": "mock-fast", "object": "model", "context_length": 128000},
                    {"id": "mock-smart", "object": "model", "context_length": 200000},
                ]
            }
            self._send_json(models)
        else:
            self._send_json({"error": "not found"}, status=404)

    def do_POST(self):
        length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(length).decode("utf-8", "replace")
        try:
            payload = json.loads(body)
        except json.JSONDecodeError:
            self._send_json({"error": "bad json"}, status=400)
            return

        prompt = "\n".join(
            message.get("content", "") for message in payload.get("messages", [])
        )
        translated = []
        for raw in prompt.splitlines():
            match = LINE.match(raw.strip())
            if match:
                translated.append(f"{match.group(1)}|[pt] {match.group(2)}")

        time.sleep(DELAY_SECONDS)

        if payload.get("stream"):
            self._send_stream(translated)
        else:
            self._send_json(
                {
                    "choices": [
                        {"message": {"role": "assistant", "content": "\n".join(translated)}}
                    ]
                }
            )

    def _send_json(self, data, status=200):
        encoded = json.dumps(data).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(encoded)))
        self.end_headers()
        self.wfile.write(encoded)

    def _send_stream(self, lines):
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Cache-Control", "no-cache")
        self.send_header("Connection", "close")
        self.end_headers()
        for line in lines:
            chunk = {"choices": [{"delta": {"content": line + "\n"}}]}
            self.wfile.write(f"data: {json.dumps(chunk)}\n\n".encode("utf-8"))
            self.wfile.flush()
            time.sleep(0.05)
        self.wfile.write(b"data: [DONE]\n\n")
        self.wfile.flush()


def main() -> None:
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8123
    server = ThreadingHTTPServer(("127.0.0.1", port), Handler)
    print(f"mock API listening on http://127.0.0.1:{port}/v1")
    server.serve_forever()


if __name__ == "__main__":
    main()
