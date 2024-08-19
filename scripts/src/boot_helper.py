# This program is free software: you can redistribute it and/or modify it under
# the terms of the GNU General Public License as published by the Free Software
# Foundation, either version 3 of the License, or (at your option) any later
# version.
#
# This program is distributed in the hope that it will be useful, but WITHOUT
# ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
# FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License along with
# this program. If not, see <https://www.gnu.org/licenses/>.

import serial

# Initialize the serial port to communicate with the SAM-BA program,
# as described in section 20.4.1 of the SAM3X / SAM3A Datasheet.
try:
	serial_port = serial.Serial(port='/dev/ttyACM0', baudrate=115200,
								parity=serial.PARITY_NONE, stopbits=serial.STOPBITS_ONE,
								bytesize=serial.EIGHTBITS, timeout=5, write_timeout=1)
except:
	exit('ERROR: could not open serial port.')


def run(command, verbose=False):
	serial_port.write(bytearray(command.encode('ascii')))
	if command.endswith('#'):
		bytes = bytearray()
		while True:
			byte = serial_port.read()
			if len(byte) != 1:
				exit('ERROR: no response from device.')
			if chr(byte[0]) == '>':
				result = bytes.decode('ascii').lstrip()
				if verbose:
					print(f'{result}>', end='')
				return result
			bytes.append(byte[0])


# Flush the connection and switch the SAM-BA Monitor to ASCII mode.
serial_port.flush()
run('T#', verbose=True)

if __name__ == '__main__':
	# Main loop (read commands from stdin, run them).
	while True:
		try:
			commands = input().replace('#', '#\n').split()
		except (EOFError, KeyboardInterrupt):
			exit('')
		for command in commands:
			if command.strip() == 'exit#':
				exit()
			run(command, verbose=True)
