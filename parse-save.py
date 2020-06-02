#!/usr/bin/env python3

import zlib
import sys
from typing import Dict, Tuple, List
import codecs
import py2jdbc.mutf8
from PIL import Image

# Header (4 bytes)
# Version (4 bytes)
# Regions seem to not be marked in the file
# Metadata region (TODO: How is region start signified?)
# - Length of region (int)
# - Number of metadata Map entries (16 bit signed)
#   * For each entry:
#     - Read map key as modified UTF-8 String (length in bytes prepended as a short)
#     - Read value as modified UTF-8 String (length in bytes prepended as a short)
# - Content region
#   * Content header
#     - Number of map entries (8 bit)
#     - And other data, see
#
#    public void readContentHeader(DataInput stream) throws IOException{
#        byte mapped = stream.readByte();
#
#        MappableContent[][] map = new MappableContent[ContentType.values().length][0];
#
#        for(int i = 0; i < mapped; i++){
#            ContentType type = ContentType.values()[stream.readByte()];
#            short total = stream.readShort();
#            map[type.ordinal()] = new MappableContent[total];

#            for(int j = 0; j < total; j++){
#                String name = stream.readUTF();
#                map[type.ordinal()][j] = content.getByName(type, fallback.get(name, name));
#            }
#        }

#        content.setTemporaryMapper(map);
#    }
# - Map region
#   * width (unsigned short) 16 bit
#   * height (unsigned short) 16 bit
#
#   * For width * height times: read ground
#     - Floor ID: short (16 bit)
#     - Ore ID: short (16 bit)
#     - Consecutives: unsigned byte (says how many times to clone the prev object) (ATTENTION: Increase loop index by keeping consecutives in mind!)
#   * For width * height times: read block
#     - Block ID: short (16 bit)
#     - If block contains tile entity (TODO: Figure out how this is determined):
#       * Read entity chunk (TODO: Continue reading SaveVersion.java line 198 onward)
#       *
#

BLOCKS = {
    # Actually air
    0: "titanium2",
    # Actually spawn
    1: "titanium1",
    2: "deepwater",
    3: "water",
    4: "tainted-water",
    5: "tar",
    6: "stone1",
    7: "craters1",
    8: "char1",
    9: "sand1",
    10: "darksand1",
    11: "ice1",
    12: "snow1",
    13: "darksand-tainted-water",
    14: "holostone1",
    15: "rocks1",
    16: "sporerocks1",
    17: "icerocks1",
    18: "cliffs1",
    19: "spore-pine",
    20: "snow-pine",
    21: "pine",
    22: "shrubs1",
    23: "white-tree",
    24: "white-tree-dead",
    25: "spore-cluster1",
    26: "ice-snow1",
    27: "sand-water",
    28: "darksand-water",
    29: "dunerocks1",
    30: "sandrocks1",
    31: "moss1",
    32: "spore-moss1",
    33: "shale1",
    34: "shalerocks1",
    35: "shale-boulder1",
    36: "sand-boulder1",
    37: "grass1",
    38: "salt",
    39: "metal-floor",
    40: "metal-floor-damaged1",
    41: "metal-floor-2",
    42: "metal-floor-3",
    43: "metal-floor-5",
    44: "ignarock1",
    45: "magmarock1",
    46: "hotrock1",
    47: "snowrocks1",
    48: "rock1",
    49: "snowrock1",
    50: "saltrocks1",
    51: "dark-panel-1",
    52: "dark-panel-2",
    53: "dark-panel-3",
    54: "dark-panel-4",
    55: "dark-panel-5",
    56: "dark-panel-6",
    57: "dark-metal1",
    58: "pebbles1",
    59: "tendrils1",
    60: "copper1",
    61: "lead1",
    62: "scrap1",
    63: "coal1",
    64: "titanium1",
    65: "thorium1",
    66: "silicon-smelter",
    67: "kiln",
    68: "graphite-press",
    69: "plastanium-compressor",
    70: "multi-press",
    71: "phase-weaver",
    72: "alloy-smelter",
    73: "pyratite-mixer",
    74: "blast-mixer",
    75: "cryofluidmixer-top",
    76: "melter",
    77: "separator",
    78: "spore-press",
    79: "pulverizer",
    80: "incinerator",
    81: "coal-centrifuge",
    82: "power-source",
    83: "power-void",
    84: "item-source",
    85: "item-void",
    86: "liquid-source",
    87: "liquid-void",
    88: "message",
    89: "illuminator",
    90: "copper-wall",
    91: "copper-wall-large",
    92: "titanium-wall",
    93: "titanium-wall-large",
    94: "plastanium-wall",
    95: "plastanium-wall-large",
    96: "thorium-wall",
    97: "thorium-wall-large",
    98: "door",
    99: "door-large",
    100: "phase-wall",
    101: "phase-wall-large",
    102: "surge-wall",
    103: "surge-wall-large",
    104: "mender",
    105: "mend-projector",
    106: "overdrive-projector",
    107: "force-projector",
    108: "shock-mine",
    109: "scrap-wall1",
    111: "scrap-wall-large1",
    112: "scrap-wall-huge1",
    113: "scrap-wall-gigantic",
    114: "thruster",
}


