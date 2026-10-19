import { initPlatform, primaryModifier, appIdentity } from "./platform";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import type { Binding, Config, EngineStatus, InitState, Scope, Step } from "./types";
import { api, errText } from "./api";
import { conflicts, decompose, fileName, newBinding, uid } from "./gesture";
import { displayName, initLang, t, useLang } from "./i18n";
import { updateBinding, updateScope, useConfigStore } from "./store";
import { Sidebar, scopeTitle, type View } from "./components/Sidebar";
import { BindingList, type ListFilter, type ListTab } from "./components/BindingList";
import { Editor } from "./components/Editor";
import { Prefs } from "./components/Prefs";
import { AppPicker, DataPage, Toast } from "./components/Dialogs";
import { NavIcon } from "./components/Icons";
import { TipLayer } from "./components/Tip";
import { Guide, guideUnseen } from "./components/Guide";
import { About } from "./components/About";

export function App() {
  const [init, setInit] = useState<InitState | null>(null);
  const [fatal, setFatal] = useState<string | null>(null);

  useEffect(() => {
    api.initState().then(
      (s) => {
        initPlatform(s.platform ?? "windows");
        document.documentElement.dataset.platform = s.platform ?? "windows";
        initLang({ setting: s.config.settings.language, systemLangs: s.systemLangs });
        setInit(s);
      },
      (e) => setFatal(errText(e)),
    );
  }, []);

  // 首帧渲染后再显示窗口，避免白屏闪烁
  useEffect(() => {
    if (init || fatal) requestAnimationFrame(() => getCurrentWindow().show());
  }, [init, fatal]);

  if (fatal) return <div className="fatal">{t("ui.app.fatal", { error: fatal })}</div>;
  if (!init) return null;
  return <Main init={init} />;
}

type SaveState = "idle" | "saving" | "saved" | "error" | "incomplete";

/** 尚未录完的笔画留在编辑状态，不能触发自动保存或关闭时写盘。 */
function hasIncompleteStroke(config: Config): boolean {
  return config.scopes.some((s) => s.bindings.some((b) => b.pattern.some((s) => (s.type === "shape" && s.templates.length === 0) || (s.type === "trace" && s.points.length < 4))));
}

interface ToastMsg {
  id: number;
  text: string;
  action?: string;
  onAction?: () => void;
}

