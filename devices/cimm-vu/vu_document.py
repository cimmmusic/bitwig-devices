"""VU Meter device document, adapted from Polarity's Blueprint Gain (Windows 6.1.3 wire types).

Inspector panel, list/on-off parameters, runtime readouts and panel expressions
follow the structures in Polarity's published Multiband FX-4, Normalizer and OTT.
"""
from __future__ import annotations

import hashlib

from devices.common.document_graph import iter_owned_objects
from devices.common.panel_helpers import connection, grid_layout, obj, value as v

PANEL_WIDTH, PANEL_HEIGHT = 88, 53


def owned(*objects):
    return [{"Object": item} for item in objects]


def settings(connections=()):
    desktop = obj(17, "desktop_settings", {
        "x#17": v("Int16", 0), "y#18": v("Int16", 0),
        "color#19": v("Color", [0.5, 0.5, 0.5, 1.0]),
    })
    return v("Object", obj(236, "component_settings", {
        "desktop_settings#612": v("Object", desktop),
        "inport_connections#614": v("ObjectList", list(connections)),
        "is_polyphonic#615": v("Bool", False),
    }))


def proxy(*, output=False, note=False):
    caption = " Note out" if note else " Audio in (PARENT)" if output else " Audio out (PARENT)"
    port_fields = {
        "name#498": v("String", ""), "description#4741": v("String", ""),
        "decorated_name#499": v("String", caption), "is_inport#500": v("Bool", output),
        "is_optional#1384": v("Bool", False), "exclude_from_graph#7174": v("Bool", False),
    }
    if note:
        port_fields["oversampling#13326"] = v("Byte", 0)
        port = obj(61, "note_port", port_fields)
    else:
        port_fields.update({"channel_count#372": v("Byte", 3), "oversampling#13324": v("Int32", 0)})
        port = obj(242, "audio_port", port_fields)
    return obj(50 if output else 154, "proxy_out_port_component" if output else "proxy_in_port_component", {
        "settings#6194": settings(), "port#301": v("Object", port),
    })


import math

# Bitwig display units.
# Confirmed in Bitwig 6.1.4's Inspector: 3 shows "s", 4 shows "Hz".
UNIT_NONE, UNIT_PERCENT, UNIT_DB, UNIT_SECONDS = 0, 1, 2, 3
# Decimal parameters: (preset id, name, short name, min, max, default, smoothed, unit).
# REF keeps its id from v1. Percent values are fractions (0.015 shows 1.5 %).
PARAMETERS = (
    ("REF", "0 VU Reference", "Ref", -30.0, 0.0, -18.0, True, UNIT_DB),
    ("TRIM", "Trim", "Trim", -24.0, 24.0, 0.0, True, UNIT_DB),
    ("OVERSHOOT", "Overshoot", "Overshoot", 0.0, 0.10, 0.015, False, UNIT_PERCENT),
    ("HOLD", "Peak Hold Time", "Hold", 0.0, 10.0, 2.0, False, UNIT_SECONDS),
    ("CLIP_LED", "Clip LED Threshold", "Clip LED", -24.0, 0.0, -3.0, False, UNIT_DB),
)
# List parameters: (preset id, name, items, default index).
CHOICES = (
    ("MODE", "Meter Type", ("VU", "PPM (DIN)", "RMS"), 0),
    ("CHANNELS", "Channels", ("L / R", "M / S"), 0),
)
# On/off parameters: (preset id, name, default).
SWITCHES = (
    ("HOLD_ON", "Peak Hold", True),
    ("FORCE_MONO", "Force Mono", False),
    ("VALUES", "Values", True),
)
# Nitro input names, in vu.nitro order after "in".
NITRO_INPUTS = ("ref", "trim", "overshoot", "hold", "led_db", "mode", "channels", "hold_on", "force_mono", "show_values")


