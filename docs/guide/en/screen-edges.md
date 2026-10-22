---
title: "Screen corners and edges"
description: "Configure hot corners, edge scrolling for volume and brightness, edge clicks and rubbing gestures on Windows and Mac."
order: 4
---

# Screen corners and edges

## Try the defaults

Under **All apps → Screen edges**, every corner or edge binding is editable.

| Trigger | Windows default | Mac default |
|---|---|---|
| Push into bottom-left corner | Start menu | Spotlight |
| Push into bottom-right corner | Show desktop | Show desktop (F11) |
| Push into top-left corner | Switch app (Alt+Tab) | Mission Control (Control+Up) |
| Push into top-right corner | Task view (Win+Tab) | Binding disabled initially |
| Scroll at top edge | Volume up/down | Volume up/down |
| Middle-click at top edge | Mute | Mute |
| Right-click at top edge | Play / pause | Play / pause |
| Scroll at bottom edge | Brightness up/down | Brightness bindings are disabled initially |
| Rub right edge up/down twice | Task Manager binding; enable Run as soon as recognized first | Activity Monitor |

Corner actions need no button: push the pointer into the corner. Edge scrolling and clicks need the pointer touching the chosen edge. Rubbing means moving along the edge in the recorded directions, rather than scrolling the wheel.

## Configure an edge trigger

1. Choose **Screen edges → New edge trigger** in the desired app scope or **All apps**.
2. Under **How it starts**, choose **Screen edge** and select a side or corner in the diagram.
3. Leave **Then** empty for a corner action, or use **+ Add a step** to add wheel-up/down, a mouse click or a stroke along the edge.
4. Choose an [action](actions.md). For volume, choose **Media & system → Volume up** or **Volume down**. Enable **Run as soon as recognized** for repeated scrolling or button-free rubbing.

For rubbing, add **Draw a stroke** and record the alternating directions. A useful example is `↑↓↑↓` against the right edge. Try with an innocuous action such as a Lua toast before assigning something disruptive.

## Brightness and platform differences

Windows **Brightness up** / **Brightness down** starts with the monitor under the cursor, adjusts by 10 percentage points and shows the result. It tries DDC/CI, then WMI. External displays must support and allow DDC/CI. The WMI fallback uses the first WMI brightness device, usually a built-in panel; it is not matched to the cursor's monitor. If both routes fail, an on-screen message appears. See [Lua recipes](lua-examples.md#brightness-under-the-cursor) for a smaller step.

Mac **Media & system → Brightness up** / **Brightness down** sends system brightness-key events for the built-in display. It does not select the screen under the cursor and does not support external DDC/CI displays. The bottom-edge defaults are disabled. Lua brightness APIs are also unsupported on Mac.

On Mac, Mousetroke warns if a system Hot Corner overlaps a configured corner. Change one of the assignments to avoid competing actions. System shortcuts such as F11 or Control+Up depend on the macOS shortcut settings.

App exclusions and fullscreen detection suppress edge triggers too. See [per-app rules](per-app.md) when an edge works in one app but not another.