"""
class Block(IntEnum):
    # Mappings extracted from mindustry source file "Blocks.java"
    # environment
    air, spawn, deepwater, water, taintedWater, tar, stone, craters, charr, sand, darksand, ice, snow, darksandTaintedWater,
    holostone, rocks, sporerocks, icerocks, cliffs, sporePine, snowPine, pine, shrubs, whiteTree, whiteTreeDead, sporeCluster,
    iceSnow, sandWater, darksandWater, duneRocks, sandRocks, moss, sporeMoss, shale, shaleRocks, shaleBoulder, sandBoulder, grass, salt,
    metalFloor, metalFloorDamaged, metalFloor2, metalFloor3, metalFloor5, ignarock, magmarock, hotrock, snowrocks, rock, snowrock, saltRocks,
    darkPanel1, darkPanel2, darkPanel3, darkPanel4, darkPanel5, darkPanel6, darkMetal,
    pebbles, tendrils,

    # ores
    oreCopper, oreLead, oreScrap, oreCoal, oreTitanium, oreThorium,

    # crafting
    siliconSmelter, kiln, graphitePress, plastaniumCompressor, multiPress, phaseWeaver, surgeSmelter, pyratiteMixer, blastMixer, cryofluidMixer,
    melter, separator, sporePress, pulverizer, incinerator, coalCentrifuge,

    # sandbox
    powerSource, powerVoid, itemSource, itemVoid, liquidSource, liquidVoid, message, illuminator,

    # defense
    copperWall, copperWallLarge, titaniumWall, titaniumWallLarge, plastaniumWall, plastaniumWallLarge, thoriumWall, thoriumWallLarge, door, doorLarge,
    phaseWall, phaseWallLarge, surgeWall, surgeWallLarge, mender, mendProjector, overdriveProjector, forceProjector, shockMine,
    scrapWall, scrapWallLarge, scrapWallHuge, scrapWallGigantic, thruster, //ok, these names are getting ridiculous, but at least I don't have humongous walls yet

    # transport
    conveyor, titaniumConveyor, armoredConveyor, distributor, junction, itemBridge, phaseConveyor, sorter, invertedSorter, router, overflowGate, underflowGate, massDriver,

    # liquid
    mechanicalPump, rotaryPump, thermalPump, conduit, pulseConduit, platedConduit, liquidRouter, liquidTank, liquidJunction, bridgeConduit, phaseConduit,

    # power
    combustionGenerator, thermalGenerator, turbineGenerator, differentialGenerator, rtgGenerator, solarPanel, largeSolarPanel, thoriumReactor,
    impactReactor, battery, batteryLarge, powerNode, powerNodeLarge, surgeTower, diode,

    # production
    mechanicalDrill, pneumaticDrill, laserDrill, blastDrill, waterExtractor, oilExtractor, cultivator,

    # storage
    coreShard, coreFoundation, coreNucleus, vault, container, unloader, launchPad, launchPadLarge,

    # turrets
    duo, scatter, scorch, hail, arc, wave, lancer, swarmer, salvo, fuse, ripple, cyclone, spectre, meltdown,

    # units
    commandCenter, draugFactory, spiritFactory, phantomFactory, wraithFactory, ghoulFactory, revenantFactory, daggerFactory, crawlerFactory, titanFactory,
    fortressFactory, repairPoint,

    # upgrades
    dartPad, deltaPad, tauPad, omegaPad, javelinPad, tridentPad, glaivePad
"""


