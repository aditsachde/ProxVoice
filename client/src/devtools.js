const axios = require("axios").default;
const request = axios.create({
  baseURL: "http://127.0.0.1:40543",
});
const { remote } = require("electron");

document.getElementById("back").addEventListener("click", async () => {
  remote.getCurrentWindow().loadFile("src/config.html");
});

setInterval(async () => {
  try {
    response = await request.get("/globals");
  } catch (e) {
    console.log(e);
  }

  document.getElementById("globals").textContent = JSON.stringify(
    response.data,
    null,
    2
  );
}, 1000);
