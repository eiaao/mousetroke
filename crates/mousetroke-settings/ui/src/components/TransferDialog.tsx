import { useState } from "react";
import type { Config } from "../types";
import { displayName, t } from "../i18n";
import { transferConflicts, transferTargets, type ConflictChoice, type TransferMode, type TransferSource } from "../transfer";
import { Modal } from "./Dialogs";
import { scopeTitle } from "./Sidebar";

export function TransferDialog({ config, source, onApply, onAddApp, onClose }: {
  config: Config;
  source: TransferSource;
  /** 目标与冲突处理都按作用域 id 传出：对话框打开期间撤销会改变下标。 */
  onApply: (mode: TransferMode, destinationIds: string[], choiceIds: Record<string, ConflictChoice>) => void;
  onAddApp: () => void;
  onClose: () => void;
}) {
  const binding = config.scopes[source.scope].bindings[source.binding];
  const [selected, setSelected] = useState<string[]>([]);
  const [choices, setChoices] = useState<Record<string, ConflictChoice>>({});
  const destinations = config.scopes.map((scope, index) => ({ scope, index })).filter(({ index }) => index !== source.scope);
  const toggle = (id: string) => setSelected((before) => before.includes(id) ? before.filter((i) => i !== id) : [...before, id]);
  const indexChoices: Record<number, ConflictChoice> = {};
  for (const { scope, index } of destinations) if (choices[scope.id]) indexChoices[index] = choices[scope.id];
  const selectedIndexes = destinations.filter(({ scope }) => selected.includes(scope.id)).map(({ index }) => index);
  const ready = transferTargets(config, source, selectedIndexes, indexChoices).length > 0;
  return (
    <Modal title={t("ui.transfer.title")} onClose={onClose}>
      <div className="modal-body">
        <p className="field-help transfer-summary">
          {t("ui.transfer.from", { name: displayName(binding.name, binding.name_key), scope: scopeTitle(config.scopes[source.scope]) })}
          <br />
          {t("ui.transfer.move_help", { scope: scopeTitle(config.scopes[source.scope]) })}
        </p>
        <p className="field-help">{t("ui.transfer.destinations")}</p>
        <div className="transfer-destinations">
          {destinations.map(({ scope }) => {
            const clashes = transferConflicts(scope, binding);
            const checked = selected.includes(scope.id);
            return (
              <div key={scope.id} className="transfer-destination">
                <label className="check">
                  <input type="checkbox" checked={checked} onChange={() => toggle(scope.id)} />
                  <span className="transfer-destination-name">{scopeTitle(scope)}</span>
                  {scope.disabled && <small className="transfer-destination-off">{t("ui.transfer.off")}</small>}
                </label>
                {checked && clashes.length > 0 && (
                  <div className="transfer-conflict">
                    <span>{t("ui.transfer.conflict", { names: clashes.map((i) => displayName(scope.bindings[i].name, scope.bindings[i].name_key)).join(", ") })}</span>
                    <select value={choices[scope.id] ?? "skip"} aria-label={t("ui.transfer.conflict_action", { scope: scopeTitle(scope) })} onChange={(e) => setChoices((before) => ({ ...before, [scope.id]: e.target.value as ConflictChoice }))}>
                      <option value="skip">{t("ui.transfer.skip")}</option>
                      <option value="replace">{t("ui.transfer.replace")}</option>
                    </select>
                  </div>
                )}
              </div>
            );
          })}
        </div>
        <button type="button" className="link transfer-add-app" onClick={onAddApp}>{t("ui.sidebar.add_app")}</button>
      </div>
      <footer className="modal-foot btn-row">
        <button type="button" className="btn primary" disabled={!ready} onClick={() => onApply("copy", selected, choices)}>{t("ui.transfer.copy")}</button>
        <button type="button" className="btn primary" disabled={!ready} onClick={() => onApply("move", selected, choices)}>{t("ui.transfer.move")}</button>
        <button type="button" className="btn" onClick={onClose}>{t("common.cancel")}</button>
      </footer>
    </Modal>
  );
}
