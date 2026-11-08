import { initPlatform, primaryModifier, appIdentity, isMac } from "./platform";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import type { Action, Binding, Config, EngineStatus, ImportPlan, InitState, Scope, Step } from "./types";
import { api, errText } from "./api";
import { initActionCatalog } from "./actions";
import { actionSummary, conflicts, decompose, fileName, gestureName, keyseqToCaps, newBinding, uid } from "./gesture";
import { displayName, initLang, t, useLang } from "./i18n";
import { updateBinding, updateScope, useConfigStore } from "./store";
import { presetDefinition, presetName } from "./actions";
import {
  addGesture, applyImportPlan, assignGesture, canAdopt, categoryOf, deleteGesture, detach, gestureById, gestureUses, gesturesOf,
  keepChangeHere, link, promote, renameGesture, setBindingPattern, setGesturePattern, type BindingRef, type GroupChoice,
} from "./library";
import { GestureDetail, GestureLibrary, conflictLines, placeText, type LibraryFilter, type LibraryTab } from "./components/GestureLibrary";
import { GesturePicker } from "./components/GesturePicker";
import { ImportWizard } from "./components/ImportWizard";
import { Sidebar, scopeTitle, type View } from "./components/Sidebar";
import { BindingList, type ListFilter, type ListTab } from "./components/BindingList";
import { Editor } from "./components/Editor";
import { Prefs } from "./components/Prefs";
import { AppPicker, DataPage, Toast, type ToastExtra } from "./components/Dialogs";
import { NavIcon } from "./components/Icons";
import { TipLayer } from "./components/Tip";
import { Guide, guideUnseen, permissionsReady } from "./components/Guide";
import { About } from "./components/About";

