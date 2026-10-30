"""Remet un habitant en forme lisible : une ligne de pixels par ligne de texte, une image par bloc.

    python tools/habitants/ranger.py habitants/marin.json
"""
import json
import sys


def ranger(h):
    out = ['{']
    for cle in ('nom', 'description', 'valide', 'taille'):
        out.append(f'  {json.dumps(cle)}: {json.dumps(h[cle], ensure_ascii=False)},')
    out.append('  "palette": {')
    pal = list(h['palette'].items())
    out += [f'    {json.dumps(k)}: {json.dumps(v)}' + (',' if i < len(pal) - 1 else '') for i, (k, v) in enumerate(pal)]
    out.append('  },')
    out.append('  "etats": {')
    etats = list(h['etats'].items())
    for i, (nom, e) in enumerate(etats):
        out.append(f'    {json.dumps(nom)}: {{')
        out.append(f'      "every": {e["every"]},')
        out.append('      "images": [')
        for j, img in enumerate(e['images']):
            out.append('        [')
            out += [f'          {json.dumps(l)}' + (',' if k < len(img) - 1 else '') for k, l in enumerate(img)]
            out.append('        ]' + (',' if j < len(e['images']) - 1 else ''))
        out.append('      ]')
        out.append('    }' + (',' if i < len(etats) - 1 else ''))
    out.append('  }')
    out.append('}')
    return '\n'.join(out) + '\n'


if __name__ == '__main__':
    for p in sys.argv[1:]:
        h = json.load(open(p, encoding='utf-8'))
        open(p, 'w', encoding='utf-8', newline='\n').write(ranger(h))
