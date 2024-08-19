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

ToyPcEmulator = {};

(function () {
  const WIDTH = 800;
  const HEIGHT = 480;
  const MAX_CHARACTERS = 3000;

  const DRAW_CHARACTER_VERTEX_SHADER =
    `#version 300 es
   in vec2 uv;
   in uvec2 position;
   in uint character;
   out vec2 tex_uv;
   void main() {
     vec2 pos = (vec2(position) + uv * vec2(8.0, 16.0)) / vec2(${WIDTH}.0, ${HEIGHT}.0);
     pos = pos * 2.0 - vec2(1.0, 1.0);
     gl_Position = vec4(pos.x, -pos.y, 0.0, 1.0);
     float col = float(character % 32u);
     float row = float(character / 32u);
     tex_uv = (vec2(col, row) + uv) / vec2(32.0, 8.0);
   }`;

  const DRAW_CHARACTER_FRAGMENT_SHADER =
    `#version 300 es
   precision highp float;
   uniform sampler2D tex;
   uniform vec3 background;
   uniform vec3 foreground;
   in vec2 tex_uv;
   out vec4 frag_color;  
   void main() {
     vec3 color = mix(background, foreground, texture(tex, tex_uv).r);
     frag_color = vec4(color, 1.0);
   }`;

  const DRAW_LAYER_VERTEX_SHADER =
    `#version 300 es
   in vec2 uv;
   out vec2 tex_uv;
   void main() {
     gl_Position = vec4(uv * 2.0 - vec2(1.0, 1.0), 0.0, 1.0);
     tex_uv = uv;
   }`;

  const DRAW_LAYER_FRAGMENT_SHADER =
    `#version 300 es
   precision highp float;
   uniform vec3 foreground;
   uniform vec4 cursor_area;
   uniform sampler2D tex;
   uniform float on;
   in vec2 tex_uv;
   out vec4 frag_color;  
   void main() {
     vec3 color = texture(tex, tex_uv).rgb;
     vec2 cursor = step(cursor_area.xy, tex_uv) * step(tex_uv, cursor_area.zw);
     color = mix(color, foreground, cursor.x * cursor.y) * on;
     frag_color = vec4(color, 1.0);
   }`;

  const createVertexBuffer = function (gl) {
    const vertexBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, vertexBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([0, 0, 1, 0, 0, 1, 1, 1]), gl.STATIC_DRAW);
    return vertexBuffer;
  }

  const createTexture = function (gl) {
    const texture = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, texture);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB, WIDTH, HEIGHT, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    return texture;
  }

  const createFramebuffer = function (gl, texture) {
    const framebuffer = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, framebuffer);
    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture, 0);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
    return framebuffer;
  }

  const createShader = function (gl, type, source) {
    const shader = gl.createShader(type);
    gl.shaderSource(shader, source);
    gl.compileShader(shader);
    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
      console.log(gl.getShaderInfoLog(shader));
      gl.deleteShader(shader);
    }
    return shader;
  }

  const createProgram = function (gl, vertexShaderSource, fragmentShaderSource) {
    const program = gl.createProgram();
    gl.attachShader(program, createShader(gl, gl.VERTEX_SHADER, vertexShaderSource));
    gl.attachShader(program, createShader(gl, gl.FRAGMENT_SHADER, fragmentShaderSource));
    gl.linkProgram(program);
    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
      console.log(gl.getProgramInfoLog(program));
      gl.deleteProgram(program);
    }
    return program;
  }

  const createDrawCharacterProgram = function (gl) {
    const program = createProgram(gl, DRAW_CHARACTER_VERTEX_SHADER, DRAW_CHARACTER_FRAGMENT_SHADER);
    program.uv = gl.getAttribLocation(program, 'uv');
    program.position = gl.getAttribLocation(program, 'position');
    program.character = gl.getAttribLocation(program, 'character');
    program.tex = gl.getUniformLocation(program, 'tex');
    program.background = gl.getUniformLocation(program, 'background');
    program.foreground = gl.getUniformLocation(program, 'foreground');
    return program;
  }

  const createDrawLayerProgram = function (gl) {
    const program = createProgram(gl, DRAW_LAYER_VERTEX_SHADER, DRAW_LAYER_FRAGMENT_SHADER);
    program.uv = gl.getAttribLocation(program, 'uv');
    program.foreground = gl.getUniformLocation(program, 'foreground');
    program.cursorArea = gl.getUniformLocation(program, 'cursor_area');
    program.tex = gl.getUniformLocation(program, 'tex');
    program.on = gl.getUniformLocation(program, 'on');
    return program;
  }

  const createCharacterVertexArray = function (gl, program, vertexBuffer, positionsBuffer, charactersBuffer) {
    const vertexArray = gl.createVertexArray();
    gl.bindVertexArray(vertexArray);

    gl.bindBuffer(gl.ARRAY_BUFFER, vertexBuffer);
    gl.enableVertexAttribArray(program.uv);
    gl.vertexAttribPointer(program.uv, 2, gl.FLOAT, false, 0, 0);

    gl.bindBuffer(gl.ARRAY_BUFFER, positionsBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, MAX_CHARACTERS * 4, gl.DYNAMIC_DRAW);
    gl.enableVertexAttribArray(program.position);
    gl.vertexAttribIPointer(program.position, 2, gl.UNSIGNED_SHORT, false, 0, 0);
    gl.vertexAttribDivisor(program.position, 1);

    gl.bindBuffer(gl.ARRAY_BUFFER, charactersBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, MAX_CHARACTERS, gl.DYNAMIC_DRAW);
    gl.enableVertexAttribArray(program.character);
    gl.vertexAttribIPointer(program.character, 1, gl.UNSIGNED_BYTE, false, 0, 0);
    gl.vertexAttribDivisor(program.character, 1);
    return vertexArray;
  }

  const createDrawLayerVertexArray = function (gl, program, vertexBuffer) {
    const vertexArray = gl.createVertexArray();
    gl.bindVertexArray(vertexArray);
    gl.bindBuffer(gl.ARRAY_BUFFER, vertexBuffer);
    gl.enableVertexAttribArray(program.uv);
    gl.vertexAttribPointer(program.uv, 2, gl.FLOAT, false, 0, 0);
    return vertexArray;
  }

  const setColor = function (gl, uniform, color) {
    const red = (color >> 16) & 0xFF;
    const green = (color >> 8) & 0xFF;
    const blue = (color) & 0xFF;
    gl.uniform3f(uniform, red, green, blue);
  }

  class Screen {
    constructor(root) {
      const gl = root.getContext('webgl2');
      const vertexBuffer = createVertexBuffer(gl);
      const positionsBuffer = gl.createBuffer();
      const charactersBuffer = gl.createBuffer();

      this.root = root;
      this.gl = gl;
      this.positionsBuffer = positionsBuffer;
      this.positionsArray = new Uint16Array(MAX_CHARACTERS * 2);
      this.charactersBuffer = charactersBuffer;
      this.charactersArray = new Uint8Array(MAX_CHARACTERS);
      this.fontTexture = gl.createTexture();
      this.layerTextures = [createTexture(gl), createTexture(gl)];
      this.layerFrameBuffers = [
        createFramebuffer(gl, this.layerTextures[0]),
        createFramebuffer(gl, this.layerTextures[1])
      ];
      this.drawCharacterProgram = createDrawCharacterProgram(gl);
      this.drawLayerProgram = createDrawLayerProgram(gl);
      this.characterVertexArray = createCharacterVertexArray(gl, this.drawCharacterProgram,
        vertexBuffer, positionsBuffer, charactersBuffer);
      this.layerVertexArray = createDrawLayerVertexArray(gl, this.drawLayerProgram, vertexBuffer);

      this.messages = [{ type: 'reset' }];
      this.on = false;
      this.readLayer = 0;
      this.writeLayer = 0;
      this.foreground = 0;
      this.background = 0;
      this.cursor = { x: 0, y: 0 };
      this.cursorEnabled = false;
      this.cursorTime = 0;
      this.cursorBlinkTime = -1;
      this.cursorVisible = false;
    }

    initialize(callback) {
      const image = new Image();
      image.addEventListener('load', () => {
        const gl = this.gl;
        gl.bindTexture(gl.TEXTURE_2D, this.fontTexture);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
        gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB, gl.RGB, gl.UNSIGNED_BYTE, image);
        requestAnimationFrame(() => this.draw());
        callback();
      });
      image.src = 'font.png';
    }

    onMessage(message) {
      this.messages.push(message);
    }

    draw() {
      let cursorVisible = false;
      if (this.cursorBlinkTime >= 0) {
        this.cursorTime = (this.cursorTime + 1) % (2 * this.cursorBlinkTime);
        cursorVisible = this.cursorTime < this.cursorBlinkTime;
      }
      if (cursorVisible != this.cursorVisible || this.messages.length > 0) {
        this.cursorVisible = cursorVisible;
        this.processMessages();

        const gl = this.gl;
        gl.bindFramebuffer(gl.FRAMEBUFFER, null);
        gl.useProgram(this.drawLayerProgram);
        gl.activeTexture(gl.TEXTURE0);
        gl.bindTexture(gl.TEXTURE_2D, this.layerTextures[this.readLayer]);
        setColor(gl, this.drawLayerProgram.foreground, this.foreground);
        if (cursorVisible) {
          gl.uniform4f(this.drawLayerProgram.cursorArea,
            this.cursor.x / WIDTH,
            this.cursor.y / HEIGHT,
            (this.cursor.x + 8) / WIDTH,
            (this.cursor.y + 1) / HEIGHT);
        } else {
          gl.uniform4f(this.drawLayerProgram.cursorArea, 0.0, 0.0, 0.0, 0.0);
        }
        gl.uniform1i(this.drawLayerProgram.tex, 0);
        gl.uniform1f(this.drawLayerProgram.on, this.on);

        gl.bindVertexArray(this.layerVertexArray);
        gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
      }
      requestAnimationFrame(() => this.draw());
    }

    processMessages() {
      let numCharacters = 0;
      for (const message of this.messages) {
        switch (message.type) {
          case 'set_on': {
            this.on = message.on;
            break;
          }
          case 'set_read_layer': {
            this.readLayer = message.layer;
            break;
          }
          case 'set_write_layer': {
            this.drawCharacters(numCharacters);
            this.writeLayer = message.layer;
            numCharacters = 0;
            break;
          }
          case 'draw_char': {
            if (message.foreground != this.foreground
              || message.background != this.background
              || numCharacters == MAX_CHARACTERS) {
              this.drawCharacters(numCharacters);
              this.foreground = message.foreground;
              this.background = message.background;
              numCharacters = 0;
            }
            this.positionsArray[2 * numCharacters] = message.x;
            this.positionsArray[2 * numCharacters + 1] = message.y;
            this.charactersArray[numCharacters] = message.c;
            numCharacters += 1;
            break;
          }
          case 'set_cursor': {
            this.cursor = { x: message.x, y: 480 - (message.y + 16) };
            this.cursorEnabled = message.enabled;
            this.cursorBlinkTime = message.blink_time;
            break;
          }
          case 'clear': {
            if (message.full_screen) {
              this.clearLayer(0, null);
              this.clearLayer(1, null);
            } else {
              this.clearLayer(this.writeLayer,
                [message.left, message.top, message.right, message.bottom]);
            }
            numCharacters = 0;
            break;
          }
          case 'reset': {
            this.clearLayer(0, null);
            this.clearLayer(1, null);
            this.on = false;
            this.readLayer = 0;
            this.writeLayer = 0;
            this.cursor = { x: 0, y: 0 };
            this.cursorEnabled = false;
            this.cursorTime = 0;
            this.cursorBlinkTime = -1;
            this.cursorVisible = false;
            break;
          }
        }
      }
      this.messages.length = 0;
      this.drawCharacters(numCharacters);
    }

    drawCharacters(numCharacters) {
      if (numCharacters == 0) {
        return;
      }
      const gl = this.gl;
      gl.bindFramebuffer(gl.FRAMEBUFFER, this.layerFrameBuffers[this.writeLayer]);
      gl.useProgram(this.drawCharacterProgram);
      gl.activeTexture(gl.TEXTURE0);
      gl.bindTexture(gl.TEXTURE_2D, this.fontTexture);
      gl.uniform1i(this.drawCharacterProgram.tex, 0);
      setColor(gl, this.drawCharacterProgram.foreground, this.foreground);
      setColor(gl, this.drawCharacterProgram.background, this.background);

      gl.bindBuffer(gl.ARRAY_BUFFER, this.positionsBuffer);
      gl.bufferSubData(gl.ARRAY_BUFFER, 0, this.positionsArray.slice(0, numCharacters * 2));
      gl.bindBuffer(gl.ARRAY_BUFFER, this.charactersBuffer);
      gl.bufferSubData(gl.ARRAY_BUFFER, 0, this.charactersArray.slice(0, numCharacters));

      gl.bindVertexArray(this.characterVertexArray);
      gl.drawArraysInstanced(gl.TRIANGLE_STRIP, 0, 4, numCharacters);
    }

    clearLayer(layer, area) {
      const gl = this.gl;
      gl.bindFramebuffer(gl.FRAMEBUFFER, this.layerFrameBuffers[layer]);
      if (area) {
        gl.viewport(area[0], area[1], area[2] - area[0] + 1, area[3] - area[1] + 1);
        gl.clear(gl.COLOR_BUFFER_BIT);
        gl.viewport(0, 0, WIDTH, HEIGHT);
      } else {
        gl.clear(gl.COLOR_BUFFER_BIT);
      }
    }
  }

  ToyPcEmulator.Screen = Screen;
})();
