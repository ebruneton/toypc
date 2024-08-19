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

import json
import sys

scope_by_name = {}
scope_name_by_id = {}


def update_subcircuit(element, all_nodes):
    i = all_nodes[element['inputNodes'][0]].get('state')
    c = all_nodes[element['inputNodes'][1]].get('state')
    output = all_nodes[element['outputNodes'][0]]
    if 'NormallyClosed' in scope_name_by_id[element['id']]:
        if i != None and c != 1:
            output['state'] = i
    elif i != None and c == 1:
        output['state'] = i


def maybe_update_subcircuit_input(element, all_nodes):
    input = all_nodes[element['inputNodes'][0]]
    c = all_nodes[element['inputNodes'][1]].get('state')
    o = all_nodes[element['outputNodes'][0]].get('state')
    if 'NormallyClosed' in scope_name_by_id[element['id']]:
        if o != None and c != 1:
            input['state'] = o
    elif o != None and c == 1:
        input['state'] = o


def update_and_gate(nodes, all_nodes):
    i0 = all_nodes[nodes['inp'][0]].get('state')
    i1 = all_nodes[nodes['inp'][1]].get('state')
    if len(nodes['inp']) != 2:
        raise Exception(f'Unsupported and gate (>2 inputs)')
    output = all_nodes[nodes['output1']]
    if i0 != None or i1 != None:
        output['state'] = 1 if i0 == 1 and i1 == 1 else 0


def update_demultiplexer(nodes, all_nodes):
    i = all_nodes[nodes['input']].get('state')
    c = all_nodes[nodes['controlSignalInput']].get('state')
    output0 = all_nodes[nodes['output1'][0]]
    output1 = all_nodes[nodes['output1'][1]]
    if i != None and c != None:
        if c == 0:
            output0['state'] = i
            output1['state'] = 0
        else:
            output0['state'] = 0
            output1['state'] = i


def update_multiplexer(nodes, all_nodes):
    i0 = all_nodes[nodes['inp'][0]].get('state')
    i1 = all_nodes[nodes['inp'][1]].get('state')
    if len(nodes['inp']) != 2:
        raise Exception(f'Unsupported multiplexer (>2 inputs)')
    c = all_nodes[nodes['controlSignalInput']].get('state')
    output = all_nodes[nodes['output1']]
    if c != None:
        if c == 0 and i0 != None:
            output['state'] = i0
        elif i1 != None:
            output['state'] = i1


def update_nand_gate(nodes, all_nodes):
    i0 = all_nodes[nodes['inp'][0]].get('state')
    i1 = all_nodes[nodes['inp'][1]].get('state')
    if len(nodes['inp']) != 2:
        raise Exception(f'Unsupported nand gate (>2 inputs)')
    output = all_nodes[nodes['output1']]
    if i0 != None or i1 != None:
        output['state'] = 0 if i0 == 1 and i1 == 1 else 1


def update_nor_gate(nodes, all_nodes):
    i0 = all_nodes[nodes['inp'][0]].get('state')
    i1 = all_nodes[nodes['inp'][1]].get('state')
    if len(nodes['inp']) != 2:
        raise Exception(f'Unsupported nor gate (>2 inputs)')
    output = all_nodes[nodes['output1']]
    if i0 != None or i1 != None:
        output['state'] = 0 if i0 == 1 or i1 == 1 else 1


def update_not_gate(nodes, all_nodes):
    i = all_nodes[nodes['inp1']].get('state')
    output = all_nodes[nodes['output1']]
    if i != None:
        output['state'] = 1 if i == 0 else 0


def update_or_gate(nodes, all_nodes):
    i0 = all_nodes[nodes['inp'][0]].get('state')
    i1 = all_nodes[nodes['inp'][1]].get('state')
    if len(nodes['inp']) == 3:
        i2 = all_nodes[nodes['inp'][2]].get('state')
        output = all_nodes[nodes['output1']]
        if i0 != None or i1 != None or i2 != None:
            output['state'] = 1 if i0 == 1 or i1 == 1 or i2 == 1 else 0
    else:
        if len(nodes['inp']) != 2:
            raise Exception(f'Unsupported or gate (>3 inputs)')
        output = all_nodes[nodes['output1']]
        if i0 != None or i1 != None:
            output['state'] = 1 if i0 == 1 or i1 == 1 else 0


