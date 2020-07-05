#!/usr/bin/env python3

import zlib
import time
import sys
from typing import Dict, Tuple, List
import codecs
import py2jdbc.mutf8
from PIL import Image
import os
from enum import IntEnum, auto
from convert_assets import rescale_sprites_if_needed
import convert_assets

def log(msg, header = "INFO"):

    logstring = "[{}]: {}".format(header, msg)
    #convert_assets.LOGFILE.write(logstring + "\n")
    #convert_assets.LOGFILE.flush()
    

    print(logstring)


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


BLOCK_NAMES: List[str] = [
    # Actually air
    "titanium2",
    # Actually spawn
    "titanium1",
    "deepwater",
    "water",
    "tainted-water",
    "tar",
    "stone1",
    "craters1",
    "char1",
    "sand1",
    "darksand1",
    "ice1",
    "snow1",
    "darksand-tainted-water",
    "holostone1",
    "rocks1",
    "sporerocks1",
    "icerocks1",
    "cliffs1",
    "spore-pine",
    "snow-pine",
    "pine",
    "shrubs1",
    "white-tree",
    "white-tree-dead",
    "spore-cluster1",
    "ice-snow1",
    "sand-water",
    "darksand-water",
    "dunerocks1",
    "sandrocks1",
    "moss1",
    "spore-moss1",
    "shale1",
    "shalerocks1",
    "shale-boulder1",
    "sand-boulder1",
    "grass1",
    "salt",
    "metal-floor",
    "metal-floor-damaged1",
    "metal-floor-2",
    "metal-floor-3",
    "metal-floor-5",
    "ignarock1",
    "magmarock1",
    "hotrock1",
    "snowrocks1",
    "rock1",
    "snowrock1",
    "saltrocks1",
    "dark-panel-1",
    "dark-panel-2",
    "dark-panel-3",
    "dark-panel-4",
    "dark-panel-5",
    "dark-panel-6",
    "dark-metal1",
    "pebbles1",
    "tendrils1",
    "copper1",
    "lead1",
    "scrap1",
    "coal1",
    "titanium1",
    "thorium1",
    "silicon-smelter",
    "kiln",
    "graphite-press",
    "plastanium-compressor",
    "multi-press",
    "phase-weaver",
    "alloy-smelter",
    "pyratite-mixer",
    "blast-mixer",
    "cryofluidmixer-top",
    "melter",
    "separator",
    "spore-press",
    "pulverizer",
    "incinerator",
    "coal-centrifuge",
    "power-source",
    "power-void",
    "item-source",
    "item-void",
    "liquid-source",
    "liquid-void",
    "message",
    "illuminator",
    "copper-wall",
    "copper-wall-large",
    "titanium-wall",
    "titanium-wall-large",
    "plastanium-wall",
    "plastanium-wall-large",
    "thorium-wall",
    "thorium-wall-large",
    "door",
    "door-large",
    "phase-wall",
    "phase-wall-large",
    "surge-wall",
    "surge-wall-large",
    "mender",
    "mend-projector",
    "overdrive-projector",
    "force-projector",
    "shock-mine",
    "scrap-wall1",
    "scrap-wall-large1",
    "scrap-wall-huge1",
    "scrap-wall-gigantic",
    "thruster",
    # conveyor multiple versions
    "conveyor-0-0",
    "titanium-conveyor-0-0",
    "armored-conveyor-0-0",
    "distributor",
    "junction",
    # itemBridge multiple versions
    "phase-conduit-bridge",
    "phase-conveyor",
    "sorter",
    "inverted-sorter",
    "router",
    "overflow-gate",
    "underflow-gate",
    "mass-driver",
    "mechanical-pump",
    "rotary-pump",
    "thermal-pump",
    # conduit multiple versions
    "conduit-top-0",
    # pulseConduit multiple versions
    "pulse-conduit-top-0",
    # platedConduit multiple versions
    "plated-conduit-top-0",
    # liquidRouter multiple versions
    "liquid-router-top",
    # liquidTank multiple versoins
    "liquid-tank-top",
    "liquid-junction",
    "bridge-conduit",
    "phase-conduit",
    "combustion-generator",
    "thermal-generator",
    "turbine-generator",
    "differential-generator",
    "rtg-generator",
    "solar-panel",
    "solar-panel-large",
    "thorium-reactor",
    "impact-reactor",
    "battery",
    "battery-large",
    "power-node",
    "power-node-large",
    "surge-tower",
    "diode",
]

BLOCKS: Dict[int, str] = {i: k for (i, k) in enumerate(BLOCK_NAMES)}

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
    if not version == 2:
        print("Unknown map format version: {}".format(version))
        raise Exception("Unknown map format version")
    else:
        log("Version: {}".format(version), "MAP")
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
    # print("Metadata: {}".format(metadata))
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
            # print(content_string)
    return data