def decimal_parameter(ident, name, short, low, high, default, smoothed, unit):
    value_type = obj(123, "decimal_value_type", {
        "min#292": v("Double", low), "max#293": v("Double", high),
        "default_value#891": v("Double", default), "domain#294": v("Byte", 1 if unit == UNIT_DB else 0),
        "engine_domain#295": v("Byte", 1 if unit == UNIT_DB else 0),
        "value_origin#3014": v("Byte", 3 if unit == UNIT_DB else 0),
        "pixel_step_size#1988": v("Double", -1.0), "unit#296": v("Byte", unit),
        "#297": v("Byte", 2), "parameter_smoothing#4433": v("Bool", smoothed),
        "allow_automation_curves#4434": v("Bool", True),
    })
    return obj(289, "decimal_value_atom", {
        "settings#6194": settings(), "channel_count#607": v("Byte", 0),
        "oversampling#608": v("Byte", 0), "name#374": v("String", name),
        "short_name#821": v("String", short), "preset_identifier#701": v("String", ident),
        "modulations_to_ignore#3826": v("String", ""),
        "value_type#702": v("Object", value_type), "value#712": v("Double", default),
        "allow_automation#7489": v("Bool", True), "allow_modulation#7490": v("Bool", True),
    })


def choice_parameter(ident, name, items, default):
    """indexed_value_atom, field for field as in Polarity's Normalizer (THEME)."""
    entries = owned(*(obj(258, "indexed_value_item", {
        "id#650": v("Byte", index), "name#651": v("String", text), "icon#9823": v("String", ""),
        "description#9824": v("String", ""), "deprecated#10795": v("Bool", False),
    }) for index, text in enumerate(items)))
    value_type = obj(155, "indexed_value_type", {
        "items#393": v("ObjectList", entries), "default_value#6958": v("Byte", default),
        "is_periodic#16667": v("Bool", False),
    })
    return obj(180, "indexed_value_atom", {
        "settings#6194": settings(), "channel_count#607": v("Byte", 0), "oversampling#608": v("Byte", 0),
        "preset_identifier#701": v("String", ident), "name#374": v("String", name),
        "short_name#821": v("String", name), "should_inherit_name#10561": v("Bool", False),
        "modulations_to_ignore#3826": v("String", ""), "value_type#702": v("Object", value_type),
        "allow_automation#7489": v("Bool", True), "allow_modulation#7490": v("Bool", False),
        "value#457": v("Byte", default),
    })


def switch_parameter(ident, name, default):
    """boolean_value_atom, as in Polarity's Multiband FX-4 (LINEAR_PHASE)."""
    value_type = obj(198, "boolean_value_type", {
        "icon_name#4755": v("String", ""), "default_value#6957": v("Bool", default),
    })
    return obj(87, "boolean_value_atom", {
        "settings#6194": settings(), "channel_count#607": v("Byte", 0), "oversampling#608": v("Byte", 0),
        "preset_identifier#701": v("String", ident), "name#374": v("String", name),
        "short_name#821": v("String", name), "should_inherit_name#10561": v("Bool", False),
        "modulations_to_ignore#3826": v("String", ""), "value_type#702": v("Object", value_type),
        "allow_automation#7489": v("Bool", True), "allow_modulation#7490": v("Bool", False),
        "value#210": v("Bool", default),
    })


def readout_value(name, ident, connection_entry):
    """Runtime-only value the DSP sends to (Normalizer's settable_decimal_value_atom)."""
    value_type = obj(123, "decimal_value_type", {
        "min#292": v("Double", -60.0), "max#293": v("Double", 12.0),
        # dB values use the decibel domain, exactly as FILAR's Output (unit 2,
        # domain 1, engine_domain 1, value_origin 3); with domain 0 Bitwig shows NaN.
        "default_value#891": v("Double", -60.0), "domain#294": v("Byte", 1),
        "engine_domain#295": v("Byte", 1), "value_origin#3014": v("Byte", 3),
        "pixel_step_size#1988": v("Double", -1.0), "unit#296": v("Byte", UNIT_DB),
        "parameter_smoothing#4433": v("Bool", False), "allow_automation_curves#4434": v("Bool", False),
    })
    return obj(2037, "settable_decimal_value_atom", {
        "settings#6194": settings([connection_entry]), "channel_count#607": v("Byte", 0),
        "oversampling#608": v("Byte", 0), "name#374": v("String", name),
        "short_name#821": v("String", "VU"), "preset_identifier#701": v("String", ident),
        "modulations_to_ignore#3826": v("String", ""),
        "value_type#702": v("Object", value_type), "allow_automation#7489": v("Bool", False),
        "allow_modulation#7490": v("Bool", False), "value#712": v("Double", -60.0),
    })


