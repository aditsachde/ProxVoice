module.exports = {
    packagerConfig: {},
    makers: [
      {
        name: "@electron-forge/maker-zip",
        platforms: [
          "darwin", "win32", "linux"
        ]
      }
    ]
  }