def read_map(
    data: bytearray,
) -> Tuple[int, int, List[List[int]], List[List[int]], List[List[int]], bytearray]:
    global BLOCKS
    """
    Parse actual map data.
    Returns: Map width and height, as well as list of lists describing
    floor (ground) tile type, another one describing ore type, another one describing block type,
    and remaining unread part of the bytearray.
    """
    used_sprites = {}
    # Discard 4 byte region length we don't care about ATM
    print("Region length data: {}".format(data[0:4].hex()))
    data = data[4:]
    print("Map width and height data: {}".format(data[0:4].hex()))
    width = int.from_bytes([data[0], data[1]], byteorder="big", signed=False)
    height = int.from_bytes([data[2], data[3]], byteorder="big", signed=False)
    print("Width {}, height {}".format(width, height))

    data = data[4:]
    print("Actual map data (first 16 bytes): {}".format(data[0:16].hex()))
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
        if floor_id < 0 or floor_id > 29000:
            print("WRONG ID: " + str(floor_id) + " bytes: {}".format(data[0:4].hex()))
            time.sleep(10)
        if floor_id not in used_sprites:
            used_sprites[floor_id] = BLOCKS[floor_id]
            log("found Floor ID: {} ({})".format(floor_id, BLOCKS[floor_id]))

        data = data[2:]
        ore_id = int.from_bytes(data[:2], byteorder="big", signed=True)
        data = data[2:]
        # Repetitive tiles are stored by marking how many tiles are identical to the one we just read
        consecutives = int.from_bytes(data[:1], byteorder="big", signed=False)
        data = data[1:]

        for _ in range(0, consecutives + 1):

            try:
                floor_ids[x_pos][y_pos] = floor_id
            except Exception:
                print("Map except X: {}, Y: {}".format(x_pos, y_pos))
            ore_ids[x_pos][y_pos] = ore_id

            if x_pos == (width - 1):
                y_pos += 1
                x_pos = 0
            else:
                x_pos += 1

            if y_pos >= height:
                return (width, height, floor_ids, ore_ids, list(), data, used_sprites)


def utf8m_java_to_utf8(data: bytearray) -> Tuple[str, bytearray]:
    """
    Convert java-serialized String modified UTF8 with prepended length to proper UTF-8.
    :return: string and remaining byte slice with string removed
    """

    length = int.from_bytes(data[:2], byteorder="big", signed=False)
    data = data[2:]
    utf8_string: str = data[:length].decode("mutf-8")
    return (utf8_string, data[length:])


def floor_ids_to_png(
    width: int, height: int, floor_ids: List[List[int]], png_path: str, used_sprites: Dict[int, str]
):
    # Create image with appropriate size
    # Map names to filenames
    import glob

    images = dict()
    root_directory = "Mindustry/core/assets-raw/sprites/blocks/"
    for key in range(0, len(BLOCKS)):
        if key in used_sprites:
            filename = "{}.png".format(BLOCKS[key])
            # Find out in which subdir the file resides
            matches = glob.glob(root_directory + "/**/" + filename, recursive=True)
            log("Sprite for ID {}: {}".format( key, matches[0]), "MAP")
            # print(matches)
            sprite_path_after_resize = rescale_sprites_if_needed([matches[0]])[0]
            try:
                
                log("tmpSprite for ID {}: {}".format( key, sprite_path_after_resize), "MAP")
                img = Image.open(sprite_path_after_resize)
                
                images[BLOCKS[key]] = img
            except Exception:
                print("No such file: {}".format(sprite_path_after_resize))

    map_img = Image.new(size=(width * 16, height * 16), mode="RGBA")
    for (i, col) in enumerate(floor_ids):
        # print("{}".format(len(col)))
        for (j, floor_id) in enumerate(col):
            # Find filename for given tile
            if floor_id not in BLOCKS.keys():
                # print("Unknown floor block with ID: {}, substituting".format(floor_id))
                # Placeholder until bug is fixed
                # TODO: Remove
                floor_name = BLOCKS[0]
            else:
                floor_name = BLOCKS[floor_id]
            img = images[floor_name]
            map_img.paste(im=img, box=(i * 16, j * 16))
    map_img.save(fp=png_path, format="png")


def map_file_to_map(path: str) -> Tuple[int, int, str, str]:
    """
    Converts a mindustry .msav map to PNG.
    Returns tuple containing width, height, map name and path to PNG.
    """
    log("TESTETSTETSETSET!!!!!!!!")
    with open(path, "rb") as f:
        print("Decompressing...")
        decompressed = zlib.decompress(f.read())
        savedata = bytearray(decompressed)
        print("Reading header")
        savedata = read_msav_header(savedata)
        print("Reading version")
        savedata = read_version(savedata)
        print("Reading metadata")
        (metadata, savedata) = read_metadata(savedata)
        print("Reading content")
        savedata = read_content(savedata)
        print("Reading actual map data...")
        (width, height, floor_ids, ore_ids, _, savedata, used_sprites) = read_map(savedata)
        print("Width: {}, height: {}".format(width, height))
        print("Converting to PNG")
        # TODO: Tempdir
        png_path = "{}-map.png".format(os.path.splitext(path)[0])
        floor_ids_to_png(width, height, floor_ids, png_path, used_sprites)
        return (width, height, metadata["name"], png_path)


# Register java's modified UTF-8 as string codec
codecs.register(py2jdbc.mutf8.info)

if __name__ == "__main__":
    print(sys.argv)
    if len(sys.argv) != 2:
        raise Exception("Usage: ./parse-save.py <savefile>")
    map_file_to_map(sys.argv[1])
