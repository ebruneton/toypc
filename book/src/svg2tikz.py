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

import math
import re
import sys

from lxml import etree
from types import SimpleNamespace

consts = SimpleNamespace()
consts.SVG_NAMESPACE = '{http://www.w3.org/2000/svg}'
consts.CIRCLE = '{http://www.w3.org/2000/svg}circle'
consts.DEFS = '{http://www.w3.org/2000/svg}defs'
consts.ELLIPSE = '{http://www.w3.org/2000/svg}ellipse'
consts.GROUP = '{http://www.w3.org/2000/svg}g'
consts.MARKER = '{http://www.w3.org/2000/svg}marker'
consts.PATH = '{http://www.w3.org/2000/svg}path'
consts.RECT = '{http://www.w3.org/2000/svg}rect'
consts.TEXT = '{http://www.w3.org/2000/svg}text'


def check_round(value, base, allow_tenth=False):
    rounded_value = round(value / base) * base
    if not abs(rounded_value - value) < 0.05 * base:
        base = base / 2
        rounded_value = round(value / base) * base
        if not abs(rounded_value - value) < 0.05 * base:
            base = base / 5
            rounded_value = round(value / base) * base
            if not allow_tenth or not abs(rounded_value - value) < 0.001 * base:
                raise NotImplementedError(value)
    return rounded_value


def format(value):
    if abs(value - int(value)) < 1e-6:
        return f'{int(value)}'
    return f'{value:.1f}'


def simplify_path_data(path):
    tokens = path.split(' ')
    new_tokens = []
    command = None
    for token in tokens:
        if len(token) == 1 and token[0].isalpha():
            command = token
            new_tokens.append(command)
        else:
            match command:
                case 'M' | 'm' | 'L' | 'l' | 'C' | 'c':
                    coords = token.split(',')
                    x = check_round(float(coords[0]), 1, allow_tenth=True)
                    y = check_round(float(coords[1]), 1, allow_tenth=True)
                    new_tokens.append(f'{format(x)},{format(y)}')
                case 'H' | 'h' | 'V' | 'v':
                    x = check_round(float(token), 1, allow_tenth=True)
                    new_tokens.append(f'{format(x)}')
                case _:
                    raise NotImplementedError(path)
    return ' '.join(new_tokens)


def simplify_attributes(element, valid_keys):
    attributes = element.attrib
    for key, value in element.items():
        if not key in valid_keys:
            if not key in {'id', 'style'} and not ':' in key:
                raise NotImplementedError(key)
            attributes.pop(key)
        elif key == 'd':
            element.set(key, simplify_path_data(value))
        elif key == 'style':
            for style_element in value.split(';'):
                name_value = style_element.split(':')
                name = name_value[0]
                value = name_value[1]
                match name:
                    case 'fill':
                        if value != '#000000':
                            element.set(name, value)
                    case 'fill-opacity':
                        if value != '1':
                            element.set(name, value)
                    case 'font-family':
                        if value != "'Fira Sans'":
                            raise NotImplementedError(name, value)
                    case 'font-stretch' | 'font-variant' | 'font-weight' | 'font-style':
                        if value != 'normal':
                            raise NotImplementedError(name, value)
                    case 'font-size':
                        value = float(value.removesuffix('px'))
                        if value != 3.9:
                            element.set(name, f'{value}px')
                    case 'marker-start':
                        element.set(name, 'url(#arrow)')
                    case 'marker-end':
                        element.set(name, 'url(#arrow)')
                    case 'stroke':
                        if value != 'none':
                            element.set(name, f'{value}')
                    case 'stroke-dasharray':
                        if value != 'none':
                            value = ' '.join(map(
                                lambda s: f'{check_round(float(s), 0.2):.1f}',
                                value.split(',')))
                            element.set(name, f'{value}')
                    case 'stroke-dashoffset':
                        continue
                    case 'stroke-linecap':
                        if value != 'butt':
                            element.set(name, f'{value}')
                    case 'stroke-linejoin':
                        if value != 'miter':
                            element.set(name, f'{value}')
                    case 'stroke-opacity':
                        if value != '1':
                            raise NotImplementedError(name, value)
                    case 'stroke-width':
                        value = check_round(float(value), 0.2)
                        if value != 0.2:
                            element.set(name, f'{value:.1f}')
                    case 'text-align':
                        continue
                    case 'text-anchor':
                        if value != 'start':
                            element.set(name, value)
                    case 'stop-color':
                        continue
                    case _:
                        if not name.startswith('-'):
                            raise NotImplementedError(name, value)
            attributes.pop(key)
        elif (key == 'fill' and value == '#000000') or (
                key == 'fill-opacity' and value == '1') or (
                key == 'stroke' and value == 'none') or (
                key == 'stroke-opacity' and value == '1') or (
                key == 'text-anchor' and value == 'start'):
            attributes.pop(key)
        elif key in {'x', 'y', 'width', 'height', 'cx', 'cy', 'rx', 'ry', 'r'}:
            if 'transform' in element.keys() and (
                    element.get('transform') == 'rotate(-45)' or
                    element.get('transform') == 'rotate(45)'):
                continue
            if value.endswith('mm'):
                element.set(
                    key, f'{check_round(float(value.removesuffix("mm")), 1)}mm')
            else:
                element.set(key, f'{check_round(float(value), 1)}')


