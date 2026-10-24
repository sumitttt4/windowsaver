import React, { useCallback, useEffect, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Creature, type Mood } from "../shared/Creature";
import { isShortcutTarget, type Pile, type Decision } from "../shared/queue";
import { FilePreview } from "./Preview";
import type { Snapshot } from "./types";
import "./style.css";
function shortPath(path: string | null) {
  return path?.replace(/^\\\\\?\\/, "") || "Not chosen yet";
}
function size(bytes: number) {
  return bytes < 1024
    ? `${bytes} B`
    : bytes < 1048576
      ? `${(bytes / 1024).toFixed(0)} KB`
      : `${(bytes / 1048576).toFixed(1)} MB`;
}
function age(time: number) {
  const days = Math.floor((Date.now() / 1000 - time) / 86400);
  return days === 0 ? "Today" : days === 1 ? "Yesterday" : `${days} days old`;
}
function App() {
  const [data, setData] = useState<Snapshot | null>(null),
    [pile, setPile] = useState<Pile>("downloads");
  const [busy, setBusy] = useState(false),
    [loading, setLoading] = useState(true),
    [error, setError] = useState("");
  const [settings, setSettings] = useState(false),
    [history, setHistory] = useState(false),
    [mood, setMood] = useState<Mood>("idle"),
    [notice, setNotice] = useState("");
  const [suggestions, setSuggestions] = useState<Record<
    string,
    unknown
  > | null>(null);
  const lock = useRef(false),
    timer = useRef<ReturnType<typeof setTimeout> | null>(null),
    noticeTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const native = isTauri();
  const card = data?.[pile][0];
  const ready = !!(
    data?.config.downloads &&
    data.config.screenshots &&
    data.config.keep
  );
  const reactTo = useCallback((next: Mood, message: string) => {
    setMood(next);
    setNotice(message);
    if (noticeTimer.current) clearTimeout(noticeTimer.current);
    noticeTimer.current = setTimeout(() => {
      setMood("idle");
      setNotice("");
    }, 2200);
  }, []);
  const refresh = useCallback(async () => {
    if (lock.current || !native) return;
    lock.current = true;
    try {
      const next = await invoke<Snapshot>("snapshot");
      setData(next);
      setError("");
    } catch (e) {
      setError(String(e));
    } finally {
      lock.current = false;
      setLoading(false);
    }
  }, [native]);
  useEffect(() => {
    if (native) void refresh();
    else setLoading(false);
    return () => {
      if (timer.current) clearTimeout(timer.current);
      if (noticeTimer.current) clearTimeout(noticeTimer.current);
    };
  }, [refresh, native]);
  useEffect(() => {
    if (!native) return;
    let disposed = false;
    let cleanup: (() => void) | undefined;
    listen<boolean>("pile-changed", () => {
      if (timer.current) clearTimeout(timer.current);
      timer.current = setTimeout(() => void refresh(), 450);
    })
      .then((fn) => {
        if (disposed) fn();
        else cleanup = fn;
      })
      .catch(() => {});
    // Watcher events are hints; reconcile periodically only while the initial load succeeded.
    const tick = setInterval(() => {
      if (document.visibilityState === "visible" && data && !error)
        void refresh();
    }, 30000);
    const focus = () => {
      if (data && !error) void refresh();
    };
    window.addEventListener("focus", focus);
    return () => {
      disposed = true;
      cleanup?.();
      clearInterval(tick);
      window.removeEventListener("focus", focus);
    };
  }, [native, refresh, !!data, error]);
  async function action(kind: Decision | "undo" | "copy" | "open" | "recover") {
    if (
      lock.current ||
      !native ||
      (!card && !["undo", "recover"].includes(kind))
    )
      return;
    lock.current = true;
    setBusy(true);
    setError("");
    try {
      const args = card ? { pile, path: card.path, id: card.id } : {};
      if (kind === "copy") {
        await invoke("copy_image", args);
        reactTo("copy", "Copied. Paste it wherever you need it.");
      } else if (kind === "open") {
        await invoke("open_file", args);
      } else {
        const next = await invoke<Snapshot>(
          kind === "undo" ? "undo" : kind === "recover" ? "recover" : "decide",
          { ...args, kind },
        );
        setData(next);
        if (kind === "undo") reactTo("idle", "Last decision undone.");
        else if (kind === "recover")
          reactTo("idle", "Recovery check finished.");
        else if (!next[pile].length)
          reactTo("celebrate", "A lighter pile. Nicely done.");
        else
          reactTo(
            kind,
            kind === "keep"
              ? "Some things are worth keeping."
              : kind === "later"
                ? "Not now is a decision too."
                : "Safely in the Recycle Bin.",
          );
      }
    } catch (e) {
      setError(String(e));
    } finally {
      lock.current = false;
      setBusy(false);
    }
  }
  const actionRef = useRef(action);
  actionRef.current = action;
  useEffect(() => {
    function key(e: KeyboardEvent) {
      if (
        e.repeat ||
        e.metaKey ||
        e.ctrlKey ||
        e.altKey ||
        isShortcutTarget(e.target) ||
        settings ||
        history
      )
        return;
      const k = e.key.toLowerCase();
      const actions: Record<string, Decision | "undo" | "copy" | "open"> = {
        k: "keep",
        l: "later",
        b: "bin",
        z: "undo",
        c: "copy",
        " ": "open",
      };
      const command = actions[k];
      if (!command || (command === "copy" && pile !== "screenshots")) return;
      e.preventDefault();
      void actionRef.current(command);
    }
    window.addEventListener("keydown", key);
    return () => window.removeEventListener("keydown", key);
  }, [settings, history, pile]);
  async function choose(kind: string) {
    if (lock.current) return;
    lock.current = true;
    setBusy(true);
    setError("");
    try {
      const next = await invoke<Snapshot | null>("pick_folder", { kind });
      if (next) setData(next);
    } catch (e) {
      setError(String(e));
    } finally {
      lock.current = false;
      setBusy(false);
    }
  }
  async function suggest() {
    try {
      setSuggestions(await invoke("folder_suggestions"));
    } catch (e) {
      setError(String(e));
    }
  }
  const done =
    data?.history.filter((o) => o.status === "done" && o.kind !== "later")
      .length || 0;
  const review = data?.history.some((o) =>
    ["review", "applying", "undoing"].includes(o.status),
  );
  return (
    <div className="desktop-shell">
      <header className="app-header">
        <a className="wordmark" href="#main">
          <span className="pile-mark">▤</span> feed the pile
          <span className="beta-tag">BETA</span>
        </a>
        <div className="header-right">
          <span className="local-badge">
            <i /> Only on your computer
          </span>
          <button
            className="icon-button"
            onClick={() => setHistory(!history)}
            aria-expanded={history}
          >
            History <span>↶</span>
          </button>
          <button
            className="icon-button"
            onClick={() => setSettings(!settings)}
            aria-expanded={settings}
          >
            Folders <span>⚙</span>
          </button>
        </div>
      </header>
      <main id="main">
        {!native ? (
          <section className="setup-panel">
            <Creature size={110} />
            <h1>This pile lives on Windows.</h1>
            <p>
              You’re viewing the desktop interface in a browser. Local files
              stay inaccessible here.
            </p>
            <p>
              On Windows, run <code>npm ci</code>, then{" "}
              <code>npm run tauri dev</code>.
            </p>
            <a className="soft-link" href="http://localhost:3000">
              Open the landing demo ↗
            </a>
          </section>
        ) : loading ? (
          <div className="setup-panel" role="status">
            Opening your local piles…
          </div>
        ) : null}
        {error && (
          <div className="error-banner" role="alert">
            <span>{error}</span>
            <button disabled={busy} onClick={() => void refresh()}>
              Refresh once
            </button>
          </div>
        )}
        {data?.warning && (
          <div className="error-banner" role="alert">
            {data.warning}
            <button disabled={busy} onClick={() => void refresh()}>
              Retry scan
            </button>
          </div>
        )}
        {review && (
          <div className="error-banner">
            An interrupted operation needs review. No more files will move until
            it’s resolved.
            <button onClick={() => setHistory(true)}>Review history</button>
          </div>
        )}
        {native && data && (!ready || settings) && (
          <section className="setup-panel folder-panel">
            <div>
              <span className="eyebrow">YOUR FOLDERS, YOUR RULES</span>
              <h1>
                {ready ? "A place for everything." : "Give your piles a home."}
              </h1>
              <p>
                Choose two piles and a separate Keep folder. Nothing moves until
                you decide.
              </p>
            </div>
            <div className="folder-options">
              {(["downloads", "screenshots", "keep"] as const).map((kind) => (
                <div className="folder-option" key={kind}>
                  <div>
                    <strong>
                      {kind === "keep"
                        ? "Keep destination"
                        : kind === "downloads"
                          ? "Downloads"
                          : "Screenshots"}
                    </strong>
                    <span title={data.config[kind] || ""}>
                      {shortPath(data.config[kind])}
                    </span>
                  </div>
                  <button disabled={busy} onClick={() => void choose(kind)}>
                    {data.config[kind] ? "Change" : "Choose folder"} ↗
                  </button>
                </div>
              ))}
            </div>
            <div className="folder-footer">
              <button
                className="text-button"
                disabled={busy}
                onClick={() => void suggest()}
              >
                Check Windows folder suggestions
              </button>
              {ready && (
                <button className="primary" onClick={() => setSettings(false)}>
                  Back to my piles →
                </button>
              )}
            </div>
            {suggestions && (
              <div className="suggestions">
                <p>
                  Suggestions only. Take a fresh screenshot and confirm where it
                  lands.
                </p>
                <p>
                  Downloads: {String(suggestions.downloads || "Unavailable")}
                </p>
                <p>
                  Screenshots:{" "}
                  {String(
                    suggestions.screenshotsKnownFolder ||
                      suggestions.picturesFallback ||
                      "Unavailable",
                  )}
                </p>
              </div>
            )}
          </section>
        )}
        {native && data && ready && !settings && (
          <>
            <div className="workspace-heading">
              <div>
                <span className="eyebrow">A LITTLE LESS DIGITAL CLUTTER</span>
                <h1>One file. One little decision.</h1>
              </div>
              <span className="sorted-count">{done} filed away</span>
            </div>
            <div className="workspace">
              <div className="sorting-area">
                <div
                  className="pile-tabs"
                  role="tablist"
                  aria-label="Your piles"
                >
                  {(["downloads", "screenshots"] as const).map((tab) => (
                    <button
                      role="tab"
                      aria-selected={pile === tab}
                      key={tab}
                      onClick={() => {
                        setPile(tab);
                        setError("");
                      }}
                    >
                      {tab === "downloads" ? "↓ Downloads" : "▧ Screenshots"}{" "}
                      <span>{data[tab].length}</span>
                    </button>
                  ))}
                </div>
                {card ? (
                  <>
                    <div className="card-stack">
                      <article className="file-card" key={card.id}>
                        <div className="card-topline">
                          <span>
                            {pile === "screenshots"
                              ? "A NOTE TO YOURSELF"
                              : "FROM THE DOWNLOADS DRAWER"}
                          </span>
                          <span>{age(card.modified)}</span>
                        </div>
                        <FilePreview card={card} pile={pile} />
                        <div className="file-caption">
                          <div>
                            <h2 title={card.name}>{card.name}</h2>
                            <p>
                              {size(card.size)} <span>·</span>{" "}
                              {card.extension.toUpperCase() || "FILE"}
                            </p>
                          </div>
                          <button
                            className="open-button"
                            disabled={busy}
                            onClick={() => void action("open")}
                            aria-label={`Open ${card.name}`}
                          >
                            ↗
                          </button>
                        </div>
                      </article>
                    </div>
                    {pile === "screenshots" && (
                      <button
                        className="copy-button"
                        disabled={busy}
                        onClick={() => void action("copy")}
                      >
                        ▣ Copy image <kbd>C</kbd>
                        <span>Use it once. Then decide.</span>
                      </button>
                    )}
                    <div className="decision-buttons">
                      <button
                        className="keep"
                        disabled={busy || !!review}
                        onClick={() => void action("keep")}
                      >
                        <span>↗</span>Keep <kbd>K</kbd>
                      </button>
                      <button
                        className="later"
                        disabled={busy || !!review}
                        onClick={() => void action("later")}
                      >
                        <span>◷</span>Later <kbd>L</kbd>
                      </button>
                      <button
                        className="bin"
                        disabled={busy || !!review}
                        onClick={() => void action("bin")}
                      >
                        <span>⌫</span>Bin <kbd>B</kbd>
                      </button>
                    </div>
                    <p
                      className="keep-destination"
                      title={data.config.keep || ""}
                    >
                      Keep goes to{" "}
                      <strong>{shortPath(data.config.keep)}</strong>
                    </p>
                  </>
                ) : (
                  <div className="empty-pile">
                    <Creature
                      mood={data.warning ? "later" : "celebrate"}
                      size={144}
                    />
                    <h2>
                      {data.warning
                        ? "This pile is unavailable."
                        : "Room to breathe."}
                    </h2>
                    <p>
                      {data.warning
                        ? "Resolve the folder warning above to scan again."
                        : data.skipped
                          ? "No ready files here. Some items are excluded or still settling."
                          : "This pile is empty. Your creature approves."}
                    </p>
                    <button
                      className="primary"
                      onClick={() =>
                        setPile(
                          pile === "downloads" ? "screenshots" : "downloads",
                        )
                      }
                    >
                      Check the other pile →
                    </button>
                  </div>
                )}
                <div className="queue-footer">
                  <button
                    disabled={
                      busy || !data.history.some((o) => o.status === "done")
                    }
                    onClick={() => void action("undo")}
                  >
                    ↶ Undo last <kbd>Z</kbd>
                  </button>
                  <span>
                    <kbd>Space</kbd> open file
                  </span>
                </div>
              </div>
              <aside className="creature-corner">
                <div className="speech-bubble" aria-live="polite">
                  {notice || "Big piles. Small bites. We’ve got this."}
                </div>
                <Creature mood={mood} size={150} />
                <span>
                  Meet Munch.
                  <br />
                  Your tiny pile pal.
                </span>
                <div className="gentle-note">
                  <span>✓</span>
                  <p>
                    Never auto-deleted.
                    <br />
                    Always your decision.
                  </p>
                </div>
              </aside>
            </div>
          </>
        )}
        {history && data && (
          <section className="history-panel" aria-label="Decision history">
            <div className="section-heading">
              <div>
                <span className="eyebrow">LOCAL UNDO JOURNAL</span>
                <h2>Every decision leaves a trail.</h2>
              </div>
              <button
                onClick={() => setHistory(false)}
                aria-label="Close history"
              >
                ✕
              </button>
            </div>
            <p>
              Undo works newest first and never overwrites. Files changed
              elsewhere or removed from the Recycle Bin may need your help.
            </p>
            <button
              className="primary"
              disabled={busy}
              onClick={() => void action("recover")}
            >
              Retry recovery check
            </button>
            <div className="history-list">
              {!data.history.length ? (
                <p>No decisions yet.</p>
              ) : (
                data.history.map((o) => (
                  <article key={o.id}>
                    <div>
                      <strong>{o.kind.toUpperCase()}</strong>
                      <span className={`status status-${o.status}`}>
                        {o.status}
                      </span>
                    </div>
                    <p>{shortPath(o.source)}</p>
                    {o.destination && <p>→ {shortPath(o.destination)}</p>}
                    {o.message && <p className="history-error">{o.message}</p>}
                  </article>
                ))
              )}
            </div>
          </section>
        )}
      </main>
      <footer className="app-footer">
        <span>No cloud. No AI. Just a lighter folder.</span>
        <span>
          {data?.skipped
            ? `${data.skipped} folders, incomplete, linked, or unavailable items excluded`
            : "Made for the stuff you meant to deal with."}
        </span>
      </footer>
    </div>
  );
}
createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
