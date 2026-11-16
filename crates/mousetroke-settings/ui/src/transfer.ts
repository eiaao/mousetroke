import type { Binding, Config } from "./types";
import { conflicts, uid } from "./gesture";

export type TransferMode = "copy" | "move";
export type ConflictChoice = "skip" | "replace";
export interface TransferSource {
  scope: number;
  binding: number;
}

/** Match the editor's existing, scope-local drawing conflict rule. */
export function transferConflicts(scope: Config["scopes"][number], binding: Binding): number[] {
  return conflicts([...scope.bindings, binding]).get(scope.bindings.length) ?? [];
}

/** Valid destinations after applying each scope's conflict choice. */
export function transferTargets(
  config: Config,
  source: TransferSource,
  destinations: number[],
  choices: Record<number, ConflictChoice>,
): number[] {
  const binding = config.scopes[source.scope]?.bindings[source.binding];
  if (!binding) return [];
  return [...new Set(destinations)].filter((index) =>
    Number.isInteger(index) && index >= 0 && index !== source.scope && !!config.scopes[index]
      && (!transferConflicts(config.scopes[index], binding).length || choices[index] === "replace"),
  );
}

/** Apply all destinations as one undoable config edit. A copy owns its drawing. */
export function transferBinding(
  config: Config,
  source: TransferSource,
  destinations: number[],
  mode: TransferMode,
  choices: Record<number, ConflictChoice>,
): Config {
  const binding = config.scopes[source.scope]?.bindings[source.binding];
  const targets = transferTargets(config, source, destinations, choices);
  if (!binding || !targets.length) return config;
  const scopes = config.scopes.map((scope) => ({ ...scope, bindings: [...scope.bindings] }));
  for (const index of targets) {
    const scope = scopes[index];
    const clashes = transferConflicts(scope, binding);
    if (clashes.length) scope.bindings = scope.bindings.filter((_, i) => !clashes.includes(i));
    // Several destinations must own separate drawings; a one-target move can retain its library link.
    scope.bindings.push({ ...structuredClone(binding), uiId: uid(), gesture: mode === "copy" || targets.length > 1 ? undefined : binding.gesture });
  }
  if (mode === "move") {
    scopes[source.scope].bindings.splice(source.binding, 1);
  }
  return { ...config, scopes };
}
