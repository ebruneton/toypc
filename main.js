// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License along with
// this program. If not, see <https://www.gnu.org/licenses/>.

(function (Screen, Keyboard, Terminal) {

  const createBackupFile = function (flashContent, runFromFlash) {
    let address = 0x80000;
    let result = '';
    for (const word of flashContent) {
      if (word != 0xFFFFFFFF) {
        const addressStr = address.toString(16).toUpperCase();
        const wordStr = word.toString(16).toUpperCase().padStart(8, '0');
        result += `W${addressStr},${wordStr}#\n`;
      }
      address += 4;
    }
    result += 'flash#\n';
    if (runFromFlash) {
      result += 'reset#\n';
    }
    return result;
  }

  class Main {
    constructor() {
      this.worker = new Worker('worker.js');
      this.worker.onmessage = (event) => this.onWorkerMessage(event.data);
      this.workerInitialized = false;
      this.screen = new Screen(document.getElementById('screen'));
      this.screenInitialized = false;
      this.led = document.getElementById('led');
      this.keyboard = new Keyboard();
      this.terminal = new Terminal(document.getElementById('terminal'), this.worker);

      this.worker.postMessage(document.baseURI);
      this.screen.initialize(() => {
        this.screenInitialized = true;
        this.maybeStart();
      });
    }

    onWorkerMessage(message) {
      if (message.type == 'initialized') {
        this.workerInitialized = true;
        this.maybeStart();
      } else if (message.type == 'output' || message.type == 'exit') {
        this.terminal.onMessage(message);
      } else if (message.type == 'on_save') {
        this.onSave(message.flashContent, message.runFromFlash);
      } else if (message.type == 'panic') {
        this.onPanic(message.data);
      } else if (message.type == 'led') {
        if (message.on) {
          this.led.classList.add('on');
        } else {
          this.led.classList.remove('on');
        }
      } else {
        this.screen.onMessage(message);
      }
    }

    maybeStart() {
      if (!this.screenInitialized || !this.workerInitialized) { return; }

      document.addEventListener('keydown', (event) => this.onKeyEvent(event));
      document.addEventListener('keyup', (event) => this.onKeyEvent(event));
      document.getElementById('reset').addEventListener('click', () => {
        this.worker.postMessage({ type: 'reset' });
      });
      document.getElementById('load').addEventListener('click', () => this.onLoad());
      document.getElementById('save').addEventListener('click', () => {
        this.worker.postMessage({ type: 'save' });
      });
      this.terminal.initialize();
      const script = new URLSearchParams(window.location.search).get('script');
      if (script) {
        this.terminal.run(`python3 flash_helper.py < ${script}`);
      }
    }

    onKeyEvent(event) {
      event.preventDefault();
      for (const scancode of this.keyboard.getScanCodes(event)) {
        this.worker.postMessage({ type: 'scancode', data: scancode });
      }
    }

    onLoad() {
      const element = document.createElement('input');
      element.type = 'file';
      element.accept = 'text/plain'
      element.addEventListener('change', () => {
        if (element.files.length == 1) {
          this.terminal.load(element.files[0]);
        }
      });
      element.click();
    }

    onSave(flashContent, runFromFlash) {
      const fileContent = createBackupFile(flashContent, runFromFlash);
      const fileBlob = new Blob([fileContent], { type: 'text/plain' });
      const fileUrl = URL.createObjectURL(fileBlob);
      const element = document.createElement('a');
      element.setAttribute("href", fileUrl);
      element.setAttribute("download", 'backup.txt');
      element.click();
      URL.revokeObjectURL(fileUrl);
    }

    onPanic(error) {
      document.getElementById('panic-message').innerText = error;
      document.getElementById('panic').style.display = 'block';
    }
  }

  window.addEventListener('DOMContentLoaded', () => {
    new Main();
  });

})(ToyPcEmulator.Screen, ToyPcEmulator.Keyboard, ToyPcEmulator.Terminal);
