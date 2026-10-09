# Models

GLB files exported from Blender or Meshy, with standard materials (colour,
normal, and one combined occlusion/roughness/metallic image), 1024² textures.

## Roduro buildings

Four whole buildings, found by name, most detailed first:

    Roduro_Home_5k.glb    the cottage, about 5,000 triangles (drawn up close)
    Roduro_Home_2k.glb    about 2,000 (from 70 m)
    Roduro_Home_500.glb   about 500 (from 220 m)

and likewise `Roduro_Drum_*`, `Roduro_Vault_*` and `Roduro_Great_*` (the great
house, used for the biggest homes). Only the `_5k` one is needed: any lower
one that's missing is made when the game starts, by simplifying the next one
up. Make your own lower versions when the automatic ones look wrong.

Models stand on their base at height 0, face +X, and are in metres (they're
scaled to the size the game gives each building).

## Horaro kit (`horaro/`)

Pieces cut from Laz's two Meshy buildings: `<id>_lod0.glb` (close) and
`<id>.glb` (further off), with their origin at their anchor (the water line
for pieces that stand in the sea; stone pieces carry a foundation down to
-12 m). `horaro_kit.json` lists the pieces and the assemblies (which pieces,
where, turned how); the game reads it and builds each stilt home from one of
the home assemblies, stood on the water line. The file is in Blender's frame
(z up); the game turns it.

Blender's exporter leaves empty scenes in these GLBs, which Bevy's loader
rejects; they were stripped (keep only scenes that list nodes).