def simplify_marker_path(element):
    simplify_attributes(element, {'d', 'style', 'fill', 'stroke', 'transform'})


def simplify_circle(element):
    simplify_attributes(element, {'cx', 'cy', 'r', 'style', 'fill',
                                  'fill-opacity', 'stroke', 'stroke-dasharray', 'stroke-width'})


def simplify_path(element):
    simplify_attributes(element, {'d', 'style', 'fill', 'fill-opacity',
                                  'marker-start', 'marker-end', 'stroke', 'stroke-dasharray',
                                  'stroke-linecap', 'stroke-linejoin', 'stroke-width'})


def simplify_rect(element):
    simplify_attributes(element, {'d', 'style', 'fill', 'fill-opacity', 'stroke',
                                  'stroke-dasharray', 'stroke-width', 'x', 'y', 'width', 'height'})


def simplify_text(element):
    if len(element) == 1:
        text = element[0].text
        if text:
            style = element.get('style').replace('fill:none;', '')
            element.set('style', style + ';' + element[0].get('style'))
            element.remove(element[0])
            element.text = text
        else:
            element.getparent().remove(element)
            return
    if len(element) != 0:
        raise NotImplementedError
    simplify_attributes(element, {'d', 'style', 'fill', 'font-family', 'font-size',
                                  'text-anchor', 'transform', 'x', 'y'})


def simplify_group(element):
    simplify_attributes(element, {'font-family', 'font-size', 'stroke-width'})
    element.set('font-family', "'Fira Sans'")
    element.set('font-size', '3.9px')
    element.set('stroke-width', '0.2')
    for child in list(element):
        match child.tag:
            case consts.CIRCLE:
                simplify_circle(child)
            case consts.PATH:
                simplify_path(child)
            case consts.RECT:
                simplify_rect(child)
            case consts.TEXT:
                simplify_text(child)
            case _:
                if child.tag.startswith(consts.SVG_NAMESPACE):
                    raise NotImplementedError(child.tag)
                element.remove(child)


def simplify_marker(element):
    element.set('id', 'arrow')
    attributes = element.attrib
    for key, value in element.items():
        if not key in {'id', 'orient'}:
            attributes.pop(key)
    for child in list(element):
        match child.tag:
            case consts.PATH:
                simplify_marker_path(child)
            case _:
                element.remove(child)


def simplify_defs(element):
    simplify_attributes(element, {})
    marker_found = False
    for child in list(element):
        match child.tag:
            case consts.MARKER:
                if marker_found:
                    element.remove(child)
                else:
                    simplify_marker(child)
                    marker_found = True
            case _:
                raise NotImplementedError(child.tag)


