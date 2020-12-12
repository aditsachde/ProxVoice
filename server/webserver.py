from typing import Dict
import discordsdk as dsdk
import json
import zmq
from fastapi import FastAPI, BackgroundTasks
from pydantic import BaseModel
import asyncio
import uvicorn
from discordsdk.model import Lobby

# --------------------------------------------------------------------------------------

discord = dsdk.Discord(785964309799764018, dsdk.CreateFlags.default)
lobbyid = 0
lobbysecret = ""
maxdistance = 64
app = FastAPI()

mcid_user_map = {}
user_vol_map = {}
user_dcid_map = {}
dcid_user_map = {}
mcid_dcid_map = {}
dcid_vol_map = {}
connected = False
zmqstarted = False

user_manager = discord.get_user_manager()
voice_manager = discord.get_voice_manager()
activity_manager = discord.get_activity_manager()
lobby_manager = discord.get_lobby_manager()
overlay_manager = discord.get_overlay_manager()

# --------------------------------------------------------------------------------------


class Lobby(BaseModel):
    id: int
    secret: str


@app.post('/setup')
async def setup_client(background_tasks: BackgroundTasks, lobby: Lobby):
    global lobbyid
    global lobbysecret
    lobbyid = lobby.id
    lobbysecret = lobby.secret
    lobby_manager.connect_lobby(lobby.id, lobby.secret, lobby_callback)

    background_tasks.add_task(run_callbacks)
    return {"message": "success! now running!"}


async def run_callbacks():
    while True:
        await asyncio.sleep(1/10)
        discord.run_callbacks()


@app.post('/zmq')
async def start_zmq(background_tasks: BackgroundTasks):
    background_tasks.add_task(zmq_vol)
    return {"message": "success! now running zmq!"}


async def zmq_vol():
    msg = ""

    ctx = zmq.Context.instance()
    s = ctx.socket(zmq.SUB)
    url = 'tcp://127.0.0.1:40544'
    s.connect(url)
    s.setsockopt_string(zmq.SUBSCRIBE, "")
    global zmqstarted
    zmqstarted = True
    previous = {}
    while True:
        users = {}
        for k in dcid_user_map:
            users[k] = 0
        while msg != b'frame':
            await asyncio.sleep(1/150)
            try:
                msg = s.recv(flags=zmq.NOBLOCK)
                print(msg)
                if msg != b'frame':
                    msg = msg.decode("utf-8")
                    mcid, dist = msg.split()
                    dcid = mcid_dcid_map.get(mcid)
                    print("MCID", mcid, "DCID", dcid)
                    print("DCID in set", dcid in users)
                    if dcid is not None and dcid in users:
                        print("Distance:", dist)
                        vol = convert_distance(dcid, int(dist))
                        print("Volume:", vol)
                        users[dcid] = vol
                        print("-  -  -  -  -  -  -  -  -  -")
            except zmq.Again:
                pass
        # Returns list of sets
        diff = set(users.items()-previous.items())
        previous = users
        print(users, previous, diff)
        print("-----------------------------------------")
        for user_id, vol in diff:
            voice_manager.set_local_volume(user_id, vol)
        msg = ""


def convert_distance(dcid: str, distance: int) -> int:
    volmapval = int(dcid_vol_map.get(dcid))
    if volmapval is None:
        volmapval = 100
    return int(((max(maxdistance-distance, 0))/maxdistance) * volmapval)

##############################################################################################
######################## GLOBALS MODIFIED BELOW ##############################################
##############################################################################################


def on_lobby_update(_):
    try:
        mapping = lobby_manager.get_lobby_metadata_value(lobbyid, "mapping")
        global mcid_user_map
        mcid_user_map = json.loads(mapping)
    except dsdk.DiscordException as e:
        print(e)
    sync_globals()


lobby_manager.on_lobby_update = on_lobby_update


def on_member_update(_1, _2):
    count = lobby_manager.member_count(lobbyid)
    users = {}
    for i in range(count):
        id = lobby_manager.get_member_user_id(lobbyid, i)
        user = lobby_manager.get_member_user(lobbyid, id)
        username = user.username + user.discriminator
        users[username] = user.id
    global user_dcid_map, dcid_user_map
    user_dcid_map = users
    dcid_user_map = {v: k for k, v in user_dcid_map.items()}
    sync_globals()


