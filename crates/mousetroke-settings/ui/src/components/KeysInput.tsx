import * as React from "react";
import { useEffect, useRef, useState } from "react";
import { api } from "../api";
import { comboToKeyseq, keyseqToCaps } from "../gesture";
import { t } from "../i18n";

export function Caps({ keys }: { keys: string }) {
  const groups = keyseqToCaps(keys);
  if (!groups.length) return null;
  return (
    <span className="caps">
      {groups.map((g, i) => (
        <span key={i} className="cap-group">
          {g.map((k, j) => (
            <kbd key={j}>{k}</kbd>
          ))}
        </span>
      ))}
    </span>
  );
}

/**
 * 按键序列输入。两种方式：
 * - 点「录制」后直接按组合键（可连续录多组；点「完成」或点别处结束）
 * - 直接编辑语法文本（^Ctrl %Alt +Shift #Win，{F5} {SLEEP 100} 等），实时校验
 */
export function KeysInput({ value, onChange, single }: { value: string; onChange: (v: string) => void; single?: boolean }) {
  const [recording, setRecording] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const btn = useRef<HTMLButtonElement>(null);
  /** 本次录制是否已记下至少一组（决定追加还是覆盖原值）。 */
  const recordingStarted = useRef(false);

  useEffect(() => {
    if (!value) {
      setError(null);
      return;
    }
    const t = setTimeout(() => api.validateKeys(value).then(setError), 200);
    return () => clearTimeout(t);
  }, [value]);

  const onKey = (e: React.KeyboardEvent) => {
    if (!recording) return;
    e.preventDefault();
    e.stopPropagation();
    const k = comboToKeyseq(e);
    if (!k) return;
    if (single) {
      onChange(k);
      setRecording(false);
    } else {
      onChange((value && recordingStarted.current ? value : "") + k);
      recordingStarted.current = true;
    }
  };
  const toggle = () => {
    recordingStarted.current = false;
    setRecording((r) => !r);
    btn.current?.focus(); // 按键事件发到按钮上
  };

  return (
    <div className="keys-input">
      <div className={`keys-row ${error ? "invalid" : ""}`}>
        {recording ? (
          <div className="keys-recording" aria-live="polite">
            {value && recordingStarted.current ? <Caps keys={value} /> : <span className="muted">{t("ui.keys.recording_hint")}</span>}
          </div>
        ) : (
          <input
            className="mono-input"
            value={value}
            spellCheck={false}
            placeholder={single ? t("ui.keys.ph_single") : t("ui.keys.ph")}
            onChange={(e) => onChange(e.target.value)}
            aria-invalid={!!error}
          />
        )}
        <button
          ref={btn}
          type="button"
          className={`btn small ${recording ? "rec" : ""}`}
          onClick={toggle}
          onKeyDown={onKey}
          onKeyUp={(e) => recording && e.preventDefault()}
          onBlur={() => setRecording(false)}
        >
          {recording ? t("ui.keys.done") : t("ui.keys.record")}
        </button>
      </div>
      {!recording && value && !error && (
        <div className="keys-preview">
          <Caps keys={value} />
        </div>
      )}
      {error && <div className="field-error">{error}</div>}
    </div>
  );
}
