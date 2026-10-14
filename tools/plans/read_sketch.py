# Read one of Laz's town sketches (docs/town-plans.md).
#
# Claude measures each drawn building's four corners on a gridded copy of the
# drawing (make one with --grid) and writes them into B below, with the kind,
# the variant's depth:width ratio, the head count and the variant key; then
# this prints the plan entries (metres from the centre, facing, size), draws
# the reading over the sketch (plan_overlay.png: check it against the drawing
# before anything else), and rasters the trees from the ink (woods_preview.png,
# plan_read.json: the `rows` for the plan file's `woods`).
#
#   python tools/plans/read_sketch.py            # read (edit SRC, SCALE, ORIGIN, B, extra first)
#   python tools/plans/read_sketch.py --grid     # only write sketch_grid.png and the four quadrants
#
# Scale: a "room for 12" house is taken as 15 m along its long side.
import math, json
from PIL import Image, ImageDraw, ImageFont
import numpy as np
import sys, os
SRC=os.path.join(os.path.dirname(__file__),"..","..","docs","plans","last_wave_outpost_southwest.png")
im=Image.open(SRC).convert("RGB"); W,H=im.size
if "--grid" in sys.argv:
    g=im.copy(); d=ImageDraw.Draw(g)
    try: f=ImageFont.truetype("arial.ttf",26)
    except: f=None
    for x in range(0,W,100):
        d.line([(x,0),(x,H)],fill=(0,160,255) if x%500 else (0,0,255),width=1 if x%500 else 2); d.text((x+3,3),str(x),fill=(0,0,255),font=f); d.text((x+3,H//2),str(x),fill=(0,0,255),font=f)
    for y in range(0,H,100):
        d.line([(0,y),(W,y)],fill=(0,160,255) if y%500 else (0,0,255),width=1 if y%500 else 2); d.text((3,y+3),str(y),fill=(0,0,255),font=f); d.text((W//2,y+3),str(y),fill=(0,0,255),font=f)
    g.save("sketch_grid.png")
    for k,b in {"tl":(0,0,W//2+200,H//2+150),"tr":(W//2-200,0,W,H//2+150),"bl":(0,H//2-150,W//2+200,H),"br":(W//2-200,H//2-150,W,H)}.items():
        g.crop(b).save(f"sketch_{k}.png")
    print("gridded copy written"); sys.exit(0)
SCALE=15.0/260.0           # m per px: a "room for 12" house drawn 260 px long is 15 m
ORIGIN=(1650,1250)         # the hearth: open ground east of the chieftain's quarters
# label, corners (px), kind, ratio (depth, width) of the variant, size rule
B=[
 ("Shield maker's house",[(185,270),(415,150),(500,290),(255,410)],"QotiroBlock",(1.0,0.8),12,"qotiro_workyard"),
 ("Fletcher and bowyer's house",[(480,585),(690,380),(805,470),(640,690)],"QotiroBlock",(1.0,0.8),12,"qotiro_benchyard"),
 ("Temple and priestess quarters",[(1085,870),(1200,700),(1300,900),(1150,985)],"QotiroTemple",(1.0,1.0),12,"qotiro_temple"),
 ("Forge",[(1395,800),(1575,790),(1580,960),(1400,965)],"QotiroBlock",(1.0,0.8),0,"qotiro_workyard"),
 ("Chieftain's quarters",[(1150,1240),(1225,1110),(1360,1210),(1270,1330)],"QotiroBlock",(1.0,0.8),12,"qotiro_court"),
 ("Incense house",[(395,1165),(575,1200),(560,1285),(385,1245)],"QotiroHall",(0.96,0.8),0,"qotiro_island_hall"),
 ("Herbalist and alchemist's house",[(270,1430),(410,1375),(495,1580),(340,1640)],"QotiroBlock",(1.0,0.8),12,None),
 ("Leather crafter's house",[(1460,1710),(1710,1605),(1775,1750),(1540,1840)],"QotiroBlock",(1.0,0.8),12,"qotiro_benchyard"),
 ("Smoker's house",[(1940,1725),(2160,1605),(2230,1770),(2010,1885)],"QotiroBlock",(1.0,0.8),12,None),
]
def axis(c):
    # the long side's direction and the long/short lengths
    best=None
    for i in range(4):
        a,b=c[i],c[(i+1)%4]; L=math.hypot(b[0]-a[0],b[1]-a[1])
        if best is None or L>best[0]: best=(L,math.atan2(b[1]-a[1],b[0]-a[0]))
    short=min(math.hypot(c[(i+1)%4][0]-c[i][0],c[(i+1)%4][1]-c[i][1]) for i in range(4))
    return best[1],best[0],short
out=[]
for label,c,kind,ratio,room,var in B:
    cx=sum(p[0] for p in c)/4; cy=sum(p[1] for p in c)/4
    a,long,short=axis(c)
    size=long*SCALE/ratio[0]
    # the door is on the short end that looks toward the centre
    to=(ORIGIN[0]-cx,ORIGIN[1]-cy)
    if math.cos(a)*to[0]+math.sin(a)*to[1]<0: a+=math.pi
    deg=(math.degrees(a)+360)%360
    at=((cx-ORIGIN[0])*SCALE,(cy-ORIGIN[1])*SCALE)
    out.append(dict(label=label,kind=kind,variant=var,at=(round(at[0],1),round(at[1],1)),facing=round(deg),size=round(size,1),room=room,px=(cx,cy),a=a,ratio=ratio))
# overlay
ov=im.copy(); d=ImageDraw.Draw(ov)
try: f=ImageFont.truetype("arial.ttf",30)
except: f=None
def rect(cx,cy,a,depth,width,col):
    ca,sa=math.cos(a),math.sin(a)
    pts=[]
    for dx,dy in [(-0.5,-0.5),(0.5,-0.5),(0.5,0.5),(-0.5,0.5)]:
        x=dx*depth; y=dy*width
        pts.append((cx+x*ca-y*sa, cy+x*sa+y*ca))
    d.polygon(pts,outline=col,width=5)
    # the door end
    d.line([(cx,cy),(cx+ca*depth*0.5,cy+sa*depth*0.5)],fill=col,width=6)
for o in out:
    depth=o["size"]*o["ratio"][0]/SCALE; width=o["size"]*o["ratio"][1]/SCALE
    rect(o["px"][0],o["px"][1],o["a"],depth,width,(0,90,255))
    d.text((o["px"][0]-60,o["px"][1]-20),o["label"].split(" ")[0],fill=(0,90,255),font=f)
# places, pens, hearth
extra={"hearth":ORIGIN,"timber store":(670,160),"sheds":(715,1408),"gardens":(210,1550),"tanner":(1920,1900),"pens":(2360,820)}
for k,(x,y) in extra.items():
    r=6/SCALE*0.5 if k!="pens" else 160
    d.ellipse([x-r,y-r,x+r,y+r],outline=(0,170,0),width=5); d.text((x-40,y+r),k,fill=(0,170,0),font=f)
ov.save("plan_overlay.png")
# woods raster: black ink (not red, not white) in cells, with the drawn things masked out
arr=np.asarray(im).astype(int); r,g,b=arr[...,0],arr[...,1],arr[...,2]
ink=((r+g+b)<540)&~((r>150)&(g<110)&(b<110))   # dark, not red
mask=Image.new("L",(W,H),0); md=ImageDraw.Draw(mask)
for o in out:
    depth=o["size"]*o["ratio"][0]/SCALE*1.35; width=o["size"]*o["ratio"][1]/SCALE*1.5
    ca,sa=math.cos(o["a"]),math.sin(o["a"]); cx,cy=o["px"]
    pts=[(cx+dx*depth*ca-dy*width*sa, cy+dx*depth*sa+dy*width*ca) for dx,dy in [(-0.5,-0.5),(0.5,-0.5),(0.5,0.5),(-0.5,0.5)]]
    md.polygon(pts,fill=255)
for k,(x,y) in extra.items():
    rr={"pens":200,"gardens":150,"timber store":140,"sheds":130,"tanner":110,"hearth":0}[k]
    md.ellipse([x-rr,y-rr,x+rr,y+rr],fill=255)
# the red text's black? none. Also the title block top-right isn't ink. 
m=np.asarray(mask)>0
CELL=32
gw,gh=W//CELL,H//CELL
grid=np.zeros((gh,gw))
for j in range(gh):
    for i in range(gw):
        cell=ink[j*CELL:(j+1)*CELL,i*CELL:(i+1)*CELL]; mm=m[j*CELL:(j+1)*CELL,i*CELL:(i+1)*CELL]
        v=cell[~mm].mean() if (~mm).any() else 0.0
        grid[j,i]=v
# a hand-drawn circle stroke covers ~8-15% of its cell; smooth over neighbours and scale to 0..9
def box3(a):
    p=np.pad(a,1,mode="edge"); o=np.zeros_like(a)
    for dj in range(3):
        for di in range(3): o+=p[dj:dj+a.shape[0],di:di+a.shape[1]]
    return o/9.0
sm=box3(grid)
dens=np.clip(sm/0.10,0,1)
rows=["".join(str(int(round(v*9))) for v in row) for row in dens]
# preview
pv=im.copy().convert("RGBA"); layer=Image.new("RGBA",(W,H),(0,0,0,0)); ld=ImageDraw.Draw(layer)
for j in range(gh):
    for i in range(gw):
        v=dens[j,i]
        if v>0.05: ld.rectangle([i*CELL,j*CELL,(i+1)*CELL,(j+1)*CELL],fill=(0,140,0,int(40+150*v)))
pv=Image.alpha_composite(pv,layer); pv.convert("RGB").resize((W//2,H//2)).save("woods_preview.png")
ov.resize((W//2,H//2)).save("plan_overlay_half.png")
json.dump(dict(scale=SCALE,origin=ORIGIN,cell_m=CELL*SCALE,grid_w=gw,grid_h=gh,rows=rows,buildings=[{k:v for k,v in o.items() if k not in("px","a","ratio")} for o in out]),open("plan_read.json","w"),indent=1)
for o in out: print(o["label"],o["at"],"facing",o["facing"],"size",o["size"])
print("cell m",CELL*SCALE,"grid",gw,gh,"origin cell",ORIGIN[0]/CELL,ORIGIN[1]/CELL)
