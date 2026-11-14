"""Constructors for panel items, layouts and inport connections.

Shared by the hybrid builders. These functions were first written in the
Panel Lab builder; they live here so no device depends on another device's
workspace.
"""
from __future__ import annotations

from bitwig_nitro import decompile_nitrobin, serialize_nitrobin  # noqa: F401  (re-exported)

from devices.common import document_graph as helpers  # noqa: F401  (re-exported)


def value(kind, data=None):
    return {"type": kind} if kind == "NullObject" else {"type": kind, "value": data}


def obj(cid, name, fields):
    return {"class_id": cid, "class_name": name, "fields": fields}


def field(o, name, data, kind=None):
    if kind:
        o["fields"][name] = value(kind, data)
    else:
        o["fields"][name]["value"] = data


def grid_layout(x, y, width, height):
    return obj(1694, "grid_panel_item_layout_settings", {
        f"{name}#{fid}": value("Int16", number)
        for name, fid, number in (("x", 6215, x), ("y", 6216, y), ("width", 6217, width), ("height", 6218, height))
    })


def layout(o, x, y, width, height):
    field(o, "layout_settings#6226", grid_layout(x, y, width, height), "Object")


def connection(source=None, port=0):
    return {"Object": obj(105, "inport_connection", {
        "source_component#248": value("ObjectRef", source) if source else value("NullObject"),
        "outport_index#249": value("Byte", port),
        "unconnected_value#6443": value("Double", 0.0),
    })}


def connections(o, items):
    settings = o["fields"]["settings#6194"]["value"]
    field(settings, "inport_connections#614", items)
