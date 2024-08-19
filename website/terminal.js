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

(function () {
  const MAX_LINES = 256;

  class Terminal {
    constructor(root, worker) {
      this.root = root;
      this.worker = worker;
      this.inputLine = root.querySelector('#input-line');
      this.userInput = root.querySelector('#user-input');
      this.shellMode = true;
    }

    initialize() {
      this.root.style.display = 'block';
      this.root.addEventListener('click', () => this.userInput.focus());
      this.root.addEventListener('keydown', (event) => event.stopPropagation());
      this.root.addEventListener('keyup', (event) => event.stopPropagation());
      this.root.addEventListener('keypress', (event) => this.onKeyPress(event));
    }

    run(command) {
      this.userInput.innerText = command;
      this.onEnter(true);
    }

    load(file) {
      this.reset();
      this.userInput.innerText = `python3 flash_helper.py < ${file.name}`;
      this.onEnter(false);
      this.worker.postMessage({ type: 'load', data: file });
    }

    onKeyPress(event) {
      event.stopPropagation();
      if (event.key == 'Enter') {
        event.preventDefault();
        this.onEnter(true);
      }
    }

    onEnter(sendToSerialPort) {
      const input = this.userInput.innerText;
      if (this.shellMode) {
        const command = document.createElement('span');
        command.innerText = input;
        const commandLine = document.createElement('div');
        commandLine.appendChild(this.inputLine.children[0].cloneNode(true));
        commandLine.appendChild(command);
        this.appendChild(commandLine);
        if (input.length > 0) {
          const outputline = document.createElement('div');
          outputline.classList.add('inline');
          this.appendChild(outputline);
          this.inputLine.children[0].style.display = 'none';
          this.inputLine.classList.add('inline');
          this.userInput.innerText = '';
          this.shellMode = false;
        }
      } else {
        this.appendOutput(input + '\n');
        this.userInput.innerText = '';
      }
      if (sendToSerialPort && input.length > 0) {
        this.worker.postMessage({ type: 'serial', data: input });
      }
    }

    onMessage(message) {
      if (message.type == 'output') {
        this.appendOutput(message.data);
      } else if (message.type == 'exit') {
        this.appendOutput(this.userInput.innerText);
        this.closeOutput();
        this.inputLine.children[0].style.display = 'initial';
        this.inputLine.classList.remove('inline');
        this.userInput.innerText = '';
        this.userInput.focus();
        this.shellMode = true;
      }
    }

    appendOutput(text) {
      for (const line of text.split(/(\n)/)) {
        let output = this.root.children[this.root.children.length - 2];
        // Can happen if reset() is called while messages from the worker are still queued.
        if (!output) { return; }
        if (line == '\n') {
          output.classList.remove('inline');
          output = document.createElement('div');
          output.classList.add('inline');
          this.appendChild(output);
        } else {
          output.innerText += line;
          output.scrollIntoView();
        }
      }
    }

    closeOutput() {
      let output = this.root.children[this.root.children.length - 2];
      // Can happen if reset() is called while messages from the worker are still queued.
      if (!output) { return; }
      if (output.innerText.length > 0) {
        output.classList.remove('inline');
      } else {
        this.root.removeChild(output);
      }
    }

    appendChild(child) {
      while (this.root.children.length > MAX_LINES) {
        this.root.removeChild(this.root.firstChild);
      }
      this.root.insertBefore(child, this.inputLine);
      child.scrollIntoView();
    }

    reset() {
      this.root.replaceChildren(this.inputLine);
      this.inputLine.children[0].style.display = 'initial';
      this.inputLine.classList.remove('inline');
      this.userInput.innerText = '';
      this.shellMode = true;
    }
  }

  ToyPcEmulator.Terminal = Terminal;
})();