def simplify_svg(element):
    simplify_attributes(element, {'width', 'height', 'version', 'viewBox'})
    for child in list(element):
        match child.tag:
            case consts.DEFS:
                simplify_defs(child)
            case consts.GROUP:
                simplify_group(child)
            case _:
                if child.tag.startswith(consts.SVG_NAMESPACE):
                    raise NotImplementedError(child.tag)
                element.remove(child)


colors = {
    '#000000': 'black',
    '#9a9a9a': 'gray0',
    '#cccccc': 'gray1',
    '#ebebeb': 'gray2',
    '#ffffff': 'white',
    '#bf3959': 'red0',
    '#ef476f': 'red1',
    '#f26c8c': 'red2',
    '#cca752': 'yellow0',
    '#ffd166': 'yellow1',
    '#ffda85': 'yellow2',
    '#048966': 'green0',
    '#05ab80': 'green1',
    '#06d6a0': 'green2',
    '#0e6e8e': 'blue0',
    '#118ab2': 'blue1',
    '#41a1c1': 'blue2',
    '#062f3d': 'violet0',
    '#073b4c': 'violet1',
    '#396270': 'violet2',
}


def convert_color(color):
    return colors[color]


def convert_style(element, remove_fill=False):
    styles = []
    angle = 0
    if transform := element.get('transform'):
        if transform == 'scale(-1)':
            angle = 180
            styles.append(f'rotate=180')
        else:
            angle = -int(re.search('rotate\((-?\d*)\)', transform).group(1))
            styles.append(f'rotate={angle}')
    if stroke := element.get('stroke'):
        styles.append(f'draw={convert_color(stroke)}')
    if dash := element.get('stroke-dasharray'):
        pattern = ''
        for index, value in enumerate(dash.replace(',', '').split(' ')):
            pattern += ' on' if index % 2 == 0 else ' off'
            pattern += f' {format(float(value))}mm'
        styles.append(f'dash pattern={pattern}')
    if fill := element.get('fill'):
        if remove_fill:
            element.attrib.pop('fill')
        elif fill != 'none':
            if element.tag == consts.TEXT:
                styles.append(f'text={convert_color(fill)}')
            else:
                styles.append(f'fill={convert_color(fill)}')
    elif element.tag == consts.RECT or element.tag == consts.CIRCLE:
        styles.append('fill=black')
    if fill_opacity := element.get('fill-opacity'):
        styles.append(f'fill opacity={fill_opacity}')
    if stroke_width := element.get('stroke-width'):
        width = float(stroke_width)
        styles.append(f'line width={format(width)}mm')
    if stroke_linecap := element.get('stroke-linecap'):
        if stroke_linecap == 'round':
            styles.append('line cap=round')
        elif stroke_linecap == 'square':
            styles.append('line cap=rect')
    if stroke_linejoin := element.get('stroke-linejoin'):
        if stroke_linejoin == 'bevel' or stroke_linejoin == 'round':
            styles.append(f'line join={stroke_linejoin}')
    if anchor := element.get('text-anchor'):
        if anchor == 'middle':
            styles.append('anchor=base')
        elif anchor == 'end':
            styles.append('anchor=base east')
    elif element.tag == consts.TEXT:
        styles.append('anchor=base west')
    if font_size := element.get('font-size'):
        font_size = float(font_size.removesuffix('px'))
        if font_size >= 8.0:
            styles.append('font=\\Huge')
        elif font_size <= 2.8:
            styles.append('font=\\tiny')
        elif font_size <= 3.5:
            styles.append('font=\\scriptsize')
    if element.get('marker-start') or element.get('marker-end'):
        arrows = ''
        if element.get('marker-start'):
            arrows = 'angle 60'
        if element.get('marker-end'):
            arrows += '-angle 60'
        else:
            arrows += '-'
        styles.append(arrows)
    return (','.join(styles), angle)


