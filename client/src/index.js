const { app, BrowserWindow } = require("electron");
var path = require("path");
const axios = require("axios").default;
const request = axios.create({
  baseURL: "http://127.0.0.1:40543",
});

function createWindow() {
  const win = new BrowserWindow({
    width: 600,
    height: 800,
    webPreferences: {
      nodeIntegration: true,
      enableRemoteModule: true,
    },
  });

  win.loadFile("src/setup.html");
}

app.whenReady().then(createWindow);

app.on("activate", () => {
  if (BrowserWindow.getAllWindows().length === 0) {
    createWindow();
  }
});

const PY_DIST_FOLDER = "dist";
const PY_MODULE = "webserver";
let webserver = null;

const getScriptPath = () => {
  if (process.platform === "win32") {
    return path.join(__dirname, PY_DIST_FOLDER, PY_MODULE, PY_MODULE + ".exe");
  }
  return path.join(__dirname, PY_DIST_FOLDER, PY_MODULE, PY_MODULE);
};

const getCwdPath = () => {
  return path.join(__dirname, PY_DIST_FOLDER, PY_MODULE);
};

const createPyProc = () => {
  if (process.env.DEV != "true") {
    let script = getScriptPath();
    let cwd = getCwdPath();
    webserver = require("child_process").execFile(
      script,
      { cwd: cwd },
      (error, stdout, stderr) => {
        if (error) {
          throw error;
        }
        console.log(stdout);
      }
    );
  }

  if (webserver != null) {
    console.log("Python webserver started");
  }
};

app.on("ready", createPyProc);
app.on("will-quit", async () => {
  console.log(await request.post("/leavelobby"));
  webserver.kill("SIGTERM");
  await new Promise(r => setTimeout(r, 1000));
  webserver.kill("SIGKILL");
  webserver = null;
});
