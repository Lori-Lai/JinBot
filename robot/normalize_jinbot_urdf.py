#!/usr/bin/env python3

from __future__ import annotations

from pathlib import Path
import xml.etree.ElementTree as ET


ROOT = Path(__file__).resolve().parent
URDF_PATH = ROOT / "robot.urdf"
ASSETS_DIR = ROOT / "assets"


def indent(elem: ET.Element, level: int = 0) -> None:
    i = "\n" + level * "  "
    if len(elem):
        if not elem.text or not elem.text.strip():
            elem.text = i + "  "
        for child in elem:
            indent(child, level + 1)
        if not child.tail or not child.tail.strip():
            child.tail = i
    elif level and (not elem.tail or not elem.tail.strip()):
        elem.tail = i


def normalize_mesh_filename(filename: str) -> str:
    mesh_name = filename
    if filename.startswith("file://"):
        mesh_name = Path(filename.removeprefix("file://")).name
    elif "assets/" in filename:
        mesh_name = filename.split("assets/", 1)[1]
    elif "/" in filename:
        mesh_name = filename.rsplit("/", 1)[1]

    return f"assets/{mesh_name}"


def main() -> None:
    tree = ET.parse(URDF_PATH)
    root = tree.getroot()

    joints = {joint.attrib["name"]: joint for joint in root.findall("joint")}
    links = root.findall("link")

    rename_map: dict[str, str] = {}

    # Root/base link: common parent of the two hip-yaw joints.
    base_parent = joints["r_hip_yaw"].find("parent").attrib["link"]
    if joints["l_hip_yaw"].find("parent").attrib["link"] != base_parent:
        raise RuntimeError("Left/right hip yaw do not share the same parent link.")
    rename_map[base_parent] = "base_link"

    # Each joint's child link becomes "<joint_name>_link".
    for joint_name, joint in joints.items():
        child_link = joint.find("child").attrib["link"]
        rename_map[child_link] = f"{joint_name}_link"

    # Rename link definitions.
    for link in links:
        old_name = link.attrib["name"]
        if old_name in rename_map:
            link.attrib["name"] = rename_map[old_name]

        for mesh in link.findall(".//mesh"):
            mesh.attrib["filename"] = normalize_mesh_filename(mesh.attrib["filename"])

    # Rename parent/child references.
    for joint in joints.values():
        parent = joint.find("parent")
        child = joint.find("child")
        parent.attrib["link"] = rename_map.get(parent.attrib["link"], parent.attrib["link"])
        child.attrib["link"] = rename_map.get(child.attrib["link"], child.attrib["link"])

    indent(root)
    xml = ET.tostring(root, encoding="unicode")
    URDF_PATH.write_text('<?xml version="1.0" ?>\n' + xml + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