def convert_path(path):
    result = ''
    size = 0
    tokens = path.split(' ')
    command = None
    (x, y) = (0, 0)
    control = 0
    for token in tokens:
        if len(token) == 1 and token[0].isalpha():
            command = token
            control = 0
            if command == 'Z' or command == 'z':
                result += ' -- cycle'
            continue
        match command:
            case 'M' | 'm' | 'L' | 'l' | 'C' | 'c':
                coords = token.split(',')
                dx, dy = float(coords[0]), float(coords[1])
                if command.isupper():
                    x, y = dx, dy
                elif command != 'c':
                    x += dx
                    y += dy
            case 'H' | 'h':
                dx = float(token)
                if command.isupper():
                    x = dx
                else:
                    x += dx
            case 'V' | 'v':
                dy = float(token)
                if command.isupper():
                    y = dy
                else:
                    y += dy
            case _:
                raise NotImplementedError(path)
        if command == 'C':
            match control:
                case 0:
                    result += f' .. controls ({format(x)},{format(y)})'
                case 1:
                    result += f' and ({format(x)},{format(y)})'
                case _:
                    result += f' .. ({format(x)},{format(y)})'
            control = (control + 1) % 3
            size += 1
        elif command == 'c':
            match control:
                case 0:
                    result += f' .. controls ({format(x+dx)},{format(y+dy)})'
                case 1:
                    result += f' and ({format(x+dx)},{format(y+dy)})'
                case _:
                    result += f' .. ({format(x+dx)},{format(y+dy)})'
            control = (control + 1) % 3
            size += 1
            if control == 0:
                x, y = x + dx, y + dy
        else:
            if command == 'M':
                command = 'L'
            elif command == 'm':
                command = 'l'
            else:
                result += ' -- '
            result += f'({format(x)},{format(y)})'
            size += 1
    return (result, size)


def convert_svg(element):
    tikz = ''
    for child in list(element):
        match child.tag:
            case consts.GROUP:
                tikz += convert_svg(child)
            case consts.CIRCLE:
                x = child.get('cx')
                y = child.get('cy')
                r = child.get('r')
                style, angle = convert_style(child)
                tikz += f'\\path[{style}] ({x},{y}) circle[radius={r}];\n'
            case consts.PATH:
                path = child.get('d')
                coords, size = convert_path(path)
                style, angle = convert_style(child, size == 2)
                tikz += f'\\path[{style}] {coords};\n'
            case consts.RECT:
                x = child.get('x')
                y = child.get('y')
                width = child.get('width')
                height = child.get('height')
                style, angle = convert_style(child)
                tikz += f'\\path[{style}] ({x},{y}) rectangle +({width},{height});\n'
            case consts.TEXT:
                x = float(child.get('x'))
                y = float(child.get('y'))
                style, angle = convert_style(child)
                angle = math.radians(angle)
                rx = x * math.cos(angle) + y * math.sin(angle)
                ry = x * math.sin(angle) - y * math.cos(angle)
                tikz += f'\\node[{style}] at ({format(rx)},{format(-ry)}){{{child.text}}};\n'
    return tikz


src = open(sys.argv[1], "rb")
tree = etree.parse(src, parser=etree.XMLParser(remove_comments=True))

simplify_svg(tree.getroot())
tikz = convert_svg(tree.getroot())
etree.cleanup_namespaces(tree)

dst = open(sys.argv[1], "w")
dst.write('<!--\n')
dst.write(
    'This work is licensed under the Creative Commons Attribution NonCommercial\n')
dst.write(
    'ShareAlike 4.0 International License. To view a copy of the license, visit\n')
dst.write('https://creativecommons.org/licenses/by-nc-sa/4.0/\n')
dst.write('-->\n')
dst.write(etree.tostring(tree, pretty_print=True).decode())
dst.close()

dst = open(sys.argv[2], "w")
dst.write(
    '% This work is licensed under the Creative Commons Attribution NonCommercial\n')
dst.write(
    '% ShareAlike 4.0 International License. To view a copy of the license, visit\n')
dst.write('% https://creativecommons.org/licenses/by-nc-sa/4.0/\n')
dst.write(
    '\\begin{tikzpicture}[x=0.8mm,y=-0.8mm,inner sep=0pt,outer sep=0pt,line width=0.2mm]\n')
dst.write(tikz)
dst.write('\\end{tikzpicture}\n')
dst.close()
