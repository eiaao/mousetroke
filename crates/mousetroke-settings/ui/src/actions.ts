import type { Action, ActionCatalog, ActionPreset } from "./types";
import { isPresetAction } from "./types";
import { t, type MsgKey } from "./i18n";

let presets = new Map<string, ActionPreset>();
export function initActionCatalog(catalog?: ActionCatalog) {
  presets = new Map(catalog?.presets.map((p) => [p.id, p]));
}
export function presetDefinition(action: Action): ActionPreset | undefined {
  return isPresetAction(action) ? presets.get(action.id) : undefined;
}
export function presetSummary(action: Action): string {
  const preset = presetDefinition(action);
  return t((preset?.name_key ?? "preset.status.unknown") as MsgKey);
}