def nitro_port(name, index, channels, *, output, kind=1):
    """kind 1 = audio port, kind 3 = value port (bool / i32 / sent f32)."""
    return obj(2428, "nitro_port_info", {
        "name#8557": v("String", name), "desc#8558": v("String", ""),
        "channel_count#8559": v("Byte", channels), "index#8560": v("Byte", index),
        "kind#8561": v("Byte", kind), "is_input#8562": v("Bool", not output),
        "is_double_precision#10071": v("Bool", False), "is_compressed_mask#8563": v("Int32", 16777215),
        "is_compressable#8564": v("Bool", False), "event_size#8565": v("Byte", 0),
        "event_type#8566": v("String", ""),
    })


# ---- Panel items --------------------------------------------------------

def label_item(text, x, y, width, height, *, color=6, align=1):
    return obj(1683, "label_panel_item", {
        "layout_settings#6226": v("Object", grid_layout(x, y, width, height)),
        "text#6219": v("String", text), "color#6520": v("Int16", color),
        "horizontal_alignment#6400": v("Byte", align), "vertical_alignment#6401": v("Byte", 1),
    })


def knob_item(binding, tooltip, x, y):
    # Bitwig's default medium knob (size 2, no style / colour overrides):
    # dark cap, white pointer and the orange value ring, like Sampler's Out knob.
    return obj(1687, "knob_panel_item", {
        "layout_settings#6226": v("Object", grid_layout(x, y, 14, 13)),
        "is_visible#6309": v("Bool", True), "is_enabled#6310": v("Bool", True),
        "tooltip_text#6867": v("String", tooltip),
        "data_model#6220": binding, "title#6241": v("String", ""),
        "knob_size#6223": v("Byte", 2), "show_value_popup#10569": v("Bool", True),
        "attribute_id#11012": v("Int16", 712),
    })


def number_field(binding, x, y, width, height, *, enabled, tooltip="", color=6, align=1):
    return obj(1713, "number_field_panel_item", {
        "layout_settings#6226": v("Object", grid_layout(x, y, width, height)),
        "is_visible#6309": v("Bool", True), "is_enabled#6310": v("Bool", enabled),
        "tooltip_text#6867": v("String", tooltip), "data_model#6220": binding,
        "style#6540": v("Byte", 1), "show_value_bar#6581": v("Bool", False),
        "color#7526": v("Int16", color), "text_alignment#10029": v("Byte", align),
        "value_text#10034": v("String", ""),
    })


def popup_chooser(binding, x, y, width, height, tooltip):
    return obj(1714, "popup_chooser_panel_item", {
        "layout_settings#6226": v("Object", grid_layout(x, y, width, height)),
        "data_model#6220": binding, "attribute_id#11012": v("Int16", 457),
        "label_style#6262": v("Byte", 0), "style#6266": v("Byte", 7),
        "color#7528": v("Int16", 6), "tooltip_text#6867": v("String", tooltip),
    })


def button_group(binding, x, y, width, height, tooltip, *, horizontal):
    """button_group_chooser_panel_item, as in Polarity's Orbit (Model / Polyphony)."""
    return obj(1712, "button_group_chooser_panel_item", {
        "layout_settings#6226": v("Object", grid_layout(x, y, width, height)),
        "is_visible#6309": v("Bool", True), "is_enabled#6310": v("Bool", True),
        "tooltip_text#6867": v("String", tooltip), "data_model#6220": binding,
        "attribute_id#11012": v("Int16", 457), "is_horizontal#6255": v("Bool", horizontal),
        "break_count#6256": v("Byte", 0), "button_style#7548": v("Byte", 0),
        "label_style#6257": v("Byte", 0), "color#10143": v("Int16", 6),
    })