def update_sr_flip_flop(nodes, all_nodes):
    i = all_nodes[nodes['S']].get('state')
    output = all_nodes[nodes['qOutput']]
    output['state'] = 1 if i == 1 else 0


def update_tristate(nodes, all_nodes):
    i = all_nodes[nodes['inp1']].get('state')
    c = all_nodes[nodes['state']].get('state')
    output = all_nodes[nodes['output1']]
    if i != None and c != None:
        output['state'] = i if c == 1 else None


def update_xor_gate(nodes, all_nodes):
    i0 = all_nodes[nodes['inp'][0]].get('state')
    i1 = all_nodes[nodes['inp'][1]].get('state')
    if len(nodes['inp']) != 2:
        raise Exception(f'Unsupported xor gate (>2 inputs)')
    output = all_nodes[nodes['output1']]
    if i0 != None or i1 != None:
        output['state'] = 1 if (i0 == 1) != (i1 == 1) else 0


def update_element(element, all_nodes):
    type = element.get('objectType')
    if type is None:
        update_subcircuit(element, all_nodes)
        return
    nodes = element['customData']['nodes']
    match type:
        case 'AndGate':
            update_and_gate(nodes, all_nodes)
        case 'Demultiplexer':
            update_demultiplexer(nodes, all_nodes)
        case 'Multiplexer':
            update_multiplexer(nodes, all_nodes)
        case 'NandGate':
            update_nand_gate(nodes, all_nodes)
        case 'NorGate':
            update_nor_gate(nodes, all_nodes)
        case 'NotGate':
            update_not_gate(nodes, all_nodes)
        case 'OrGate':
            update_or_gate(nodes, all_nodes)
        case 'SRflipFlop':
            update_sr_flip_flop(nodes, all_nodes)
        case 'TriState':
            update_tristate(nodes, all_nodes)
        case 'XorGate':
            update_xor_gate(nodes, all_nodes)


INPUT_NODE = 0
OUTPUT_NODE = 1


def element_nodes(e):
    nodes = []
    for value in e['customData']['nodes'].values():
        if type(value) is list:
            for index in value:
                nodes.append(index)
        else:
            nodes.append(value)
    return nodes


def initialize_node_states(e, all_nodes, element_by_node_index):
    type = e.get('objectType')
    if type is None:
        name = scope_name_by_id[e['id']]
        for input in e['inputNodes']:
            element_by_node_index[input] = e
        for output in e['outputNodes']:
            element_by_node_index[output] = e
            if not 'Normally' in name:
                all_nodes[output]['state'] = 0
        return
    for node in element_nodes(e):
        element_by_node_index[node] = e
    nodes = e['customData']['nodes']
    match type:
        case 'Clock' | 'Ground':
            all_nodes[nodes['output1']]['state'] = 0
        case 'Button' | 'Input':
            state = e['customData']['values']['state']
            all_nodes[nodes['output1']]['state'] = state
        case 'Power':
            all_nodes[nodes['output1']]['state'] = 1
        case 'DflipFlop':
            # Hack to show D flip flops with custom initial states.
            if e['labelDirection'] == 'UP':
                all_nodes[nodes['qOutput']]['state'] = 1
            else:
                all_nodes[nodes['qOutput']]['state'] = 0
        case 'NorGate' | 'NotGate':
            # Hack to manually break cycles in flip flop circuits.
            if e['labelDirection'] == 'UP':
                all_nodes[nodes['output1']]['state'] = 1
            elif e['labelDirection'] == 'DOWN':
                all_nodes[nodes['output1']]['state'] = 0


def find_successors(index, nodes, successors, visited):
    visited[index] = True
    for successor_index in nodes[index]['connections']:
        if not visited[successor_index]:
            if nodes[successor_index]['type'] != OUTPUT_NODE:
                successors.add(successor_index)
            find_successors(successor_index, nodes, successors, visited)


def topological_sort(index, successors_by_node_index, sorted_nodes, node_status):
    if node_status[index] == 2:
        return
    if node_status[index] == 1:
        raise Exception('Cyclic graph!')
    node_status[index] = 1
    for successor_index in successors_by_node_index[index]:
        topological_sort(
            successor_index, successors_by_node_index, sorted_nodes, node_status)
    node_status[index] = 2
    sorted_nodes.append(index)