function Main({ init }: { init: InitState }) {
  const store = useConfigStore(init.config);
  const { config, edit, undo, redo } = store;
  // 语言订阅放在根组件：切换语言时整棵树重渲染（t() 在渲染期取词条）
  const [lang] = useLang();
  const globalIdx = Math.max(0, config.scopes.findIndex((s) => s.kind === "global"));
  const [view, setView] = useState<View>(init.about ? { kind: "about" } : { kind: "scope", index: globalIdx });
  useEffect(() => {
    const timer = setInterval(() => { api.takeAboutRequest().then((about) => { if (about) setView({ kind: "about" }); }, () => {}); }, 1000);
    return () => clearInterval(timer);
  }, []);
  const [selected, setSelected] = useState<number | null>(null);
  // 分类与筛选跨作用域保留（切到某个程序时仍停在「触发角」）
  const [listTab, setListTab] = useState<ListTab>("gesture");
  const [listFilter, setListFilter] = useState<ListFilter>("all");
  const [engine, setEngine] = useState<EngineStatus>(init.engine);
  const [saveState, setSaveState] = useState<SaveState>("idle");
  const [toast, setToast] = useState<ToastMsg | null>(null);
  const [picking, setPicking] = useState(false);
  const [loadError, setLoadError] = useState(init.loadError);
  const [guide, setGuide] = useState(guideUnseen);

  const notify = useCallback((text: string, action?: string, onAction?: () => void) => setToast({ id: Date.now(), text, action, onAction }), []);

  // ---- 语言变化 → 后端窗口标题与少量原生文字跟着换
  useEffect(() => {
    api.setUiLang(lang).catch(() => {});
  }, [lang]);

  // ---- 自动保存（防抖；引擎约 1 秒内热加载）
  const lastSaved = useRef(store.revision);
  useEffect(() => {
    if (store.revision === lastSaved.current) return;
    if (hasIncompleteStroke(config)) {
      setSaveState("incomplete");
      return;
    }
    setSaveState("saving");
    const t0 = setTimeout(async () => {
      try {
        await api.saveConfig(config);
        lastSaved.current = store.revision;
        setSaveState("saved");
        setLoadError(null);
      } catch (e) {
        setSaveState("error");
        notify(t("ui.app.save_failed", { error: errText(e) }));
      }
    }, 350);
    return () => clearTimeout(t0);
  }, [store.revision, config, notify]);

  // ---- 关闭窗口前（含托盘「退出」）把防抖中尚未写盘的修改存掉
  const pending = useRef({ config, revision: store.revision });
  pending.current = { config, revision: store.revision };
  useEffect(() => {
    const off = getCurrentWindow().onCloseRequested(async () => {
      if (pending.current.revision !== lastSaved.current && !hasIncompleteStroke(pending.current.config)) {
        await api.saveConfig(pending.current.config).catch(() => {});
      }
    });
    return () => void off.then((f) => f());
  }, []);

  // ---- 引擎状态轮询
  useEffect(() => {
    const t0 = setInterval(() => api.engineStatus().then(setEngine, () => {}), 3000);
    return () => clearInterval(t0);
  }, []);

  // ---- 全局快捷键：撤销/重做（文本框内交给原生）、Esc 关闭编辑面板
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const el = e.target as HTMLElement;
      const typing = el.tagName === "INPUT" || el.tagName === "TEXTAREA";
      if (primaryModifier(e) && !typing) {
        const k = e.key.toLowerCase();
        if (k === "z" && !e.shiftKey) (e.preventDefault(), undo());
        else if (k === "y" || (k === "z" && e.shiftKey)) (e.preventDefault(), redo());
      }
      if (e.key === "Escape" && !typing && selected != null) setSelected(null);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [undo, redo, selected]);

  // 撤销可能删掉当前作用域/绑定：纠正越界的选择
  const scopeIdx = view.kind === "scope" ? Math.min(view.index, config.scopes.length - 1) : -1;
  const scope: Scope | undefined = scopeIdx >= 0 ? config.scopes[scopeIdx] : undefined;
  const sel = scope && selected != null && selected < scope.bindings.length ? selected : null;

  const clash = useMemo(() => (scope ? conflicts(scope.bindings) : new Map<number, number[]>()), [scope]);

  // ---- 绑定操作
  const addBinding = (pattern: Step[]) => {
    if (!scope) return;
    const b = { ...newBinding(), pattern, name: pattern[0]?.type === "edge" ? t("ui.app.new_edge") : t("ui.app.new_gesture") };
    edit((c) => updateScope(c, scopeIdx, (s) => ({ ...s, bindings: [...s.bindings, b] })));
    setSelected(scope.bindings.length);
  };
  const deleteBinding = (i: number) => {
    if (!scope) return;
    const name = displayName(scope.bindings[i]?.name, scope.bindings[i]?.name_key);
    edit((c) => updateScope(c, scopeIdx, (s) => ({ ...s, bindings: s.bindings.filter((_, j) => j !== i) })));
    setSelected(null);
    notify(t("ui.app.deleted", { name }), t("common.undo"), () => {
      undo();
      setSelected(i);
    });
  };
  const toggleBinding = (i: number) => {
    if (!scope) return;
    edit((c) => updateBinding(c, scopeIdx, i, (b) => ({ ...b, disabled: b.disabled ? undefined : true })));
  };
  const duplicateBinding = (i: number) => {
    if (!scope) return;
    const src = scope.bindings[i];
    const copy: Binding = { ...structuredClone(src), name: t("ui.app.copy_of", { name: displayName(src.name, src.name_key) }), name_key: undefined };
    edit((c) => updateScope(c, scopeIdx, (s) => ({ ...s, bindings: [...s.bindings.slice(0, i + 1), copy, ...s.bindings.slice(i + 1)] })));
    setSelected(i + 1);
  };

  // ---- 作用域操作
  /**
   * 添加程序（已存在的直接跳过去）；多个时跳到第一个。`disable` = 在这些程序里禁用手势；
   * `names` = 内置清单程序的词条键（preset.app.*），缺省用 exe 文件名。
   */
  const addApps = (exes: string[], disable = false, navigate = true, names: Record<string, string> = {}) => {
    setPicking(false);
    const same = (s: Scope, exe: string) => s.kind === "app" && appIdentity(s.app?.exe ?? "").toLowerCase() === appIdentity(exe).toLowerCase();
    const scopes = [...config.scopes];
    let first = -1;
    for (const exe of exes) {
      let i = scopes.findIndex((s) => same(s, exe));
      if (i < 0) {
        const nameKey = names[exe];
        const fallback = fileName(exe).replace(/\.exe$/i, "");
        const name = nameKey ? displayName(fallback, nameKey) : fallback;
        scopes.push({
          id: uid(),
          name,
          ...(nameKey ? { name_key: nameKey } : {}),
          kind: "app",
          app: { exe: appIdentity(exe), regex: false },
          bindings: [],
          ...(disable ? { disabled: true } : {}),
        });
        i = scopes.length - 1;
      } else if (disable && !scopes[i].disabled) {
        scopes[i] = { ...scopes[i], disabled: true };
      }
      if (first < 0) first = i;
    }
    if (first < 0) return;
    if (scopes.some((s, i) => s !== config.scopes[i])) {
      edit((c) => ({ ...c, scopes: [...c.scopes.map((s, i) => (disable && scopes[i]?.id === s.id ? scopes[i] : s)), ...scopes.slice(c.scopes.length)] }));
    }
    if (!navigate) return;
    setView({ kind: "scope", index: first });
    setSelected(null);
  };
  const addApp = (exe: string, disable = false, nameKey?: string) => addApps([exe], disable, true, nameKey ? { [exe]: nameKey } : {});

  // ---- 把程序或快捷方式拖进窗口 → 添加
  const [dropping, setDropping] = useState(false);
  const addAppsRef = useRef(addApps);
  addAppsRef.current = addApps;
  useEffect(() => {
    const off = getCurrentWebview().onDragDropEvent(async (e) => {
      const p = e.payload;
      if (p.type === "enter" || p.type === "over") setDropping(true);
      else if (p.type === "leave") setDropping(false);
      else if (p.type === "drop") {
        setDropping(false);
        const ok: string[] = [];
        const bad: string[] = [];
        for (const path of p.paths) {
          try {
            ok.push(await api.resolveApp(path));
          } catch (err) {
            bad.push(errText(err));
          }
        }
        if (ok.length) addAppsRef.current(ok);
        if (bad.length) notify(bad.length === 1 ? bad[0] : t("ui.drop.skipped", { count: bad.length }));
        else if (ok.length > 1) notify(t("ui.drop.added", { count: ok.length }));
      }
    });
    return () => void off.then((f) => f());
  }, [notify]);
  const deleteScope = () => {
    if (!scope) return;
    const title = scopeTitle(scope);
    edit((c) => ({ ...c, scopes: c.scopes.filter((_, i) => i !== scopeIdx) }));
    setView({ kind: "scope", index: globalIdx > scopeIdx ? globalIdx - 1 : globalIdx });
    setSelected(null);
    notify(t("ui.app.deleted", { name: title }), t("common.undo"), () => {
      undo();
      setView({ kind: "scope", index: scopeIdx });
    });
  };

  const replaceConfig = (c: Config, message: string) => {
    store.load(c, true);
    setView({ kind: "scope", index: Math.max(0, c.scopes.findIndex((s) => s.kind === "global")) });
    setSelected(null);
    notify(message, t("common.undo"), undo);
  };

  const binding = scope && sel != null ? scope.bindings[sel] : null;
  const otherBindings = useMemo(() => scope?.bindings.filter((_, i) => i !== sel) ?? [], [scope, sel]);

  // 编辑中把触发方式改成边角（或反过来）→ 列表分类跟过去，选中项不会"消失"
  const bindingTab: ListTab | null = binding ? (decompose(binding.pattern).trigger.kind === "edge" ? "edge" : "gesture") : null;
  useEffect(() => {
    if (bindingTab && bindingTab !== listTab) {
      setListTab(bindingTab);
      setListFilter("all");
    }
  }, [bindingTab]); // eslint-disable-line react-hooks/exhaustive-deps
  const conflictNames = sel != null ? (clash.get(sel) ?? []).map((j) => displayName(scope!.bindings[j].name, scope!.bindings[j].name_key)) : [];

  return (
    <div className={`app ${binding ? "with-editor" : ""}`}>
      <Sidebar
        scopes={config.scopes}
        view={view}
        onView={(v) => {
          setView(v);
          setSelected(null);
        }}
        onAddApp={() => setPicking(true)}
        engine={engine}
        onStartEngine={() => api.startEngine().then(setEngine, (e) => notify(errText(e)))}
        saveState={saveState}
      />

      <main className="main">
        {loadError && (
          <div className="banner error">{t("ui.app.load_error", { error: loadError })}</div>
        )}

        {view.kind === "scope" && scope && (
          <BindingList
            key={scope.id}
            scope={scope}
            tab={listTab}
            onTab={(tb) => {
              setListTab(tb);
              setListFilter("all");
            }}
            filter={listFilter}
            onFilter={setListFilter}
            selected={sel}
            onSelect={setSelected}
            onAdd={addBinding}
            onDelete={deleteBinding}
            onDuplicate={duplicateBinding}
            onToggle={toggleBinding}
            onScopeChange={(f, k) => edit((c) => updateScope(c, scopeIdx, f), k)}
            onDeleteScope={deleteScope}
          />
        )}
        {view.kind === "prefs" && (
          <Prefs
            settings={config.settings}
            onChange={(f, k) => edit((c) => ({ ...c, settings: f(c.settings) }), k && `settings-${k}`)}
            engine={engine}
            onAutostart={(on) => api.setAutostart(on).then(setEngine, (e) => notify(errText(e)))}
          />
        )}
        {view.kind === "about" && <About engine={engine} onShowGuide={() => setGuide(true)} notify={(tt) => notify(tt)} />}
        {view.kind === "data" && (
          <DataPage config={config} configPath={init.configPath} onReplace={replaceConfig} notify={(tt) => notify(tt)} />
        )}
      </main>

      {binding && sel != null && (
        <Editor
          binding={binding}
          bindingKey={`${scope!.id}-${sel}`}
          conflictWith={conflictNames}
          otherBindings={otherBindings}
          onChange={(f, k) => edit((c) => updateBinding(c, scopeIdx, sel, f), k && `b-${scopeIdx}-${sel}-${k}`)}
          onDuplicate={() => duplicateBinding(sel)}
          onDelete={() => deleteBinding(sel)}
          onClose={() => setSelected(null)}
        />
      )}

      {picking && (
        <AppPicker
          scopes={config.scopes}
          onPick={addApp}
          onPickMany={(apps) => {
            addApps(apps.map((a) => a.exe), true, true, Object.fromEntries(apps.flatMap((a) => (a.name_key ? [[a.exe, a.name_key]] : []))) as Record<string, string>);
            notify(t("toast.apps_disabled", { count: apps.length }));
          }}
          onClose={() => setPicking(false)}
        />
      )}
      {dropping && (
        <div className="drop-overlay" aria-live="polite">
          <div className="drop-card">
            <NavIcon kind="app" />
            <strong>{t("ui.drop.hint")}</strong>
            <span className="muted">{t("ui.drop.supported")}</span>
          </div>
        </div>
      )}
      {guide && (
        <Guide
          pauseHotkey={config.settings.pause_hotkey}
          onDisableApps={(apps) => {
            addApps(apps.map((a) => a.exe), true, false, Object.fromEntries(apps.flatMap((a) => (a.name_key ? [[a.exe, a.name_key]] : []))) as Record<string, string>);
            notify(t("toast.apps_disabled", { count: apps.length }), t("common.undo"), undo);
          }}
          onClose={() => setGuide(false)}
        />
      )}
      <TipLayer />
      {toast && <Toast key={toast.id} text={toast.text} action={toast.action} onAction={toast.onAction} onDone={() => setToast(null)} />}
    </div>
  );
}
