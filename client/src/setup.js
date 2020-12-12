const axios = require("axios").default;
const request = axios.create({
  baseURL: "http://127.0.0.1:40543",
});
const { remote } = require("electron");

document.getElementById("setup").addEventListener("submit", async () => {
  lobbyid = document.getElementById("lobbyid").value;
  lobbysecret = document.getElementById("lobbysecret").value;
  response = await request.post("/setup", {
    id: lobbyid,
    secret: lobbysecret,
  });
  await new Promise(r => setTimeout(r, 5000));
  response = await request.post("/zmq");
});

setInterval(async () => {
  try {
    response = await request.get("/started");
    if (response.data.started) {
      console.log("started");
      remote.getCurrentWindow().loadFile("src/config.html");
    }
  } catch (e) {
    console.log(e);
  }
}, 500);