def toggle_button(binding, title, x, y, width, height, tooltip):
    return obj(1702, "toggle_button_panel_item", {
        "layout_settings#6226": v("Object", grid_layout(x, y, width, height)),
        "is_visible#6309": v("Bool", True), "is_enabled#6310": v("Bool", True),
        "data_model#6220": binding, "attribute_id#11012": v("Int16", 255),
        "title#6241": v("String", title), "tooltip_text#6867": v("String", tooltip),
        "icon_name#6239": v("String", ""), "style#6240": v("Byte", 4),
        "label_style#6267": v("Byte", 0), "invert_value#6552": v("Bool", False),
        "is_selected#6719": v("Bool", False), "color#7527": v("Int16", 999), "font#7530": v("Byte", 0),
    })


def grid_item(items):
    return obj(1681, "grid_panel_item", {
        "layout_settings#6226": v("NullObject"), "items#6221": v("ObjectList", owned(*items)),
        "border_style#6228": v("Byte", 0), "title#6242": v("String", ""),
        "show_title#6243": v("Bool", False), "title_color#6522": v("Byte", 6),
        "brightness#6245": v("Byte", 0),
    })


def expression(text, target_ref):
    """Show an item only while `text` is true (attribute 6309 = is_visible)."""
    return {"Object": obj(1728, "panel_expression", {
        "expression#6274": v("String", text), "panel_item#6275": target_ref,
        "attribute_id#6316": v("Int16", 6309),
    })}


# ---- Meter geometry, shared with panel.sksl ------------------------------
# Design units: two 480 x 298 cards, a 22 gap and a 30 foot = 982 x 328,
# drawn in a 84 x 28 grid rectangle at (2, 2).
CARD_W, CARD_H, CARD_GAP = 480.0, 298.0, 22.0
DESIGN_W, DESIGN_H = 982.0, 328.0
DRAW_X, DRAW_Y, DRAW_W, DRAW_H = 2, 2, 84, 28
PIVOT = (240.0, 296.0)
ARC_R, ARC_HALF = 188.0, 0.95
LABEL_R = ARC_R + 44.0


def scale_t(db):
    """Same piecewise mapping as scaleT() in panel.sksl."""
    if db <= -10.0:
        return (db + 20.0) * 0.0105
    if db <= -7.0:
        return 0.105 + (db + 10.0) * 0.0443
    if db <= -5.0:
        return 0.238 + (db + 7.0) * 0.0665
    if db <= -3.0:
        return 0.371 + (db + 5.0) * 0.081
    if db <= -1.0:
        return 0.533 + (db + 3.0) * 0.0715
    if db <= 0.0:
        return 0.676 + (db + 1.0) * 0.105
    if db <= 3.0:
        return 0.781 + db * 0.035
    return 0.886 + (db - 3.0) * 0.038


def to_grid(card, x, y):
    dx = card * (CARD_W + CARD_GAP) + x
    return DRAW_X + dx * DRAW_W / DESIGN_W, DRAW_Y + y * DRAW_H / DESIGN_H


def centred_label(text, cx, cy, color):
    """A label centred on (cx, cy) to the nearest half grid unit."""
    cx, cy = round(cx * 2) / 2, round(cy * 2) / 2
    width = 6 if cx == int(cx) else 5
    height = 4 if cy == int(cy) else 5
    return label_item(text, int(cx - width / 2), int(cy - height / 2), width, height, color=color)


SCALE_LABELS = (("-20", -20.0), ("-10", -10.0), ("-7", -7.0), ("-5", -5.0), ("-3", -3.0),
                ("-1", -1.0), ("0", 0.0), ("+3", 3.0), ("+6", 6.0))
GREY, WHITE, ORANGE = 8, 6, 1  # Bitwig label palette indices (seen in Polarity's FILAR)


def meter_labels(card):
    items = []
    for text, db in SCALE_LABELS:
        t = scale_t(db)
        a = -ARC_HALF + 2 * ARC_HALF * t
        x = PIVOT[0] + math.sin(a) * LABEL_R
        y = PIVOT[1] - math.cos(a) * LABEL_R
        if db in (-20.0, 6.0):
            # The end numbers sit beside the arc's ends (as in the mockup), which
            # leaves room for Bitwig's label font between -20 and -10.
            side = -1.0 if db < 0 else 1.0
            x = PIVOT[0] + math.sin(a) * ARC_R + side * 34.0
            y = PIVOT[1] - math.cos(a) * ARC_R - 6.0
        items.append(centred_label(text, *to_grid(card, x, y), ORANGE if db >= 0 else GREY))
    return items


