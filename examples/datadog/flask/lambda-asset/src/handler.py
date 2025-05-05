import ddtrace

ddtrace.patch_all()
from ddtrace.trace import tracer, TraceFilter
import os
from flask import Flask, request, jsonify
import logging
import re

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

app = Flask(__name__)


class IgnoreEndpointFilter(TraceFilter):
    def __init__(self, pattern, method):
        self.pattern = re.compile(pattern)
        self.method = method

    def process_trace(self, trace):
        logger.info(f"Processing trace: {trace}")
        for span in trace:
            logger.info(f"Processing span: {span}")
            url = span.get_tag("http.url")
            logger.info(f"Span URL: {url}")
            logger.info(f"Pattern: {span.get_tag('http.method')}")
            if (
                url is not None
                and self.pattern.match(url)
                and self.method == span.get_tag("http.method")
            ):
                logger.info(f"Filtering out span with URL: {url}")
                return None
        return trace


tracer.configure(
    trace_processors=[IgnoreEndpointFilter(r"http://127.0.0.1:8080/", "GET")]
)


@app.route("/call_lwa", methods=["GET"])
def call_lwa():
    """
    Endpoint to handle POST requests to /lwa_call
    """
    with tracer.trace("lwa_call_endpoint", service="datadog-lwa-test-py") as span:
        return jsonify("ok"), 200


@app.route("/", methods=["GET"])
def health_check():
    print("AG: Health check endpoint called")
    """
    Simple health check endpoint
    """
    return jsonify({"status": "healthy"}), 200


port = int(os.environ.get("PORT", 5000))
app.run(host="0.0.0.0", port=port, debug=False)
