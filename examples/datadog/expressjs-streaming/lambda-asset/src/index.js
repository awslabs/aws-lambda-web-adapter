const tracer = require("dd-trace").init();
tracer.use("http", {
  blocklist: ["/"],
});
const express = require("express");
const app = express();
const port = process.env["PORT"] || 8080;

// SIGTERM Handler
process.on("SIGTERM", async () => {
  console.info("[express] SIGTERM received");

  console.info("[express] cleaning up");
  // perform actual clean up work here.
  await new Promise((resolve) => setTimeout(resolve, 100));

  console.info("[express] exiting");
  process.exit(0);
});

app.get("/call_lwa", async (req, res) => {
  for (let i = 0; i < 5; i++) {
    const chunk = `Hello, world! (${i + 1})\n`;
    res.write(chunk);
    // Wait for 500ms
    await new Promise((resolve) => setTimeout(resolve, 500));
  }

  res.end();
});

app.listen(port, () => {
  console.log(`Example app listening at http://localhost:${port}`);
});