def read_msav_header(data: bytearray) -> bytearray:
    if not data.startswith("MSAV".encode("ASCII")):
        raise Exception("Invalid save file")
    return data[4:]


def read_version(data: bytearray) -> bytearray:
    version = int.from_bytes(data[:4], byteorder="big", signed=False)
    assert version == 2
    print("Version: {}".format(version))
    return data[4:]


def read_metadata(data: bytearray) -> Tuple[Dict[str, str], bytearray]:
    """
    Parses metadata region of the save file.
    Returns: Metadata map and remaining bytearray
    """
    # Discard 4 byte region length we don't care about ATM
    data = data[4:]
    num_entries: int = int.from_bytes(data[:2], byteorder="big", signed=True)
    print("Number of metadata map entries: {}".format(num_entries))
    data = data[2:]
    metadata: Dict[str, str] = dict()
    for i in range(0, num_entries):
        # Read map key
        (key, data) = utf8m_java_to_utf8(data)
        # Read map value
        (value, data) = utf8m_java_to_utf8(data)
        metadata[key] = value
    print("Metadata: {}".format(metadata))
    return (metadata, data)


def read_content(data: bytearray) -> bytearray:
    # Discard 4 byte region length we don't care about ATM
    data = data[4:]
    mapped: int = int.from_bytes(data[:1], byteorder="big", signed=False)
    data = data[1:]
    for _ in range(0, mapped):
        _content_type: int = int.from_bytes(data[:1], byteorder="big", signed=False)
        data = data[1:]
        total: int = int.from_bytes(data[:2], byteorder="big", signed=True)
        data = data[2:]
        for _ in range(0, total):
            # Read a string and discard it
            (content_string, data) = utf8m_java_to_utf8(data)
            print(content_string)
    return data


def read_map(
    data: bytearray,
) -> Tuple[int, int, List[List[int]], List[List[int]], List[List[int]], bytearray]:
    """
    Parse actual map data.
    Returns: Map width and height, as well as list of lists describing
    floor (ground) tile type, another one describing ore type, another one describing block type,
    and remaining unread part of the bytearray.
    """
    # Discard 4 byte region length we don't care about ATM
    data = data[4:]
    width = int.from_bytes([data[0], data[1]], byteorder="big", signed=False)
    height = int.from_bytes([data[2], data[3]], byteorder="big", signed=False)
    data = data[4:]
    print("Width {}, height {}".format(width, height))
    # Read floor and ore IDs
    floor_ids: List[List[int]] = list()
    for _ in range(0, width):
        inner_list = list()
        for _ in range(0, height):
            inner_list.append(0)
        floor_ids.append(inner_list)

    ore_ids: List[List[int]] = list()
    for _ in range(0, width):
        inner_list = list()
        for _ in range(0, height):
            inner_list.append(0)
        ore_ids.append(inner_list)
    x_pos = 0
    y_pos = 0
    while ((x_pos - 1) <= width) and ((y_pos - 1) <= height):
        floor_id = int.from_bytes(data[:2], byteorder="big", signed=True)
        data = data[2:]
        ore_id = int.from_bytes(data[:2], byteorder="big", signed=True)
        data = data[2:]
        # Repetitive tiles are stored by marking how many tiles are identical to the one we just read
        consecutives = int.from_bytes(data[:1], byteorder="big", signed=False)
        data = data[1:]

        for _ in range(0, consecutives):
            if x_pos == (width - 1):
                y_pos += 1
                x_pos = 0
            else:
                x_pos += 1

            if y_pos >= height:
                return (width, height, floor_ids, ore_ids, list(), data)

            floor_ids[x_pos][y_pos] = floor_id
            ore_ids[x_pos][y_pos] = ore_id


