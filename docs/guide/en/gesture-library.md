---
title: "Gesture library"
description: "Define a gesture once and use it in several apps with different actions: organize and import existing gestures, assign them to other apps, edit shared gestures, and compatibility notes."
order: 3.5
---

# Gesture library

The gesture library separates how you draw from what happens. Define a gesture once, then give it a different action in each app. Re-recording a shared gesture updates every place that uses it, and a shape gesture keeps one set of samples.

## Browse the library

Click **Gestures** in the sidebar. The list is split into **Mouse gestures** and **Screen edges**. Filter by trigger button, use **Unused** to find gestures nothing uses, or search by name, drawing or use. Each card shows how many places use it.

Click a card to open its details. Redraw it at the top; **Used in** lists every app, gesture name and action that uses it, and clicking a row jumps there. On a new install the built-in gestures are already in the library. After upgrading from an older version, your gestures are still spread across apps, so organize them first as described below.

## Organize and import

When the library is empty, click **Organize and import gestures…**; later you can always use **Organize and import…** at the top right. First choose the scope:

- **Shared gestures only (recommended)**: adds the gestures in **All apps** and links identical or similar ones in other apps. Gestures set up only for one app stay app-specific.
- **All gestures**: adds shared gestures and every app-specific gesture.

Click **Next** to review the result. Nothing changes until you confirm:

- Identical drawings are merged automatically. Strokes with the same directions that differ only in segment length are merged by default.
- **Similar shapes — please confirm**: in each group, click **Use this one** on the version to keep, or click **Combine samples**. Groups below the recommended similarity are not merged by default, and a merged shape may be recognized slightly differently.
- **Added directly**: expand it to uncheck gestures. Unchecked gestures are not imported and stay app-specific.

Click **Import** to finish; the notice offers undo. Two similar gestures in the same app that do different things are never merged.

## Use a gesture in another app

In the gesture details, click **Use in another app…**, choose an app, then choose an action. Apps that already use the gesture can't be picked again; an app that already has the same drawing shows a conflict warning; an app with gestures turned off can be assigned, but the gesture won't run there. **Add app…** at the end of the list adds an app first. Afterwards you stay on the gesture details with the new row highlighted, and **Edit it** in the notice opens that binding.

You can also start from the app. In the editor, a gesture that belongs only to the current app shows **Only in this app**:

- Click **Choose from library** to switch to a library gesture. The picker is searchable; choosing one of the **Gestures in other apps** adds it to the library first and shares it with the original; **Record a new gesture (only in this app)** stays out of the library.
- The **More actions** menu has **Add to library** and **Use in another app…**.

## Edit a shared gesture

In the editor, a gesture from the library shows **Library** and where it is used. When you redraw it, the change applies to every place that uses it, and any new conflicts are listed:

- **Only change this one**: keeps the change for the current app only; it becomes app-specific, and the library and other apps keep the old drawing.
- **Done**: keeps the shared change and closes the notice.

**Change** switches to another library gesture; the ↗ button (**Show in library**) opens the gesture details.

## Create and delete

**New gesture** records a gesture directly in the library. It is added once you finish drawing; the name is optional. Assign it later with **Use in another app…**.

When you choose **Delete gesture** and the gesture is still in use, pick one:

- **Delete; each place keeps its own copy**
- **Delete it and everything using it**

Both can be undone from the notice.

## Configuration and compatibility

The library is stored under `gestures` in the configuration file, and every gesture that uses it also stores a full copy of its drawing. Older versions of Mousetroke can therefore read the new configuration and the gestures keep working. Saving settings in an older version drops the library, though: each gesture becomes app-specific again, and you can organize again after upgrading. If editing the file by hand makes a drawing disagree with the library, that place becomes app-specific and you are told once. See [Configuration, backup and updates](config.md) for file locations and backups.

Next: [Mouse gestures](gestures.md), [Per-app rules](per-app.md), [Action library](action-library.md).
