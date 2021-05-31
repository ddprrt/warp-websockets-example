let log = console.log;

const wsUri =
  ((window.location.protocol == "https:" && "wss://") || "ws://") +
  window.location.host +
  "/ws";
conn = new WebSocket(wsUri);

log("Connecting...");

conn.onopen = function () {
  log("Connected.");
};

conn.onmessage = function (e) {
  log("Received: " + e.data);
  document.getElementById("log").textContent =
    document.getElementById("log").textContent + "\n" + e.data;
};

conn.onclose = function () {
  log("Disconnected.");
  conn = null;
};

function send() {
  conn.send(document.getElementById("input").value);
}

document.getElementById("btn").addEventListener("click", send);