def main_panel(ref, knobs, drawing, readouts, channel_choice, values_switch):
    meter = obj(5517, "", {
        "layout_settings#6226": v("Object", grid_layout(DRAW_X, DRAW_Y, DRAW_W, DRAW_H)),
        "is_visible#6309": v("Bool", True), "is_enabled#6310": v("Bool", True),
        "tooltip_text#6867": v("String", ""), "help_id#10543": v("String", ""),
        "help_location#10546": v("Byte", 0), "data_model#6220": ref(drawing),
    })
    items, expressions = [meter], []
    for card in (0, 1):
        items += meter_labels(card)
        # Channel letter, top left: L / R, or M / S while Channels is M / S.
        for letters, condition in ((("L", "R"), "CHANNELS==0"), (("M", "S"), "CHANNELS==1")):
            letter = centred_label(letters[card], *to_grid(card, 34.0, 34.0), ORANGE)
            items.append(letter)
            expressions.append(expression(condition, ref(letter)))
        # Native readout of the held reading, top right.
        gx, gy = to_grid(card, CARD_W - 74.0, 34.0)
        field = number_field(ref(readouts[card]), round(gx - 6), round(gy - 2), 12, 4, enabled=False,
                             tooltip="Peak-hold reading in dB relative to 0 VU")
        items.append(field)
        expressions.append(expression("VALUES", ref(field)))
    # Knob row: Ref and Trim, the controls used while mixing.
    for (binding, title, tooltip), centre in zip(knobs, (30, 58)):
        items.append(knob_item(binding, tooltip, centre - 7, 32))
        items.append(label_item(title, centre - 10, 46, 20, 4, color=WHITE))
    return obj(1680, "panel", {
        "name#6211": v("String", "Main"), "root_item#6212": v("Object", grid_item(items)),
        "width#6232": v("Int16", PANEL_WIDTH), "height#6233": v("Byte", PANEL_HEIGHT),
        "expressions#6277": v("ObjectList", expressions),
    })


INSPECTOR_WIDTH, INSPECTOR_HEIGHT = 39, 77


def inspector_panel(ref, by_id):
    """Set-and-forget settings in Bitwig's Inspector, laid out like Compressor+:
    centred section headings, label left / orange value right, a vertical list
    chooser and segmented buttons."""
    def heading(text, y):
        return label_item(text, 1, y, 37, 4, color=WHITE, align=1)

    def row_label(text, y, height=4):
        return label_item(text, 2, y, 15, height, color=WHITE, align=0)

    def value(ident, y, tooltip):
        return number_field(ref(by_id[ident]), 17, y, 20, 4, enabled=True, tooltip=tooltip, color=ORANGE, align=2)

    # Vertical rhythm: heading (4 rows), rows of 4-5, a 2-row gap before each heading.
    items = [
        heading("Meter", 1),
        row_label("Type", 6, 12),
        button_group(ref(by_id["MODE"]), 17, 6, 20, 12, "VU, PPM (DIN) or RMS ballistics", horizontal=False),
        row_label("Channels", 19, 5),
        button_group(ref(by_id["CHANNELS"]), 17, 19, 20, 5, "Meter left / right or mid / side", horizontal=True),
        heading("Ballistics", 26),
        row_label("Overshoot", 31),
        value("OVERSHOOT", 31, "VU needle overshoot: 0 % critically damped, 1.5 % classic"),
        heading("Peak Hold", 37),
        toggle_button(ref(by_id["HOLD_ON"]), "Hold", 2, 42, 17, 5, "Show the peak-hold marker"),
        toggle_button(ref(by_id["VALUES"]), "Values", 20, 42, 17, 5, "Show the numeric readouts"),
        row_label("Time", 48),
        value("HOLD", 48, "How long the peak hold stays"),
        heading("Clip LED", 54),
        row_label("Threshold", 59),
        value("CLIP_LED", 59, "Sample peak level that lights the LED amber; red is 0 dBFS"),
        heading("Output", 65),
        toggle_button(ref(by_id["FORCE_MONO"]), "Force Mono", 2, 70, 35, 5, "Sum the output to mono"),
    ]
    return obj(1680, "panel", {
        "name#6211": v("String", "Inspector"), "root_item#6212": v("Object", grid_item(items)),
        "width#6232": v("Int16", INSPECTOR_WIDTH), "height#6233": v("Byte", INSPECTOR_HEIGHT),
        "expressions#6277": v("ObjectList", []),
        "width_is_resizeable#10404": v("Bool", False), "height_is_resizeable#10405": v("Bool", False),
    })


