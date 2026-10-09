# Ground textures

Seamless tiles for the land, in the Roduro material family: colour is patch-quilted straight from the
building models' own base-colour maps (`Roduro_Home_5k` flow-grained wall stone, `Roduro_Great_5k`
plate stone, the Home roof's grass), so the land is made of the buildings' stone and the turf is their
roof grass. Relief is a periodic height field built per tile (grain from the quilted colour plus
cells / cracks / tufts), turned into the normal and roughness maps. Flat, unlit colour; the engine lights it.

`manifest.json` lists each tile: `name`, `metres` (real size of one repeat), `colour`, `normal`
(tangent space, **OpenGL convention, green = up**), `roughness` (grey, white = rough), and for the
stone tiles an `ao`. All 1024 x 1024, sRGB colour, non-colour data for the rest.

| tile | metres | what it is |
|---|---|---|
| turf_a, turf_b | 2 | the roof grass as turf; b has more of the roof's stone showing through |
| moss | 2 | damp moss cushions on the flow stone |
| heather | 2 | low heath over dark turf, purple-brown tufts |
| rock_a, rock_b | 3 | the wall stone as rock: a = flow-grained (Home), b = plate stone (Great house) |
| rock_wet | 3 | rock_a dark and wet, low roughness, slime in the hollows |
| scree | 2 | fist-to-head fragments of the stone |
| shingle | 2 | rounded pebbles of the stone |
| dirt | 2 | trodden earth with a few small stones |
| cobbles | 2 | fitted cobbles of the stone |
| stair | 1.5 | dressed step stone with fine tooling marks |

Checked by tiling 2 x 2 on a lit plane in Blender (colour + normal + roughness): no seams.
Made by `ground_tex.py` (in the asset tools, `_tools\_roduro_lod\kit\`); rerun it to change a tile.
