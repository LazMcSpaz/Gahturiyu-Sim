# Models

GLB files exported from Blender or Meshy, with standard materials (colour,
normal, and one combined occlusion/roughness/metallic image).

Each model is a set of files found by name, most detailed first:

    Roduro_Home_5k.glb    about 5,000 triangles (drawn up close)
    Roduro_Home_2k.glb    about 2,000 (from 70 m)
    Roduro_Home_500.glb   about 500 (from 220 m)

Only the `_5k` one is needed: any lower one that's missing is made when the
game starts, by simplifying the next one up. Make your own lower versions
when the automatic ones look wrong.

Models should stand on their base at height 0 and face +X. They're scaled
to the size the game gives each building, so any unit works.
