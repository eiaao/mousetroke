---
title: "Send-keys syntax"
description: "Complete Mousetroke send-keys syntax: modifiers, groups, named keys, repeats, delays, mouse input and Windows/Mac shortcuts."
order: 7
---

# Send-keys syntax

## Start with a shortcut

In **Send keys**, paste `^c` to send Ctrl+C on Windows, or `#c` to send Command+C on Mac. You can also click **Record**, press the combination and click **Done**. Lua uses the same syntax through `ms.keys(sequence)`.

Characters normally mean keys at US keyboard positions. `abc` presses A, B, C in sequence; uppercase `ABC` does not add Shift. Use `+a` for Shift+A. For Unicode or fixed text, see [text and escaping](#text-and-escaping) and [clipboard recipes](lua-examples.md).

## Modifiers and groups

| Prefix | Windows | Mac |
|---|---|---|
| `^` | Ctrl | Control ⌃ |
| `%` | Alt | Option ⌥ |
| `+` | Shift | Shift ⇧ |
| `#` | Win | Command ⌘ |

A prefix applies to the next key or parenthesized group. `^+t` sends Ctrl+Shift+T. `^(ac)` applies Ctrl to both A and C. `^ac` applies Ctrl only to A. Groups can nest; use balanced parentheses. Group modifiers attach to each key action, so do not rely on the modifier remaining continuously held between keys. A bare trailing modifier is not a key press: use `{CTRL}`, for example, when you need the modifier key itself.

Lua `ms.is_keys_pressed` checks the union of all keys and modifiers at query time on both platforms: `^ac` requires Control, A and C down together. Either side satisfies a prefix; explicit `{LCTRL}` / `{RCTRL}` names retain their side. Repeats are ignored; sleep, text, mouse or wheel steps return false. It does not recognize a sequence of past presses.

## Named keys, repeats and delays

Use `{ENTER}` for Enter. Names are case-insensitive. `{LEFT 3}` presses Left three times. A positive count repeats a key; zero or a negative count falls back to one. `{SLEEP 100}` waits 100 ms; it requires a nonnegative integer. Separate a name and its argument with a space.

Example: `^c{SLEEP 100}^v` copies, waits briefly, then pastes on Windows. The Mac equivalent is `#c{SLEEP 100}#v`.

The table below enumerates the names accepted by the parser, including aliases and every numbered F/numpad name. A single ASCII letter, digit or supported US punctuation character can also appear inside braces, such as `{a}` or `{+}`.

| Names | Meaning |
|---|---|
| `{ENTER}`, `{RETURN}` | Enter |
| `{ESC}` | Escape |
| `{TAB}` | Tab |
| `{BACKSPACE}` | Backspace |
| `{SPACE}` | Space |
| `{CAPSLOCK}` | Caps Lock |
| `{INSERT}`, `{INS}` | Insert (Windows only) |
| `{DELETE}`, `{DEL}` | Delete / forward delete |
| `{HOME}`, `{END}` | Home / End |
| `{PGUP}`, `{PGDN}` | Page Up / Page Down |
| `{UP}`, `{DOWN}`, `{LEFT}`, `{RIGHT}` | Arrow keys |
| `{PRTSC}` | Print Screen (Windows only) |
| `{BREAK}` | Break (Windows only) |
| `{MEDIA_NEXT}`, `{MEDIA_PREV}`, `{MEDIA_STOP}`, `{MEDIA_PLAY}`, `{MEDIA_PLAY_PAUSE}` | Media keys (Windows only in key sequences) |
| `{VOLUME_MUTE}`, `{VOLUME_DOWN}`, `{VOLUME_UP}` | Volume keys (Windows only in key sequences) |
| `{CTRL}`, `{ALT}`, `{SHIFT}` | Control / Alt (Mac Option) / Shift |
| `{LCTRL}`, `{RCTRL}`, `{LSHIFT}`, `{RSHIFT}`, `{LALT}`, `{RALT}` | Left/right modifier keys; Alt is Option on Mac |
| `{LWIN}`, `{WIN}`, `{RWIN}` | Windows key; Command on Mac |
| `{F1}`, `{F2}`, `{F3}`, `{F4}`, `{F5}`, `{F6}`, `{F7}`, `{F8}`, `{F9}`, `{F10}`, `{F11}`, `{F12}`, `{F13}`, `{F14}`, `{F15}`, `{F16}`, `{F17}`, `{F18}`, `{F19}`, `{F20}`, `{F21}`, `{F22}`, `{F23}`, `{F24}` | Function keys; Mac supports F1–F20 |
| `{N0}`, `{N1}`, `{N2}`, `{N3}`, `{N4}`, `{N5}`, `{N6}`, `{N7}`, `{N8}`, `{N9}` | Numpad digits 0–9 |
| `{MOUSE_LEFT}`, `{MOUSE_RIGHT}`, `{MOUSE_MIDDLE}`, `{MOUSE_X1}`, `{MOUSE_X2}` | Mouse clicks (Windows only) |
| `{MOUSE_WHEEL}`, `{MOUSE_HWHEEL}` | Wheel: e.g. {MOUSE_WHEEL 120}, {MOUSE_HWHEEL -120} (Windows only) |
| `{SLEEP}` | Delay: e.g. {SLEEP 100}, milliseconds |

## Mouse buttons and wheel

Windows supports `{MOUSE_LEFT}`, `{MOUSE_RIGHT}`, `{MOUSE_MIDDLE}`, `{MOUSE_X1}` and `{MOUSE_X2}` as clicks. `{MOUSE_LEFT 2}` sends two clicks; it does not reposition the pointer.

`{MOUSE_WHEEL 120}` scrolls up one notch; `-120` scrolls down. `{MOUSE_HWHEEL 120}` scrolls right; `-120` scrolls left. The argument is a signed wheel delta, not a repeat count. This is the injected-input convention; it differs from the sign used by recorded gesture wheel steps.

Mac parses these tokens but skips mouse and wheel actions during input execution. They cannot be used to synthesize Mac clicks or scrolling through `ms.keys`.

## Text and escaping

Text-first mode is available only with Lua `ms.keys(s, true)`; the Send keys editor always uses key mode. Ordinary characters become Unicode text, but `^`, `%`, `+`, `#`, parentheses and the opening brace still start syntax. Braced single characters remain key actions in text-first mode.

For shortcuts within text-first mode, brace the key: `^{c}` sends Ctrl+C and `#{c}` sends Command+C. An unbraced `^c` in this mode emits the text `c` and discards the modifier. You can also make a separate `ms.keys("^c")` call using normal key mode.

| Literal character | Sequence spelling |
|---|---|
| `^` | `{^}` |
| `%` | `{%}` |
| `+` | `{+}` |
| `#` | `{#}` |
| `(` | `{(}` |
| `)` | `{)}` |
| `{` | `{{}` |
| `}` | `}` (unbraced) |

Backslash is not a key-sequence escape. In a quoted Lua string, write `"\\"` for one backslash, or use a long-bracket string. For arbitrary text containing syntax symbols, the [clipboard method](lua-examples.md#paste-and-restore-clipboard-text) avoids both layers of escaping.

## Common shortcuts

These depend on the destination application's and operating system's shortcut settings.

| Purpose | Windows sequence | Mac sequence |
|---|---|---|
| Copy | `^c` | `#c` |
| Paste | `^v` | `#v` |
| Cut | `^x` | `#x` |
| Select all | `^a` | `#a` |
| Undo | `^z` | `#z` |
| New tab | `^t` | `#t` |
| Close tab | `^w` | `#w` |
| Reopen closed tab | `^+t` | `#+t` |
| Next tab | `^{TAB}` | `^{TAB}` |
| Previous tab | `^+{TAB}` | `^+{TAB}` |
| Switch app | `%{TAB}` | `#{TAB}` |
| Previous/next desktop | `#^{LEFT}` / `#^{RIGHT}` | `^{LEFT}` / `^{RIGHT}` |
| Screenshot region | `#+s` | `#+4` |
| Show desktop | `#d` | `{F11}` |
| Task Manager / Activity Monitor | `^+{ESC}` | Use Open an app: `com.apple.ActivityMonitor` |
| Lock screen | `#l` | `^#q` |

## Why Ctrl+Alt+Del cannot be sent

Ctrl+Alt+Del is Windows' Secure Attention Sequence. Windows ignores ordinary synthetic input for this sequence, so `^%{DEL}` cannot open that security screen. Use `^+{ESC}` for Task Manager, `#l` to lock, or a **Run a command** action containing `shutdown /l` to sign out (which ends the session).

## Limits and platform notes

The shared parser has no overall string-length or repeat-count cap. Repeat counts and delays are stored as unsigned 32-bit values; wheel deltas as signed 32-bit values. Out-of-range values can wrap when converted, so use small practical counts and delays. A bad key name or missing required argument produces a parse error; the editor validates while you type.

Windows executes sequences on worker threads and sleeps for `{SLEEP ms}`. Mac sends input in timer-driven batches. Mac action strings longer than 16,384 bytes are rejected; for already-parsed Lua input, the limits are 16,384 actions and 16,384 bytes per text action. Its waiting queue holds up to 64 entries; excess input is dropped and logged.

On Mac, F1–F20, numpad digits and mapped navigation keys are available. Insert/INS, PRTSC, BREAK, F21–F24, media/volume tokens and mouse/wheel tokens are unsupported and skipped. Use **Media & system** for media and volume. `ms.keys_down` and `ms.keys_up` are Windows-only; see [Lua API](lua-api.md).
