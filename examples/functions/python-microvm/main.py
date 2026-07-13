from http.server import HTTPServer, BaseHTTPRequestHandler
import json, time, os

class Handler(BaseHTTPRequestHandler):
    start_time = time.time()
    request_count = 0

    def do_GET(self):
        Handler.request_count += 1
        body = json.dumps({
            "message": "Hello from Lambda MicroVM!",
            "uptime_seconds": round(time.time() - Handler.start_time, 2),
            "requests_served": Handler.request_count,
            "pid": os.getpid()
        })
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(body.encode())

HTTPServer(("0.0.0.0", 8080), Handler).serve_forever()
