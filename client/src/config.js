const axios = require("axios").default;
const request = axios.create({
  baseURL: "http://127.0.0.1:40543",
});
const { remote } = require("electron");

// Global state
let state = {
  connect: false,
  mute: false,
  deaf: false,
  overlay: false,
  users: {},
  volmap: {},
};

// Control Panel Buttons

document.getElementById("connect").addEventListener("click", () => {
  if (!state.connect) {
    setConnect(true);
    document.getElementById("connect").textContent = "Disconnect Voice";
    state.connect = true;
  } else if (state.connect) {
    setConnect(false);
    document.getElementById("connect").textContent = "Connect Voice";
    state.connect = false;
  }
  sync();
});

document.getElementById("mute").addEventListener("click", () => {
  state.mute ? setMute(false) : setMute(true);
  sync();
});

document.getElementById("deafen").addEventListener("click", () => {
  state.deaf ? setDeaf(false) : setDeaf(true);
  sync();
});

document.getElementById("overlay").addEventListener("click", () => {
  state.overlay ? setOverlay(false) : setOverlay(true);
  sync();
});

document.getElementById("settings").addEventListener("click", () => {
  openSettings();
  sync();
});

document.getElementById("sync").addEventListener("click", () => {
  sync();
});

// Usermap

document.getElementById("usermapsubmit"),
  addEventListener("click", async () => {
    newvolmap = JSON.parse(document.getElementById("usermapjson").value);
    try {
      await setVolmapping(newvolmap);
      console.log(newvolmap);
      state.volmap = newvolmap;
    } catch (e) {
      console.log(e);
    }
    document.getElementById("usermapjson").value = JSON.stringify(
      state.volmap,
      null,
      2
    );
    return false;
  });

// Sync Function
let sync = async () => {
  try {
    state.mute = await isMute();
    state.deaf = await isDeaf();
    state.overlay = await isOverlay();
    state.users = await getUsers();
  } catch (e) {
    console.log(e);
  }
  document.getElementById("mute").textContent = "Muted: " + state.mute;
  document.getElementById("deafen").textContent = "Deafened: " + state.deaf;
  document.getElementById("overlay").textContent = "Overlay locked: " + state.deaf;
  usergrid();
  console.log(state);
};

// User Grid Sync
let usergrid = () => {
  usergriddiv = document.getElementById("usergrid");
  while (usergriddiv.firstChild) {
    usergriddiv.removeChild(usergriddiv.firstChild);
  }
  for (const [user, dcid] of Object.entries(state.users)) {
    var div = document.createElement("div");
    div.id = dcid;
    div.className = "usergriditem";
    div.textContent = user;
    usergriddiv.appendChild(div);
  }
};

document.getElementById("devtools").addEventListener("click", async () => {
  remote.getCurrentWindow().loadFile("src/devtools.html");
});

/*
Functions to fetch data
isMute()
isDeaf()
setMute(boolean)
setDeaf(boolean)
setConnect(boolean)
getUsers()
getMapping()
getVolmapping()
openSettings()
*/

let isMute = async () => {
  return (await request.get("voice/mute")).data.status;
};

let isDeaf = async () => {
  return (await request.get("voice/deaf")).data.status;
};

let isOverlay = async () => {
  return (await request.get("overlay/lock")).data.status;
};

let setMute = async (set) => {
  console.log(
    await request.post("voice/mute", {
      status: set,
    })
  );
};

let setDeaf = async (set) => {
  console.log(
    await request.post("voice/deaf", {
      status: set,
    })
  );
};

let setOverlay = async (set) => {
  console.log(
    await request.post("overlay/lock", {
      status: set,
    })
  );
};

let setConnect = async (set) => {
  console.log(
    await request.post("voice/connect", {
      status: set,
    })
  );
};

let getUsers = async () => {
  return (await request.get("users")).data;
};

let setVolmapping = async (volmap) => {
  localStorage.setItem("volmap", JSON.stringify(volmap));
  console.log(await request.post("volmap", { volmap: volmap }));
};

let openSettings = async () => {
  console.log(await request.post("voice/settings", {}));
};

// Setup
let init = () => {
  state.volmap = JSON.parse(localStorage.getItem("volmap"));
  console.log(state);
  document.getElementById("usermapjson").value = JSON.stringify(
    state.volmap,
    null,
    2
  );
  sync();
};

init();

// Run sync cycle
setInterval(() => {
  sync();
}, 3000);
