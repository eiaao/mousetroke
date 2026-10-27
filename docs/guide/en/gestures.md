---
title: "Mouse gestures"
description: "Record mouse strokes and shapes, combine buttons, wheel and keyboard keys, and tune gesture recognition."
order: 3
---

# Mouse gestures

## Record a directional stroke

Open **All apps → Mouse gestures → New gesture**. Name it, then choose **How it starts**: **Hold down the right button**, **Hold down the middle button**, **Hold down the side button 1** or **Hold down the side button 2**. Side buttons correspond to X1 and X2.

In the drawing area, hold the left or right button, draw and release. The recording canvas accepts either button independently of the trigger you selected. Straight segments become a directional stroke, such as `←` or `↓→`; redraw to replace it. Click **Choose action**, search or browse the [action library](action-library.md), then select a preset and fill any parameters. Use **Advanced** for custom keys, commands, Lua or menus; see [Actions](actions.md). Try the gesture outside settings with its actual trigger button.

## Record a shape

Curves such as C, O or S are shape gestures. The recorder detects a line or shape automatically. If necessary, click **Use as a shape** or **Use as a line** after drawing, or choose **+ Add a step → Draw a shape**.

Draw the same shape 2–3 times, keeping the same starting point and direction. Release after each sample. At least one valid sample is needed to save; the recorder holds up to five samples and can remove individual samples. Watch its consistency and conflict messages. Shape gestures run when you release the trigger button.

## Add buttons, wheel or keys

Under **Then**, click **+ Add a step**. You can add **Draw a stroke**, **Draw a shape**, a mouse button under **Click a mouse button**, **Up** / **Down** under **Scroll the wheel**, or **Hold a keyboard key**. The last option records the keyboard key you press.

For example, start with the right button and add a left click, or add wheel-up to run a volume action repeatedly while holding the trigger. Steps are ordered: perform them in the displayed order while retaining the trigger button. Remove a step with its × button.

For directional gestures, **Run as soon as recognized** runs without waiting for release. It suits repeated wheel actions. Shapes always wait for release. **Don’t show the gesture name while drawing** hides the name for that binding.

## Recognition and appearance

Under **Preferences → Recognition**:

| Setting | Effect |
|---|---|
| Hold-still time before treating it as a click | Default 500 ms; slider 150–1500 ms. Hold without movement for this long to pass through a normal click/drag. |
| Stroke sensitivity | Default 30 px; slider 10–80 px. Smaller values recognize shorter strokes but make accidental gestures easier. |
| Gestures act on | Choose **the window under the cursor** (default) or **the active window**. |

Under **Appearance**, **Show the trail**, **Trail width**, **Trail color**, **When recognized**, **Fade out when done**, **Show the gesture name while drawing** and **Gesture name color** control feedback. There is no shape-recognition threshold slider here.

## Find, copy and turn off

Press **Ctrl+F** on Windows or **⌘+F** on Mac to search the current list by name, pattern or action. Use **Mouse gestures** / **Screen edges** and the button/position filters; **Off** shows disabled entries.

Select a gesture and click **Duplicate** to make a copy (Ctrl+D / ⌘+D). Give the copy a different pattern: identical drawings in one scope conflict, so do not rely on their list order to choose an action. Use the editor's **On** / **Off** switch or Space on a selected list row to disable it without deleting it. Disabled bindings no longer reserve their trigger by themselves.

Next: [screen edges](screen-edges.md), [per-app overrides](per-app.md), [send-keys syntax](key-sequences.md).