lobby_manager.on_member_connect = on_member_update
lobby_manager.on_member_disconnect = on_member_update


class Volmap(BaseModel):
    volmap: Dict[str, int]


@app.post('/volmap')
async def set_volmap(volmap: Volmap):
    global user_vol_map
    user_vol_map = volmap.volmap
    sync_globals()
    return {"success": 200}


def sync_globals():
    dcid_vol = {}
    for dcid, user in dcid_user_map.items():
        vol = user_vol_map.get(user)
        if vol is not None:
            dcid_vol[dcid] = vol
        else:
            dcid_vol[dcid] = 100
    global dcid_vol_map
    dcid_vol_map = dcid_vol

    mcid_dcid = {}
    for mcid, user in mcid_user_map.items():
        dcid = user_dcid_map.get(user)
        if dcid is not None:
            mcid_dcid[mcid] = dcid
    global mcid_dcid_map
    mcid_dcid_map = mcid_dcid

##############################################################################################
######################## GLOBALS MODIFIED ABOVE ##############################################
##############################################################################################


@app.get('/usermap')
async def get_usermap():
    return mcid_user_map


@app.get('/volmap')
async def get_volmap():
    return user_vol_map


class Status(BaseModel):
    status: bool


@app.post('/voice/connect')
async def connect_voice(status: Status):
    if status.status:
        lobby_manager.connect_voice(lobbyid, callback)
    elif not status.status:
        lobby_manager.disconnect_voice(lobbyid, callback)
    return {"success": "voice set!"}


@app.post('/voice/settings')
async def open_settings():
    overlay_manager.open_voice_settings(callback)
    return {"success": "settings opened!"}


@app.get('/voice/mute')
async def is_mute():
    return {"status": voice_manager.is_self_mute()}


@app.post('/voice/mute')
async def set_mute(status: Status):
    voice_manager.set_self_mute(status.status)
    return {"success": 200}


@app.get('/voice/deaf')
async def is_deaf():
    return {"status": voice_manager.is_self_deaf()}


@app.post('/voice/mute')
async def set_deaf(status: Status):
    voice_manager.set_self_deaf(status.status)
    return {"success": 200}


@app.get('/volume/{user_id}')
async def get_volume(user_id: int):
    # What happens if user is not in lobby? Returns 100
    volume = voice_manager.get_local_volume(user_id)
    return {"volume": volume}


@app.get('/users')
async def get_users():
    return user_dcid_map


@app.get('/started')
async def get_started():
    return {"started": (connected and zmqstarted)}


@app.get('/overlay/lock')
async def get_overlay():
    return {"status": overlay_manager.is_locked()}


@app.post('/overlay/lock')
async def set_overlay(status: Status):
    overlay_manager.set_locked(status.status, callback)
    return {"success": 200}


@app.post('/leavelobby')
async def leave_lobby():
    lobby_manager.disconnect_lobby(lobbyid, callback)
    return {"success": 200}

# --------------------------------------------------------------------------------------


def callback(result):
    if result == dsdk.Result.ok:
        print("Successfully performed action!")
    else:
        raise Exception(result)


def lobby_callback(result, lobby: Lobby):
    if result == dsdk.Result.ok:
        on_lobby_update(1)
        on_member_update(1, 2)
        sync_globals()
        print("Successfully set the lobby!")
        print(lobby.id)
        print(lobby.secret)
        global connected
        connected = True
    else:
        raise Exception(result)


@app.get('/globals')
async def get_globals():
    return {"mcid_user_map": mcid_user_map,
            "user_vol_map": user_vol_map,
            "dcid_vol_map": dcid_vol_map,
            "user_dcid_map": user_dcid_map,
            "dcid_user_map": dcid_user_map,
            "mcid_dcid_map": mcid_dcid_map}

if __name__ == "__main__":
    uvicorn.run("webserver:app", host="127.0.0.1",
                port=40543, log_level="info", loop="asyncio")