def utf8m_java_to_utf8(data: bytearray) -> Tuple[str, bytearray]:
    """
    Convert java-serialized String modified UTF8 with prepended length to proper UTF-8.
    :return: string and remaining byte slice with string removed
    """

    length = int.from_bytes(data[:2], byteorder="big", signed=False)
    data = data[2:]
    utf8_string: str = data[:length].decode("mutf-8")
    return (utf8_string, data[length:])


def floor_ids_to_png(width: int, height: int, floor_ids: List[List[int]]):
    # Create image with appropriate size
    # Map names to filenames
    images = dict()
    root_directory = "Mindustry/core/assets-raw/sprites/blocks/"
    for key in BLOCKS.keys():
        directory = root_directory
        if key <= 59:
            directory = "{}environment/".format(root_directory)
        # Ores
        elif key <= 65:
            directory = "{}environment/".format(root_directory)
        elif key <= 81:
            directory = "{}production/".format(root_directory)
        elif key <= 83:
            directory = "{}power/".format(root_directory)
        elif key <= 87:
            directory = "{}production/".format(root_directory)
        elif key == 88:
            directory = "{}extra/".format(root_directory)
        elif key == 89:
            directory = "{}power/".format(root_directory)
        elif key <= 103:
            directory = "{}walls/".format(root_directory)
        elif key <= 108:
            directory = "{}defense/".format(root_directory)
        elif key <= 114:
            directory = "{}walls/".format(root_directory)
        filename = "{}{}.png".format(directory, BLOCKS[key])
        img = Image.open(filename)
        images[BLOCKS[key]] = img

    map_img = Image.new(size=(width * 16, height * 16), mode="RGBA")
    for (i, col) in enumerate(floor_ids):
        print("{}".format(len(col)))
        for (j, floor_id) in enumerate(col):
            # Find filename for given tile
            floor_name = BLOCKS[floor_id]
            # Load the image and put it into the correct place in the bigger map image
            print(floor_name)
            img = images[floor_name]
            map_img.paste(im=img, box=(i * 16, j * 16))
    map_img.save(fp="map.png", format="png")
    map_img.show()


# Register java's modified UTF-8 as string codec
codecs.register(py2jdbc.mutf8.info)

print(sys.argv)
if len(sys.argv) != 2:
    raise Exception("Usage: ./parse-save.py <savefile>")
with open(sys.argv[1], "rb") as f:
    print("Decompressing...")
    decompressed = zlib.decompress(f.read())
    with open("decompressed", "wb") as f2:
        f2.write(decompressed)
    savedata = bytearray(decompressed)
    print("Reading header")
    savedata = read_msav_header(savedata)
    print("Reading version")
    savedata = read_version(savedata)
    print("Reading metadata")
    (metadata, savedata) = read_metadata(savedata)
    with open("post-metadata", "wb") as f3:
        f3.write(savedata)
    print("Reading content")
    savedata = read_content(savedata)
    print("Reading actual map data...")
    (width, height, floor_ids, ore_ids, _, savedata) = read_map(savedata)
    print("Width: {}, height: {}".format(width, height))
    floor_ids_to_png(width, height, floor_ids)