def compute_node_states(nodes, element_by_node_index):
    successors_by_node_index = [None] * len(nodes)
    for (index, node) in enumerate(nodes):
        visited = [False] * len(nodes)
        successors = set()
        if node['type'] == INPUT_NODE:
            element = element_by_node_index[index]
            if element:
                if element.get('objectType') is None:
                    name = scope_name_by_id[element['id']]
                    for output in element['outputNodes']:
                        if 'Normally' in name:
                            successors.add(output)
                else:
                    for n in element_nodes(element):
                        if nodes[n]['type'] == OUTPUT_NODE and nodes[n].get('state') is None:
                            successors.add(n)
        elif node['type'] == OUTPUT_NODE:
            find_successors(index, nodes, successors, visited)
        successors_by_node_index[index] = successors

    sorted_nodes = []
    node_status = [0] * len(nodes)
    for (index, node) in enumerate(nodes):
        if node_status[index] == 0:
            topological_sort(index, successors_by_node_index,
                             sorted_nodes, node_status)

    for index in reversed(sorted_nodes):
        node = nodes[index]
        if node['type'] != OUTPUT_NODE:
            continue
        element = element_by_node_index[index]
        if element and node.get('state') is None:
            update_element(element, nodes)
        state = node.get('state')
        if state != None:
            for successor_index in successors_by_node_index[index]:
                nodes[successor_index]['state'] = state

    for index in sorted_nodes:
        node = nodes[index]
        if node.get('state') is not None:
            continue
        for connection in node.get('connections'):
            state = nodes[connection].get('state')
            if state != None:
                node['state'] = state
                break
        if node['type'] == OUTPUT_NODE:
            element = element_by_node_index[index]
            if element and element.get('objectType') is None:
                maybe_update_subcircuit_input(element, nodes)


src = open(sys.argv[1], "r")
data = json.load(src)
src.close()

dst = open(sys.argv[1], "w")
json.dump(data, dst, indent=0)
dst.close()

tikz = open(sys.argv[2], "w")
tikz.write(
    '\\begin{tikzpicture}[x=0.15mm,y=-0.15mm,inner sep=0pt,outer sep=0pt,line width=0.45mm]\n')

bounding_box = [float('inf'), float('inf'), -float('inf'), -float('inf')]
xrange = [-float('inf'), float('inf')]
if len(sys.argv) > 3:
    xrange = [int(sys.argv[3]), int(sys.argv[4])]


UNKNOWN = 'red0'
OFF = 'violet2'
ON = 'green2'


def rotation(direction):
    match direction:
        case 'UP':
            return '[rotate=90] '
        case 'LEFT':
            return '[rotate=180] '
        case 'DOWN':
            return '[rotate=-90] '
        case _:
            return ''


def transform(node, x0, y0, direction):
    x, y = node['x'], node['y']
    match direction:
        case 'UP':
            (x, y) = (y, -x)
        case 'LEFT':
            (x, y) = (-x, y)
        case 'DOWN':
            (x, y) = (y, x)
    node['x'] = x0 + x
    node['y'] = y0 + y


def update_bounding_box(x, y):
    bounding_box[0] = min(bounding_box[0], x)
    bounding_box[1] = min(bounding_box[1], y)
    bounding_box[2] = max(bounding_box[2], x)
    bounding_box[3] = max(bounding_box[3], y)


