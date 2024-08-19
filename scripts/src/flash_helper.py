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

import os
import re
import stat
import boot_helper

# Check if stdin is a file. If not, we assume it is an interactive terminal.
stdin_from_file = stat.S_ISREG(os.fstat(0).st_mode)


def wait_ready(register):
	while boot_helper.run(f'w{register:08X},#').strip() != '0x00000001':
		pass


class Page:
	def __init__(self, index):
		self._index = index
		self._address = index * 256 + 0x80000
		self._values = []
		self._dirty = False
		print(f'Reading page {self._index}...', end='')
		for i in range(0, 64):
			address = self._address + 4 * i
			value = boot_helper.run(f'w{address:08X},#').strip()
			self._values.append(value[2:])
		print(' Done.')

	def set(self, index, value):
		if value != self._values[index]:
			self._values[index] = value
			self._dirty = True

	def flash(self):
		if not self._dirty:
			return
		print(f'Writing page {self._index}...', end='')
		for i in range(0, 64):
			address, value = self._address + 4 * i, self._values[i]
			boot_helper.run(f'W{address:08X},{value}#')
		if self._index < 1024:
			command = 0x5A000003 | (self._index << 8)
			boot_helper.run(f'W400E0A04,{command:08X}#')
			wait_ready(0x400E0A08)
		else:
			command = 0x5A000003 | ((self._index - 1024) << 8)
			boot_helper.run(f'W400E0C04,{command:08X}#')
			wait_ready(0x400E0C08)
		for i in range(0, 64):
			address = self._address + 4 * i
			value = boot_helper.run(f'w{address:08X},#').strip()[2:]
			if int(value, 16) != int(self._values[i], 16):
				exit(f'ERROR: page write failed at address {address:08X}')
		print(' Done.')


# Main loop (read commands from stdin, run them).
pages = {}
while True:
	try:
		commands = input().replace('#', '#\n').split()
	except (EOFError, KeyboardInterrupt):
		if stdin_from_file:
			print('Done.', end='')
		exit('')
	for command in commands:
		command = command.strip()
		if command == 'exit#':
			exit()
		if command == 'flash#':
			for _, page in sorted(pages.items()):
				page.flash()
			pages = {}
			print('>', end='')
			continue
		if command == 'reset#':
			boot_helper.run('W400E0A04,5A00010B#')  # Set boot from flash.
			wait_ready(0x400E0A08)
			reset = 'W400E1A00,A500000D#'
			boot_helper.serial_port.write(bytearray(reset.encode('ascii')))
			exit()
		match = re.match(r"W([0-9A-Fa-f]{1,8}),([0-9A-Fa-f]{1,8})#", command)
		if match:
			address, value = int(match.group(1), 16), match.group(2)
			if address >= 0x80000 and address < 0x100000:
				if address % 4 != 0:
					exit(f'ERROR: invalid address {address}.')
				page, word = (address - 0x80000) // 256, (address % 256) // 4
				if page not in pages:
					pages[page] = Page(page)
				pages[page].set(word, value)
				if not stdin_from_file:
					print('>', end='')
				continue
		boot_helper.run(command, verbose=not stdin_from_file)
