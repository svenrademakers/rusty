
const { contextBridge } = require('electron')
const { wat } = require('../daemon/grpc_client');

contextBridge.exposeInMainWorld('myAPI', {
  wat: wat
})

window.addEventListener('DOMContentLoaded', () => {
  const replaceText = (selector, text) => {
    const element = document.getElementById(selector)
    if (element) element.innerText = text
  }

  for (const dependency of ['chrome', 'node', 'electron']) {
    replaceText(`${dependency}-version`, process.versions[dependency])
  }
})