def convert_subcircuit(e, all_nodes):
    x, y = e['x'], e['y']
    for index in e['inputNodes']:
        transform(all_nodes[index], x, y, 'RIGHT')
    for index in e['outputNodes']:
        transform(all_nodes[index], x, y, 'RIGHT')
    name = scope_name_by_id[e['id']]
    if not 'Normally' in name:
        layout = scope_by_name[name]['layout']
        dx = layout['width']
        dy = layout['height']
        tikz.write(f'\\path[draw=black] ({x}, {y}) rectangle +({dx}, {dy});\n')
        tikz.write(
            f'\\node[cv_font,anchor=center,black] at ({x+dx/2},{y+dy/2}){{{name}}};\n')
        return
    normally_open = 'NormallyOpen' in name
    input = all_nodes[e['inputNodes'][0]].get('state')
    active = all_nodes[e['inputNodes'][1]].get('state') == 1
    if active == normally_open:
        if input == None:
            input = UNKNOWN
        else:
            input = ON if input == 1 else OFF
    else:
        input = 'black'
    coil = 'yellow2' if active else 'black'
    if '-right' in name:
        tikz.write(
            f'\\begin{{scope}}[rotate around={{-90:({x+10},{y+10})}}]\n')
    tikz.write(f'\\path[fill={coil}] ({x}, {y+4}) rectangle +(4.5, 12);\n')
    tikz.write(f'\\path[draw=black] ({x}, {y}) rectangle +(30, 20);\n')
    if normally_open:
        if active:
            tikz.write(
                f'\\path[draw={input},line cap=round] ({x+12}, {y+3}) -- +(0, 14);\n')
            tikz.write(f'\\path[draw={input}] ({x+9}, {y+5}) -- +(0, 10);\n')
        else:
            tikz.write(
                f'\\path[draw={input},line cap=round] ({x+20}, {y+3}) -- +(0, 14);\n')
            tikz.write(f'\\path[draw={input}] ({x+17}, {y+5}) -- +(0, 10);\n')
    else:
        if active:
            tikz.write(
                f'\\path[draw={input},line cap=round] ({x+10}, {y+3}) -- +(0, 14);\n')
            tikz.write(f'\\path[draw={input}] ({x+7}, {y+5}) -- +(0, 10);\n')
        else:
            tikz.write(
                f'\\path[draw={input},line cap=round] ({x+18}, {y+3}) -- +(0, 14);\n')
            tikz.write(f'\\path[draw={input}] ({x+15}, {y+5}) -- +(0, 10);\n')
    if '-right' in name:
        tikz.write('\\end{scope}\n')


def convert_circuit_element(e, pic, all_nodes):
    x, y = e['x'], e['y']
    direction = e['direction']
    nodes = e['customData']['nodes']
    for key, value in nodes.items():
        match key:
            case ('clockInp' | 'controlSignalInput' | 'dInp' | 'inp' | 'inp1' |
                  'input' | 'input1' | 'output1' | 'qOutput' | 'R' | 'S' | 'state'):
                if type(value) is list:
                    for index in value:
                        transform(all_nodes[index], x, y, direction)
                else:
                    transform(all_nodes[value], x, y, direction)
            case _:
                all_nodes[value].clear()
    if x < xrange[0] or x > xrange[1]:
        return
    if pic == 'DigitalLed' and all_nodes[nodes['inp1']].get('state') == 1:
        tikz.write(
            f'\\pic at ({x},{y}) {rotation(direction)}{{DigitalLedOn}};\n')
    elif pic == 'Output' and all_nodes[nodes['inp1']].get('state') is None:
        tikz.write(f'\\pic at ({x},{y}) {rotation(direction)}{{OutputX}};\n')
    else:
        tikz.write(f'\\pic at ({x},{y}) {rotation(direction)}{{{pic}}};\n')
    if e['objectType'] == 'Input':
        state = e['customData']['values']['state']
        tikz.write(f'\\node[cv_font,anchor=center] at ({x},{y}){{{state}}};\n')
    elif e['objectType'] == 'Output':
        state = all_nodes[nodes['inp1']].get('state')
        if state == None:
            state = 'X'
        tikz.write(f'\\node[cv_font,anchor=center] at ({x},{y}){{{state}}};\n')
    elif e['objectType'] == 'DflipFlop' or e['objectType'] == 'SRflipFlop':
        state = all_nodes[nodes['qOutput']].get('state')
        tikz.write(f'\\node[cv_font,anchor=center] at ({x},{y}){{{state}}};\n')


def convert_node(node):
    x, y = node['x'], node['y']
    update_bounding_box(x, y)
    if x < xrange[0] or x > xrange[1]:
        return
    color = UNKNOWN
    state = node.get('state')
    if state == 0:
        color = OFF
    elif state == 1:
        color = ON
    if node['type'] == 2:
        tikz.write(f'\\path[fill={color}] ({x},{y}) circle[radius=3];\n')
    else:
        tikz.write(f'\\path[fill={OFF}] ({x},{y}) circle[radius=3];\n')


