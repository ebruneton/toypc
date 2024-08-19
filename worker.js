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
  const Mode = {
    Shell: 'Shell',
    BootHelper: 'BootHelper',
    FlashHelper: 'FlashHelper',
  };

  const readLines = async function* (reader) {
    let chunk = await reader.read();
    let startIndex = 0;
    while (chunk.value) {
      const newLineIndex = chunk.value.indexOf('\n', startIndex);
      if (newLineIndex == -1) {
        const remainder = chunk.value.substring(startIndex);
        chunk = await reader.read();
        if (chunk.value) {
          chunk.value = remainder + chunk.value;
          startIndex = 0;
        } else {
          yield remainder;
        }
      } else {
        yield chunk.value.substring(startIndex, newLineIndex);
        startIndex = newLineIndex + 1;
      }
    }
  }

  class Emulator {
    constructor(baseUrl) {
      this.baseUrl = baseUrl;
      this.moduleExports = undefined;
      this.arduino = 0;
      this.serial_input = '';
      this.mode = Mode.Shell;
      this.bootHelper = 0;
      this.flashHelper = 0;
      this.ready = false;
    }

    initialize() {
      const moduleUrl = new URL('webapp.wasm', this.baseUrl);
      const moduleImports = this.moduleImports();
      WebAssembly.instantiateStreaming(fetch(moduleUrl), moduleImports).then(module => {
        this.moduleExports = module.instance.exports;
        this.moduleExports.initialize();
        this.arduino = this.moduleExports.new_arduino(Math.floor(Math.random() * 4294967295));
        this.ready = true;
        self.onmessage = (event) => this.onMessage(event);
        self.postMessage({ type: 'initialized' });
      });
    }

    moduleImports() {
      return {
        panic: {
          log: (chars, len) => {
            const charArray = new Int8Array(this.moduleExports.memory.buffer, chars, len);
            const str = new TextDecoder().decode(charArray);
            self.postMessage({ type: 'panic', data: str });
            this.ready = false;
          }
        },
        serial: {
          send: (chars, len) => {
            const charArray = new Int8Array(this.moduleExports.memory.buffer, chars, len);
            const str = new TextDecoder().decode(charArray);
            self.postMessage({ type: 'output', data: str });
          },
          receive: () => {
            let c = -1;
            if (this.serial_input.length > 0) {
              c = this.serial_input.charCodeAt(0);
              this.serial_input = this.serial_input.substr(1);
            }
            return c;
          },
        },
        display: {
          set_on: on => { self.postMessage({ type: 'set_on', on }); },
          set_read_layer: layer => {
            self.postMessage({ type: 'set_read_layer', layer });
          },
          set_write_layer: layer => {
            self.postMessage({ type: 'set_write_layer', layer });
          },
          draw_char: (x, y, c, foreground, background) => {
            self.postMessage({ type: 'draw_char', x, y, c, foreground, background });
          },
          set_cursor: (x, y, enabled, blink_time) => {
            self.postMessage({ type: 'set_cursor', x, y, enabled, blink_time });
          },
          clear: (left, top, right, bottom, full_screen) => {
            self.postMessage({ type: 'clear', left, top, right, bottom, full_screen });
          },
          reset: () => { self.postMessage({ type: 'reset' }); },
          set_led: on => { self.postMessage({ type: 'led', on }); }
        },
        timer: {
          wait: micros => { setTimeout(() => this.run(), micros / 1000.0); },
        },
      };
    }

    onMessage(event) {
      if (!this.ready) { return; }
      const message = event.data;
      if (message.type == 'scancode') {
        this.processScancode(message.data);
      } else if (message.type == 'serial') {
        const userInput = message.data;
        if (this.mode == Mode.Shell) {
          if (this.processShellCommand(userInput)) {
            self.postMessage({ type: 'exit' });
          }
        } else if (this.mode == Mode.BootHelper) {
          this.processBootHelperInput(userInput);
        } else if (this.mode == Mode.FlashHelper) {
          this.processFlashHelperInput(userInput);
        }
      } else if (message.type == 'reset') {
        this.moduleExports.reset(this.arduino);
        this.run();
      } else if (message.type == 'load') {
        this.load(message.data);
      } else if (message.type == 'save') {
        this.save();
      }
    }

    run() {
      if (!this.ready) { return; }
      const delay = this.moduleExports.run(this.arduino);
      if (delay >= 0) {
        setTimeout(() => this.run(), delay);
      }
    }

    load(file) {
      if (this.ready) {
        if (this.mode == Mode.BootHelper) {
          this.moduleExports.delete_boot_helper(this.bootHelper);
        } else if (this.mode == Mode.FlashHelper) {
          this.moduleExports.delete_flash_helper(this.flashHelper);
        }
      }
      this.serial_input = '';
      this.mode = Mode.Shell;
      this.bootHelper = 0;
      this.flashHelper = 0;
      const reader = new FileReader();
      reader.addEventListener('load', () => this.processFlashHelperCommandsFile(reader.result));
      reader.readAsText(file);
    }

    save() {
      if (!this.ready) { return; }
      const NUM_FLASH_MEMORY_WORDS = 128 * 1024;
      const flashContentPtr = this.moduleExports.get_flash_content(this.arduino);
      const runFromFlash = this.moduleExports.run_from_flash(this.arduino);
      const flashContent = new Uint32Array(
        this.moduleExports.memory.buffer, flashContentPtr, NUM_FLASH_MEMORY_WORDS);
      self.postMessage({ type: 'on_save', flashContent, runFromFlash });
    }

    processScancode(scancode) {
      if (!this.ready) { return; }
      this.moduleExports.process_scancode(this.arduino, scancode);
    }

    processShellCommand(command) {
      if (!this.ready) { return; }
      const tokens = command.split(/\s+/);
      if (tokens.length == 0) {
        return true;
      }
      if (tokens[0] != 'python3') {
        self.postMessage({ type: 'output', data: `Can't find '${tokens[0]}'\n` });
        return true;
      }
      if (tokens.length == 1) {
        self.postMessage({
          type: 'output',
          data: "Usage: 'python3 boot_helper.py' or 'python3 flash_helper.py [< file]'"
        });
        return true;
      }
      if (tokens[1] == 'boot_helper.py') {
        this.processBootHelperCommand(tokens.slice(2));
      } else if (tokens[1] == 'flash_helper.py') {
        this.processFlashHelperCommand(tokens.slice(2));
      } else {
        self.postMessage({
          type: 'output',
          data: `python3: can't open file '${tokens[1]}'\n`
        });
        return true;
      }
      return false;
    }

    processBootHelperCommand() {
      this.bootHelper = this.moduleExports.new_boot_helper(this.arduino);
      if (this.bootHelper != 0) {
        this.mode = Mode.BootHelper;
      } else {
        self.postMessage({ type: 'exit' });
      }
    }

    processBootHelperInput(input) {
      this.serial_input = input;
      if (this.moduleExports.run_boot_helper(this.arduino, this.bootHelper)) {
        this.moduleExports.delete_boot_helper(this.bootHelper);
        this.mode = Mode.Shell;
        this.bootHelper = null;
        self.postMessage({ type: 'exit' });
      }
      this.run();
    }

    processFlashHelperCommand(tokens) {
      if (tokens.length == 0) {
        this.serial_input = '';
        this.flashHelper = this.moduleExports.new_flash_helper(this.arduino);
        if (this.flashHelper != 0) {
          this.mode = Mode.FlashHelper;
          return;
        }
      } else if (tokens.length == 2 && tokens[0] == '<') {
        this.processFlashHelperCommandsFilename(tokens[1]);
        return;
      } else {
        self.postMessage({
          type: 'output',
          data: "Usage: 'python3 flash_helper.py [< file]"
        });
      }
      self.postMessage({ type: 'exit' });
    }

    async processFlashHelperCommandsFilename(filename) {
      const url = new URL(filename, this.baseUrl);
      const response = await fetch(url);
      if (response.ok) {
        this.serial_input = 'ignored';
        this.flashHelper = this.moduleExports.new_flash_helper(this.arduino);
        if (this.flashHelper != 0) {
          const reader = response.body.pipeThrough(new TextDecoderStream()).getReader();
          for await (const line of readLines(reader)) {
            this.serial_input = line;
            // Can happen if 'load()' is called while this loop is running.
            if (this.flashHelper == 0) { return; }
            if (this.moduleExports.run_flash_helper(this.arduino, this.flashHelper)) {
              break;
            }
          }
          self.postMessage({ type: 'output', data: 'Done.\n' });
          this.moduleExports.delete_flash_helper(this.flashHelper);
        }
      } else {
        self.postMessage({
          type: 'output',
          data: `Can't find file '${filename}'\n`
        });
      }
      self.postMessage({ type: 'exit' });
      this.run();
    }

    processFlashHelperCommandsFile(content) {
      if (!this.ready) { return; }
      this.serial_input = 'ignored';
      this.flashHelper = this.moduleExports.new_flash_helper(this.arduino);
      if (this.flashHelper != 0) {
        for (const line of content.split(/(\n)/)) {
          this.serial_input = line;
          // Can happen if 'load()' is called while this loop is running.
          if (this.flashHelper == 0) { return; }
          if (this.moduleExports.run_flash_helper(this.arduino, this.flashHelper)) {
            break;
          }
        }
        self.postMessage({ type: 'output', data: 'Done.\n' });
        this.moduleExports.delete_flash_helper(this.flashHelper);
      }
      self.postMessage({ type: 'exit' });
      this.run();
    }

    processFlashHelperInput(input) {
      this.serial_input = input;
      if (this.moduleExports.run_flash_helper(this.arduino, this.flashHelper)) {
        this.moduleExports.delete_flash_helper(this.flashHelper);
        this.mode = Mode.Shell;
        this.flashHelper = null;
        self.postMessage({ type: 'exit' });
      }
      this.run();
    }
  }

  self.onmessage = (event) => new Emulator(event.data).initialize();
})();