def metadata(project, revision):
    identity = project["identity"]
    fields = {
        "application_version_name": v("String", project["target"]["bitwigVersion"]),
        "creator": v("String", identity["creator"]), "device_creator": v("String", identity["creator"]),
        "device_category": v("String", identity["category"]),
        "device_description": v("String", project["presentation"]["description"]),
        "device_id": v("String", identity["uuid"]), "device_uuid": v("Uuid", identity["uuid"]),
        "device_name": v("String", project["name"]), "device_type": v("String", "audio_to_audio"),
        "has_audio_input": v("Bool", True), "has_audio_output": v("Bool", True),
        "has_note_input": v("Bool", False), "has_note_output": v("Bool", False),
        "preset_name": v("String", project["name"]), "revision_id": v("String", revision),
        "revision_no": v("Int32", 5), "suggest_for_audio_input": v("Bool", True),
        "suggest_for_note_input": v("Bool", False), "tags": v("String", "vu meter level analysis"),
        "type": v("String", "application/bitwig-device"),
    }
    return {**obj(4, "meta", fields), "wide_names": [False] * len(fields)}


def document(project, binary, shader):
    # Bind to Python objects first. Serialize references only after the tree is final.
    bindings = []

    def ref(target, *, list_entry=False):
        key = "ObjectRef" if list_entry else "value"
        wrapper = {"ObjectRef": 0} if list_entry else v("ObjectRef", 0)
        bindings.append((wrapper, key, target))
        return wrapper

    def wire(target, port=0):
        entry = connection(port=port)
        entry["Object"]["fields"]["source_component#248"] = ref(target)
        return entry

    input_proxy = proxy()
    input_owner = connection()
    input_owner["Object"]["fields"]["source_component#248"] = v("Object", input_proxy)
    audio_source = obj(2000, "audio_source_atom", {
        "settings#6194": settings([input_owner]), "channel_count#607": v("Byte", 0),
        "oversampling#608": v("Byte", 0), "port_color#10378": v("Int16", 999),
        "name#3639": v("String", "In"), "preset_identifier#3640": v("String", "IN"),
    })
    compensation = obj(1371, "delay_compensation_atom", {
        "settings#6194": settings([wire(input_proxy)]), "channel_count#607": v("Byte", 0),
        "oversampling#608": v("Byte", 0),
    })
    params = [decimal_parameter(*spec) for spec in PARAMETERS]
    choices = [choice_parameter(*spec) for spec in CHOICES]
    switches = [switch_parameter(*spec) for spec in SWITCHES]
    by_id = {atom["fields"]["preset_identifier#701"]["value"]: atom for atom in (*params, *choices, *switches)}
    controls = [by_id[name] for name in ("REF", "TRIM", "OVERSHOOT", "HOLD", "CLIP_LED", "MODE", "CHANNELS",
                                         "HOLD_ON", "FORCE_MONO", "VALUES")]
    dsp = obj(1721, "nitro_atom", {
        "settings#6194": settings([wire(compensation), *(wire(atom) for atom in controls)]),
        "channel_count#607": v("Byte", 0), "oversampling#608": v("Byte", 0),
        "fft_order#6567": v("Byte", 0), "code_storage#10421": v("ByteArray", [0, *binary]),
        "is_code_storage_binary_ast#14616": v("Bool", True),
        "use_code_path#11360": v("Bool", False), "code_path#11357": v("String", ""),
        # These descriptors follow the inport and outport order in vu.nitro.
        "nitro_inputs#15756": v("ObjectList", owned(
            nitro_port("in", 0, 2, output=False),
            *(nitro_port(name, index, 1, output=False, kind=1 if index <= 5 else 3)
              for index, name in enumerate(NITRO_INPUTS, 1)),
        )),
        "nitro_outputs#15757": v("ObjectList", owned(
            nitro_port("out", 0, 2, output=True), nitro_port("display", 1, 4, output=True),
            nitro_port("read_1", 2, 1, output=True, kind=3), nitro_port("read_2", 3, 1, output=True, kind=3),
            nitro_port("needle_1", 4, 1, output=True, kind=3), nitro_port("needle_2", 5, 1, output=True, kind=3),
        )),
    })
    # Runtime values the DSP sends; their IDs let the mixer mini view (BML mod) read them.
    readouts = [readout_value(name, ident, wire(dsp, port))
                for name, ident, port in (("Reading 1", "READING_1", 2), ("Reading 2", "READING_2", 3))]
    needles = [readout_value(name, ident, wire(dsp, port))
               for name, ident, port in (("Needle 1", "NEEDLE_1", 4), ("Needle 2", "NEEDLE_2", 5))]
    drawing = obj(5512, "", {
        "settings#6194": settings([wire(dsp, 1), *(connection() for _ in range(8))]),
        "channel_count#607": v("Byte", 0), "name#374": v("String", ""),
        "prevent_parent_from_sleeping#11002": v("Bool", False),
        "preset_identifier#701": v("String", "VU_DISPLAY_2"), "#16316": v("WideString", shader),
    })
    output_proxy = proxy(output=True)
    output_proxy["fields"]["settings#6194"] = settings([wire(dsp, 0)])
    knobs = (
        (ref(by_id["REF"]), "Ref", "0 VU reference: sine peak level in dBFS that reads 0 VU"),
        (ref(by_id["TRIM"]), "Trim", "Gain applied to the audio and the meter, in dB"),
    )
    panel = main_panel(ref, knobs, drawing, readouts, by_id["CHANNELS"], by_id["VALUES"])
    inspector = inspector_panel(ref, by_id)
    identity = project["identity"]
    body = obj(151, "device_contents", {
        "child_components#173": v("ObjectList", owned(audio_source, compensation, *params, *choices, *switches, dsp, *readouts, *needles, drawing)),
        "panels#6213": v("ObjectList", owned(panel, inspector)),
        # Tells Bitwig which panel to show in its Inspector (as Multiband FX-4).
        "inspector_panel#9820": ref(inspector),
        "proxy_in_ports#177": v("ObjectList", [ref(input_proxy, list_entry=True), *owned(proxy(note=True))]),
        "proxy_out_ports#178": v("ObjectList", owned(output_proxy)),
        "device_UUID#385": v("Uuid", identity["uuid"]), "device_name#386": v("String", project["name"]),
        "description#2057": v("String", project["presentation"]["description"]),
        "creator#387": v("String", identity["creator"]), "category#390": v("String", identity["category"]),
        "suggest_for_note_input#4846": v("Bool", False), "suggest_for_audio_input#4847": v("Bool", True),
        "settings#6194": v("NullObject"), "fft_order#6566": v("Byte", 0),
        "analysis_fft_order#11047": v("Byte", 11), "analysis_fft_fall_speed#11110": v("Byte", 1),
        "device_revision#13647": v("Byte", 5), "sleep_level_threshold#1977": v("Double", -96.0),
        "sleep_level_timeout#1978": v("Double", 0.05), "prevent_sleep#14608": v("Bool", True),  # keep metering so needles fall in silence
        "comment#388": v("String", ""), "keywords#389": v("String", "vu meter level analysis"),
        "device_type#391": v("Byte", 0), "has_been_modified#392": v("Bool", True),
        "color_tint#13354": v("Color", [0.0, 0.0, 0.0, 0.0]),
    })
    object_ids = {id(item): index for index, item in enumerate(iter_owned_objects(body), 1)}
    for wrapper, key, target in bindings:
        wrapper[key] = object_ids[id(target)]

    revision = hashlib.sha1(f"{identity['uuid']}/v5".encode("utf-8")).hexdigest()
    return {
        "header": {"format_version": 3, "serialization_format": 2, "document_version": 207,
                   "object_offset": 0, "resources_offset": 0, "flags": 0},
        "metadata": metadata(project, revision), "body": body,
        "meta_nonce": "0" * 32, "body_nonce": "0" * 32,
    }