def convert_connections(index, node, nodes):
    x, y = node['x'], node['y']
    color = UNKNOWN
    state = node.get('state')
    if state == 0:
        color = OFF
    elif state == 1:
        color = ON
    for connection in node['connections']:
        if index < connection:
            other = nodes[connection]
            x0, y0 = x, y
            x1, y1 = other['x'], other['y']
            if x0 > x1:
                (x0, y0), (x1, y1) = (x1, y1), (x0, y0)
            if x1 < xrange[0] or x0 > xrange[1]:
                continue
            x0 = max(xrange[0], min(xrange[1], x0))
            x1 = max(xrange[0], min(xrange[1], x1))
            tikz.write(f'\\path[draw={color}] ({x0},{y}) -- ({x1},{y1});\n')


def convert_annotation(element):
    x, y = element['x'], element['y']
    if element['objectType'] == 'Rectangle':
        parameters = element['customData']['constructorParamaters']
        dx, dy = 10 * parameters[1], 10 * parameters[0]
        update_bounding_box(x, y)
        update_bounding_box(x + dx, y + dy)
        if x >= xrange[0] and x <= xrange[1]:
            tikz.write('\\path[draw=black,dash pattern=on 0.6mm off 0.9mm,'
                       f'line width=0.3mm,line cap=round] ({x},{y}) rectangle +({dx},{dy});\n')
        return
    if x < xrange[0] or x > xrange[1]:
        return
    if element['objectType'] in {'Input', 'Output'}:
        label = element['label']
        if not label:
            return
        anchor = ''
        match element['labelDirection']:
            case 'LEFT':
                anchor = 'east'
                x = x - 20
            case 'RIGHT':
                anchor = 'west'
                x = x + 20
            case 'UP':
                anchor = 'base'
                y = y - 20
            case 'DOWN':
                anchor = 'base'
                y = y + 30
        tikz.write(
            f'\\node[cv_font,anchor={anchor},baseline,color=black] at ({x},{y}) {{{label}}};\n')
    else:
        label = element['label']
        tikz.write(
            f'\\node[cv_font,anchor=base west,baseline,color=black] at ({x},{y}) {{{label}}};\n')


def main():
    for scope in data['scopes']:
        id = str(scope['id'])
        name = scope['name']
        scope_by_name[name] = scope
        scope_name_by_id[id] = name
    scope = scope_by_name['Main']
    nodes = scope['allNodes']
    supported = {'AndGate', 'Button', 'Clock', 'Demultiplexer', 'DflipFlop', 'DigitalLed',
                 'Ground', 'Input', 'Multiplexer', 'NandGate', 'NorGate', 'NotGate', 'OrGate',
                 'Output', 'Power', 'SRflipFlop', 'SubCircuit', 'TriState', 'XorGate'}
    ignored = {'layout', 'verilogMetadata', 'allNodes', 'id', 'name', 'Text',
               'Rectangle', 'restrictedCircuitElementsUsed', 'nodes'}
    element_by_node_index = [None] * len(nodes)
    for key, value in scope.items():
        if key in supported:
            for element in value:
                initialize_node_states(element, nodes, element_by_node_index)
    compute_node_states(nodes, element_by_node_index)
    for key, value in scope.items():
        if key in supported:
            for element in value:
                if key == 'SubCircuit':
                    convert_subcircuit(element, nodes)
                else:
                    convert_circuit_element(element, key, nodes)
        elif key not in ignored:
            raise Exception(f'Unsupported element {key}')
    for (index, node) in enumerate(nodes):
        if node:
            convert_connections(index, node, nodes)
    for node in nodes:
        if node:
            convert_node(node)
    for key, value in scope.items():
        if key in {'Input', 'Output', 'Rectangle', 'Text'}:
            for element in value:
                convert_annotation(element)
    if xrange[0] != -float('inf'):
        bounding_box[0] = max(bounding_box[0], xrange[0])
        bounding_box[2] = min(bounding_box[2], xrange[1])
        tikz.write(f'\\useasboundingbox ({bounding_box[0]},{bounding_box[1]}) '
                   f'rectangle ({bounding_box[2]},{bounding_box[3]});\n')
    tikz.write('\\end{tikzpicture}\n')
    tikz.close()


main()