export function App() {
  const [init, setInit] = useState<InitState | null>(null);
  const [fatal, setFatal] = useState<string | null>(null);

  useEffect(() => {
    api.initState().then(
      (s) => {
        initPlatform(s.platform ?? "windows");
        initActionCatalog(s.actionCatalog);
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

/** 尚未录完的笔画留在编辑状态，不能触发自动保存或关闭时写盘（手势库条目同样检查）。 */
function hasIncompleteStroke(config: Config): boolean {
  const incomplete = (pattern: Step[]) => pattern.some((s) => (s.type === "shape" && s.templates.length === 0) || (s.type === "trace" && s.points.length < 4));
  return config.scopes.some((s) => s.bindings.some((b) => incomplete(b.pattern))) || gesturesOf(config).some((g) => incomplete(g.pattern));
}

const GESTURES_SEEN_KEY = "mousetroke.gestures.seen";
function gesturesSeen(): boolean {
  try {
    return localStorage.getItem(GESTURES_SEEN_KEY) === "1";
  } catch {
    return true;
  }
}
function markGesturesSeen() {
  try {
    localStorage.setItem(GESTURES_SEEN_KEY, "1");
  } catch {
    // 无痕/受限存储：只影响"新"标记
  }
}

interface ToastMsg {
  id: number;
  text: string;
  action?: string;
  onAction?: () => void;
  extra?: ToastExtra;
}

function Main({ init }: { init: InitState }) {
  const store = useConfigStore(init.config);
  const { config, edit, undo, redo } = store;
  // 语言订阅放在根组件：切换语言时整棵树重渲染（t() 在渲染期取词条）
  const [lang] = useLang();
  const globalIdx = Math.max(0, config.scopes.findIndex((s) => s.kind === "global"));
  const [view, setView] = useState<View>(init.about ? { kind: "about" } : { kind: "scope", index: globalIdx });
  // 再次打开设置（如托盘「关于」）时由后端推送，不再轮询
  useEffect(() => {
    const off = listen("show-about", () => {
      api.takeAboutRequest().catch(() => {});
      setView({ kind: "about" });
    });
    return () => void off.then((f) => f());
  }, []);
  const [selected, setSelected] = useState<number | null>(null);
  // 手势库：选中的手势（"new" = 新建）、分类与筛选、弹窗
  const [libSelected, setLibSelected] = useState<string | null>(null);
  const [libTab, setLibTab] = useState<LibraryTab>("gesture");
  const [libFilter, setLibFilter] = useState<LibraryFilter>("all");
  const [seen, setSeen] = useState(gesturesSeen);
  const [wizard, setWizard] = useState(false);
  const [gesturePicker, setGesturePicker] = useState(false);
  const [assignScope, setAssignScope] = useState<number | null>(null);
  // 「在其他程序中使用」：打开该手势详情时直接弹出程序选择
  const [assignFor, setAssignFor] = useState<string | null>(null);
  const [pendingAssign, setPendingAssign] = useState(false);
  const [autoPickKey, setAutoPickKey] = useState<string | null>(null);
  // 分类与筛选跨作用域保留（切到某个程序时仍停在「触发角」）
  const [listTab, setListTab] = useState<ListTab>("gesture");
  const [listFilter, setListFilter] = useState<ListFilter>("all");
  const [engine, setEngine] = useState<EngineStatus>(init.engine);
  const [saveState, setSaveState] = useState<SaveState>("idle");
  const [toast, setToast] = useState<ToastMsg | null>(null);
  /** 刚在「在其他程序中使用」里新建的绑定（手势详情里高亮这一行）。 */
  const [justAssigned, setJustAssigned] = useState<string | null>(null);
  useEffect(() => {
    if (!justAssigned) return;
    const timer = setTimeout(() => setJustAssigned(null), 2500);
    return () => clearTimeout(timer);
  }, [justAssigned]);
  const [picking, setPicking] = useState(false);
  const [loadError, setLoadError] = useState(init.loadError);
  const [recovering, setRecovering] = useState(false);
  const [setup, setSetup] = useState(() => isMac() && (!!init.setup || !!init.permissions && !permissionsReady(init.permissions)));
  const [guide, setGuide] = useState(() => guideUnseen() || (isMac() && (!!init.setup || !!init.permissions && !permissionsReady(init.permissions))));
  const [guideRequest, setGuideRequest] = useState(0);
  const guideOpen = useRef(guide);
  guideOpen.current = guide;
  useEffect(() => {
    if (!isMac()) return;
    // 先订阅再消费暂存请求，覆盖 init_state 返回到 React 订阅之间的启动竞态。
    let disposed = false;
    const openSetup = () => {
      if (disposed) return;
      guideOpen.current = true;
      setSetup(true);
      setGuide(true);
      setGuideRequest((n) => n + 1);
    };
    const consumeSetup = () => {
      if (disposed) return;
      api.takeSetupRequest().then((requested) => { if (requested) openSetup(); }).catch(() => {});
    };
    const off = listen("show-setup", consumeSetup);
    const offSettings = listen("show-settings", () => {
      if (disposed) return;
      api.getPermissions().then((status) => {
        if (!guideOpen.current && !permissionsReady(status)) openSetup();
      }).catch(() => {});
    });
    Promise.all([off, offSettings]).then(consumeSetup).catch(() => {});
    return () => { disposed = true; void off.then((f) => f()); void offSettings.then((f) => f()); };
  }, []);

  const notify = useCallback((text: string, action?: string, onAction?: () => void, extra?: ToastExtra) => setToast({ id: Date.now(), text, action, onAction, extra }), []);

  // ---- 手势库引用的兜底提示：加载时（手改配置）与编辑时（界面漏改）
  useEffect(() => {
    if (init.detachedGestureRefs) notify(t("ui.gestures.detached", { count: init.detachedGestureRefs }));
  }, []); // eslint-disable-line react-hooks/exhaustive-deps
  const lastRepairs = useRef(store.repairs);
  useEffect(() => {
    if (store.repairs > lastRepairs.current) notify(t("ui.gestures.repaired", { count: store.repairs - lastRepairs.current }));
    lastRepairs.current = store.repairs;
  }, [store.repairs, notify]);

  // ---- 语言变化 → 后端窗口标题与少量原生文字跟着换；动作目录的默认值（搜索引擎）也按新语言重取
  useEffect(() => {
    api.setUiLang(lang).catch(() => {});
    api.actionCatalog().then(initActionCatalog).catch(() => {});
  }, [lang]);

  // ---- 自动保存（防抖；引擎约 1 秒内热加载）
  const lastSaved = useRef(store.revision);
  useEffect(() => {
    if (loadError || store.revision === lastSaved.current) return;
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
  }, [store.revision, config, notify, loadError]);

  // ---- 关闭窗口前（含托盘「退出」）把防抖中尚未写盘的修改存掉
  const pending = useRef({ config, revision: store.revision, loadError });
  pending.current = { config, revision: store.revision, loadError };
  useEffect(() => {
    const off = getCurrentWindow().onCloseRequested(async () => {
      if (!pending.current.loadError && pending.current.revision !== lastSaved.current && !hasIncompleteStroke(pending.current.config)) {
        await api.saveConfig(pending.current.config).catch(() => {});
      }
    });
    return () => void off.then((f) => f());
  }, []);

  // ---- 引擎状态轮询
  useEffect(() => {
    // 窗口隐藏时不轮询；状态未变时保持同一对象，避免整棵树每 3 秒重渲染
    const t0 = setInterval(() => {
      if (document.hidden) return;
      api.engineStatus().then(
        (next) => setEngine((prev) => (prev.running === next.running && prev.autostart === next.autostart && prev.exeFound === next.exeFound ? prev : next)),
        () => {},
      );
    }, 3000);
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
      if (e.key === "Escape" && !typing && libSelected != null) setLibSelected(null);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [undo, redo, selected, libSelected]);

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
    setAutoPickKey(b.uiId!);
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
    edit((c) => updateBinding(c, scopeIdx, i, (b) => ({ ...b, disabled: b.draft || !b.disabled ? true : undefined, draft: undefined })));
  };
  const duplicateBinding = (i: number) => {
    if (!scope) return;
    const src = scope.bindings[i];
    // 副本改为本程序专用：复制通常是为了改手势，保留引用会让改副本同时改到原绑定
    const copy: Binding = { ...structuredClone(src), uiId: uid(), gesture: undefined, name: t("ui.app.copy_of", { name: displayName(src.name, src.name_key) }), name_key: undefined };
    edit((c) => updateScope(c, scopeIdx, (s) => ({ ...s, bindings: [...s.bindings.slice(0, i + 1), copy, ...s.bindings.slice(i + 1)] })));
    setSelected(i + 1);
  };

  // ---- 作用域操作
  /**
   * 添加程序（已存在的直接跳过去）；多个时跳到第一个。`disable` = 在这些程序里禁用手势；
   * `names` = 内置清单程序的词条键（preset.app.*），缺省用 exe 文件名。
   */
  const addApps = (exes: string[], disable = false, navigate = true, names: Record<string, string> = {}): number => {
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
    if (first < 0) return -1;
    if (scopes.some((s, i) => s !== config.scopes[i])) {
      edit((c) => ({ ...c, scopes: [...c.scopes.map((s, i) => (disable && scopes[i]?.id === s.id ? scopes[i] : s)), ...scopes.slice(c.scopes.length)] }));
    }
    if (!navigate) return first;
    setView({ kind: "scope", index: first });
    setSelected(null);
    return first;
  };
  const addApp = (exe: string, disable = false, nameKey?: string) => {
    // 「在其他程序中使用 → 添加程序」：加完直接为它选动作，留在手势库页
    if (pendingAssign) {
      setPendingAssign(false);
      const idx = addApps([exe], false, false, nameKey ? { [exe]: nameKey } : {});
      if (idx >= 0) setAssignScope(idx);
      return;
    }
    addApps([exe], disable, true, nameKey ? { [exe]: nameKey } : {});
  };

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

  const replaceConfig = async (c: Config, message: string) => {
    const recovering = !!loadError;
    if (recovering) await api.recoverConfig(c);
    store.load(c, true);
    if (recovering) setLoadError(null);
    setView({ kind: "scope", index: Math.max(0, c.scopes.findIndex((s) => s.kind === "global")) });
    setSelected(null);
    if (recovering) notify(message);
    else notify(message, t("common.undo"), undo);
  };

  const binding = scope && sel != null ? scope.bindings[sel] : null;
  const otherBindings = useMemo(() => scope?.bindings.filter((_, i) => i !== sel) ?? [], [scope, sel]);

  // ---- 手势库
  const bindingRef: BindingRef | null = binding && sel != null ? { scope: scopeIdx, binding: sel } : null;
  const shared = useMemo(() => {
    const g = gestureById(config, binding?.gesture);
    if (!g || sel == null) return undefined;
    const uses = gestureUses(config, g.id);
    const others = uses.filter((u) => !(u.scope === scopeIdx && u.binding === sel));
    return { gesture: g, uses: uses.length, places: others.map((u) => placeText(config, u)), conflicts: conflictLines(config, g.id, scopeIdx) };
  }, [config, binding, scopeIdx, sel]);
  const openView = (v: View) => {
    setView(v);
    setSelected(null);
    if (v.kind === "gestures" && !seen) {
      markGesturesSeen();
      setSeen(true);
    }
  };
  const showLibrary = (id: string | null) => {
    openView({ kind: "gestures" });
    setLibSelected(id);
    const g = gestureById(config, id ?? undefined);
    if (g) {
      setLibTab(categoryOf(g.pattern));
      setLibFilter("all");
    }
  };
  const jumpTo = (ref: BindingRef) => {
    setView({ kind: "scope", index: ref.scope });
    setSelected(ref.binding);
    setLibSelected(null);
  };
  const editPattern = (pattern: Step[]) => bindingRef && edit((c) => setBindingPattern(c, bindingRef, pattern));
  const keepHere = (before: Step[]) => {
    if (!bindingRef) return;
    edit((c) => keepChangeHere(c, bindingRef, before));
    notify(t("ui.gestures.kept_here"), t("common.undo"), undo);
  };
  const pickGesture = (id: string) => {
    if (!bindingRef) return;
    setGesturePicker(false);
    edit((c) => link(c, bindingRef, id));
    notify(t("ui.gestures.picked"), t("common.undo"), undo);
  };
  const pickInline = (source: BindingRef) => {
    if (!bindingRef) return;
    setGesturePicker(false);
    const place = placeText(config, source);
    edit((c) => {
      const r = promote(c, source);
      return r ? link(r.config, bindingRef, r.id) : c;
    });
    notify(t("ui.gestures.picked_shared", { place }), t("common.undo"), undo);
  };
  const recordNew = () => {
    if (!bindingRef) return;
    setGesturePicker(false);
    // 改回本程序专用并清空笔画，录制器随即就绪；未录完前按草稿停用（不会变成单击手势）
    edit((c) => {
      const d = detach(c, bindingRef);
      const b = d.scopes[bindingRef.scope].bindings[bindingRef.binding];
      if (b.pattern[0]?.type === "edge") return d;
      const press = b.pattern.find((s) => s.type === "press") ?? { type: "press", key: "mouse:right" };
      return updateBinding(setBindingPattern(d, bindingRef, [press]), bindingRef.scope, bindingRef.binding, (x) => ({ ...x, draft: true, disabled: true }));
    });
  };
  const promoteBinding = () => {
    if (!bindingRef) return;
    edit((c) => promote(c, bindingRef)?.config ?? c);
    notify(t("ui.gestures.added"));
  };
  const useElsewhere = () => {
    if (!bindingRef) return;
    const r = promote(config, bindingRef);
    if (!r) return;
    edit(() => r.config);
    showLibrary(r.id);
    setAssignFor(r.id);
  };
  const assign = (scopeIndex: number, action: Action) => {
    const id = libSelected;
    setAssignScope(null);
    if (!id || id === "new") return;
    const preset = presetDefinition(action);
    // 预置动作用动作名；高级动作（可能还没填内容）退回默认名，之后在编辑器里补全
    // 发送快捷键以组合键命名（Ctrl+Shift+K），不用原始语法
    const keyName = action.type === "key_sequence" && typeof action.keys === "string" ? keyseqToCaps(action.keys).map((g) => g.join("+")).join(" ") : "";
    const name = (preset ? presetName(preset) : keyName || actionSummary(action)) || t("ui.app.new_gesture");
    const b: Binding = { uiId: uid(), loaded: true, name, lastActionName: preset ? name : undefined, pattern: [], action, execute_on_match: false, hide_label: false, secured: false };
    edit((c) => assignGesture(c, scopeIndex, id, b));
    // 留在手势详情（常会连着分配给几个程序），新的一行高亮；提示条可直接去设置这一条
    setJustAssigned(b.uiId ?? null);
    const target = config.scopes[scopeIndex];
    const open = () => {
      const c = pending.current.config;
      const si = c.scopes.findIndex((s) => s.bindings.some((x) => x.uiId === b.uiId));
      if (si >= 0) jumpTo({ scope: si, binding: c.scopes[si].bindings.findIndex((x) => x.uiId === b.uiId) });
    };
    notify(t("ui.gestures.assigned", { scope: target ? scopeTitle(target) : "", name }), t("common.undo"), undo, { label: t("ui.gestures.go_edit"), run: open });
  };
  const removeGesture = (mode: "detach" | "cascade") => {
    const g = gestureById(config, libSelected ?? undefined);
    if (!g) return;
    const n = gestureUses(config, g.id).length;
    edit((c) => deleteGesture(c, g.id, mode));
    setLibSelected(null);
    const text = !n ? t("ui.gestures.deleted", { name: gestureName(g) }) : mode === "cascade" ? t("ui.gestures.deleted_cascade", { count: n }) : t("ui.gestures.deleted_detach", { count: n });
    notify(text, t("common.undo"), () => {
      undo();
      setLibSelected(g.id);
    });
  };
  const createGesture = (pattern: Step[], name: string) => {
    const r = addGesture(config, pattern, name);
    edit(() => r.config);
    setLibSelected(r.id);
    setLibTab(categoryOf(pattern));
    notify(t("ui.gestures.added"));
  };
  const applyImport = (plan: ImportPlan, choices: GroupChoice[]) => {
    const { stats } = applyImportPlan(config, plan, choices);
    setWizard(false);
    edit((c) => applyImportPlan(c, plan, choices).config);
    showLibrary(null);
    notify(t("ui.gestures.imported", { count: stats.added }), t("common.undo"), undo);
  };

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
    <div className={`app ${binding || (view.kind === "gestures" && libSelected) ? "with-editor" : ""}`}>
      <Sidebar
        scopes={config.scopes}
        view={view}
        onView={openView}
        gestureCount={gesturesOf(config).length}
        gesturesNew={!seen && gesturesOf(config).length === 0}
        onAddApp={() => setPicking(true)}
        engine={engine}
        onStartEngine={() => api.startEngine().then(setEngine, (e) => notify(errText(e)))}
        saveState={saveState}
      />

      <main className="main">
        {loadError && (
          <div className="banner error" role="alert">
            <p>{t("ui.app.load_error", { error: loadError })}</p>
            <div className="action-card-tools">
              <button type="button" className="btn small" onClick={() => api.openPath("config_dir").catch((e) => notify(errText(e)))}>{t("ui.app.open_config_folder")}</button>
              <button type="button" className="btn small" disabled={recovering} onClick={async () => {
                setRecovering(true);
                try {
                  const defaults = await api.recoverConfig();
                  lastSaved.current = store.revision;
                  store.load(defaults, false);
                  setSelected(null); setAutoPickKey(null); setLoadError(null); setSaveState("saved");
                  setView({ kind: "scope", index: Math.max(0, defaults.scopes.findIndex((scope) => scope.kind === "global")) });
                } catch (e) { notify(errText(e)); }
                finally { setRecovering(false); }
              }}>{t("ui.app.recover_defaults")}</button>
            </div>
          </div>
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
            clash={clash}
          />
        )}
        {view.kind === "gestures" && (
          <GestureLibrary
            config={config}
            selected={libSelected}
            onSelect={setLibSelected}
            onNew={() => setLibSelected("new")}
            onImport={() => setWizard(true)}
            tab={libTab}
            onTab={(tb) => {
              setLibTab(tb);
              setLibFilter("all");
            }}
            filter={libFilter}
            onFilter={setLibFilter}
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
          bindingKey={binding.uiId!}
          autoPickAction={autoPickKey === binding.uiId}
          onPickerShown={() => setAutoPickKey((key) => key === binding.uiId ? null : key)}
          conflictWith={conflictNames}
          otherBindings={otherBindings}
          onChange={(f, k) => edit((c) => updateBinding(c, scopeIdx, sel, f), k && `b-${scopeIdx}-${sel}-${k}`)}
          onPatternChange={editPattern}
          shared={shared}
          canAdopt={canAdopt(binding)}
          onOpenPicker={() => setGesturePicker(true)}
          onPromote={promoteBinding}
          onUseElsewhere={useElsewhere}
          onShowInLibrary={(id) => showLibrary(id)}
          onKeepHere={keepHere}
          onDuplicate={() => duplicateBinding(sel)}
          onDelete={() => deleteBinding(sel)}
          onClose={() => setSelected(null)}
        />
      )}

      {view.kind === "gestures" && libSelected && (
        <GestureDetail
          key={libSelected}
          config={config}
          id={libSelected}
          assignScope={assignScope}
          startAssign={assignFor === libSelected}
          onAssignStarted={() => setAssignFor(null)}
          onPattern={(p) => edit((c) => setGesturePattern(c, libSelected, p))}
          onRename={(name) => edit((c) => renameGesture(c, libSelected, name), `gname-${libSelected}`)}
          onCreate={createGesture}
          highlight={justAssigned}
          onJump={jumpTo}
          onAssign={assign}
          onAssignCancel={() => setAssignScope(null)}
          onAssignScope={setAssignScope}
          onAddApp={() => {
            setPendingAssign(true);
            setPicking(true);
          }}
          onDelete={removeGesture}
          onClose={() => setLibSelected(null)}
        />
      )}
      {gesturePicker && bindingRef && (
        <GesturePicker config={config} target={bindingRef} onPickGesture={pickGesture} onPickInline={pickInline} onRecordNew={recordNew} onClose={() => setGesturePicker(false)} />
      )}
      {wizard && <ImportWizard config={config} onApply={applyImport} onClose={() => setWizard(false)} />}
      {picking && (
        <AppPicker
          scopes={config.scopes}
          onPick={addApp}
          onPickMany={(apps) => {
            addApps(apps.map((a) => a.exe), true, true, Object.fromEntries(apps.flatMap((a) => (a.name_key ? [[a.exe, a.name_key]] : []))) as Record<string, string>);
            notify(t("toast.apps_disabled", { count: apps.length }));
          }}
          onClose={() => {
            setPicking(false);
            setPendingAssign(false);
          }}
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
          key={guideRequest}
          setup={setup}
          initialPermissions={setup && guideRequest === 0 ? init.permissions : undefined}
          pauseHotkey={config.settings.pause_hotkey}
          onDisableApps={(apps) => {
            addApps(apps.map((a) => a.exe), true, false, Object.fromEntries(apps.flatMap((a) => (a.name_key ? [[a.exe, a.name_key]] : []))) as Record<string, string>);
            notify(t("toast.apps_disabled", { count: apps.length }), t("common.undo"), undo);
          }}
          onClose={() => { setGuide(false); setSetup(false); }}
        />
      )}
      <TipLayer />
      {toast && <Toast key={toast.id} text={toast.text} action={toast.action} onAction={toast.onAction} extra={toast.extra} onDone={() => setToast(null)} />}
    </div>
  );
}
