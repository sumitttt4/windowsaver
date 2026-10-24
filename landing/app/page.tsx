"use client";
import { Icon } from "../components/Icon";
import { useEffect, useRef, useState } from "react";
import { Creature, type Mood } from "../../shared/Creature";
import {
  rotate,
  isShortcutTarget,
  type Decision,
  type Pile,
} from "../../shared/queue";
type DemoFile = {
  id: string;
  name: string;
  size: string;
  type: string;
  kind: "photo" | "pdf" | "zip" | "screenshot";
};
const seed: Record<Pile, DemoFile[]> = {
  downloads: [
    {
      id: "d1",
      name: "somewhere-i-want-to-go.jpg",
      size: "2.4 MB",
      type: "JPG",
      kind: "photo",
    },
    {
      id: "d2",
      name: "final-final-actually-final.pdf",
      size: "846 KB",
      type: "PDF",
      kind: "pdf",
    },
    {
      id: "d3",
      name: "new-folder-(7).zip",
      size: "12.8 MB",
      type: "ZIP",
      kind: "zip",
    },
  ],
  screenshots: [
    {
      id: "s1",
      name: "Screenshot 2026-10-24.png",
      size: "384 KB",
      type: "PNG",
      kind: "screenshot",
    },
    {
      id: "s2",
      name: "that-one-recipe.png",
      size: "692 KB",
      type: "PNG",
      kind: "screenshot",
    },
    {
      id: "s3",
      name: "remember-this-later.png",
      size: "218 KB",
      type: "PNG",
      kind: "screenshot",
    },
  ],
};
function FileArt({ kind }: { kind: DemoFile["kind"] }) {
  return (
    <div
      className={`demo-art art-${kind}`}
      aria-label={
        kind === "photo"
          ? "Photograph of Moraine Lake at sunrise by ConfinedRiley"
          : kind === "screenshot"
            ? "Illustrated screenshot"
            : `Sample ${kind} file`
      }
      role="img"
    >
      {kind === "photo" ? (
        <img
          src="/moraine-lake.jpg"
          width="1400"
          height="934"
          alt="Moraine Lake at sunrise, with turquoise water and rugged mountain peaks. Photo by ConfinedRiley on Unsplash."
          decoding="async"
        />
      ) : kind === "pdf" ? (
        <div className="pdf-sheet">
          <span>VERY IMPORTANT DOCUMENT</span>
          <strong>
            A plan.
            <br />
            Probably.
          </strong>
          <i />
          <i />
          <i />
          <div className="pdf-stamp">FINAL v7</div>
        </div>
      ) : kind === "zip" ? (
        <div className="zip-art">
          <span>zip</span>
          <i>
            ▦<br />▦<br />▦
          </i>
        </div>
      ) : (
        <div className="screenshot-art">
          <div className="mini-browser">
            <i />
            <i />
            <i />
          </div>
          <span>A little note to future me.</span>
          <div className="note-line" />
          <div className="note-line short" />
          <div className="note-star">
            <Icon name="spark" size={27} />
          </div>
        </div>
      )}
    </div>
  );
}
function Demo() {
  const [pile, setPile] = useState<Pile>("downloads");
  const [queues, setQueues] = useState(seed);
  const [mood, setMood] = useState<Mood>("idle");
  const [message, setMessage] = useState("Big piles. Small bites.");
  const [undo, setUndo] = useState<{
    queues: Record<Pile, DemoFile[]>;
    pile: Pile;
  } | null>(null);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const card = queues[pile][0];
  function react(m: Mood, text: string) {
    if (timer.current) clearTimeout(timer.current);
    setMood(m);
    setMessage(text);
    timer.current = setTimeout(() => {
      setMood("idle");
      setMessage("I could get used to this.");
    }, 2100);
  }
  function decide(action: Decision) {
    if (!card) return;
    const next = {
      ...queues,
      [pile]: action === "later" ? rotate(queues[pile]) : queues[pile].slice(1),
    };
    setUndo({ queues, pile });
    setQueues(next);
    react(
      !next[pile].length ? "celebrate" : action,
      !next[pile].length
        ? "Look at all that breathing room."
        : action === "keep"
          ? "A keeper. Good call."
          : action === "later"
            ? "Back of the queue. Still here."
            : "Nom. Into the pretend Recycle Bin.",
    );
  }
  function restore() {
    if (undo) {
      setQueues(undo.queues);
      setPile(undo.pile);
      setUndo(null);
      react("idle", "No harm. No foul.");
    }
  }
  async function copy() {
    if (!card) return;
    try {
      const canvas = document.createElement("canvas");
      canvas.width = 600;
      canvas.height = 360;
      const ctx = canvas.getContext("2d")!;
      ctx.fillStyle = "#f4efd9";
      ctx.fillRect(0, 0, 600, 360);
      ctx.fillStyle = "#284c62";
      ctx.font = "28px sans-serif";
      ctx.fillText("A little note to future me.", 45, 110);
      ctx.font = "18px sans-serif";
      ctx.fillText("Feed the Pile — sample screenshot", 45, 160);
      const blob = await new Promise<Blob>((resolve, reject) =>
        canvas.toBlob(
          (b) => (b ? resolve(b) : reject(new Error("Image unavailable"))),
          "image/png",
        ),
      );
      if (!navigator.clipboard?.write || typeof ClipboardItem === "undefined")
        throw new Error("Clipboard unavailable");
      await navigator.clipboard.write([
        new ClipboardItem({ "image/png": blob }),
      ]);
      react("copy", "Sample image copied. Now decide.");
    } catch {
      react("later", "Clipboard blocked here. The desktop app copies locally.");
    }
  }
  useEffect(
    () => () => {
      if (timer.current) clearTimeout(timer.current);
    },
    [],
  );
  function key(e: React.KeyboardEvent) {
    if (
      e.repeat ||
      e.ctrlKey ||
      e.metaKey ||
      e.altKey ||
      isShortcutTarget(e.target)
    )
      return;
    const k = e.key.toLowerCase();
    if (k === "k" || k === "l" || k === "b") {
      e.preventDefault();
      decide(k === "k" ? "keep" : k === "l" ? "later" : "bin");
    } else if (k === "z") {
      e.preventDefault();
      restore();
    } else if (k === "c" && pile === "screenshots") {
      e.preventDefault();
      void copy();
    }
  }
  return (
    <div className="demo-wrapper" id="demo">
      <div
        className="demo-app"
        tabIndex={0}
        onKeyDown={key}
        aria-label="Interactive sample-file demo. Focus here to use K, L, B, Z and C."
      >
        <div className="demo-titlebar">
          <span className="window-dots">
            <i />
            <i />
            <i />
          </span>
          <span>feed the pile</span>
          <span className="demo-sample-label">SAMPLE FILES</span>
        </div>
        <div className="demo-inside">
          <div className="demo-main">
            <div className="demo-tabs" role="tablist" aria-label="Demo piles">
              {(["downloads", "screenshots"] as const).map((tab) => (
                <button
                  role="tab"
                  aria-selected={pile === tab}
                  key={tab}
                  onClick={() => setPile(tab)}
                >
                  <Icon
                    name={tab === "downloads" ? "download" : "image"}
                    size={16}
                  />
                  {tab === "downloads" ? "Downloads" : "Screenshots"}
                  <span>{queues[tab].length}</span>
                </button>
              ))}
            </div>
            {card ? (
              <>
                <div className="demo-stack">
                  <article className="demo-file" key={card.id}>
                    <div className="demo-file-top">
                      <span>{card.type} FILE</span>
                      <span>3 days old</span>
                    </div>
                    <FileArt kind={card.kind} />
                    <div className="demo-file-caption">
                      <strong>{card.name}</strong>
                      <span>
                        {card.size} · {card.type}
                      </span>
                    </div>
                  </article>
                </div>
                {pile === "screenshots" && (
                  <button className="demo-copy" onClick={() => void copy()}>
                    <Icon name="copy" size={16} /> Copy sample image{" "}
                    <kbd>C</kbd>
                  </button>
                )}
                <div className="demo-actions">
                  <button
                    className="action-keep"
                    onClick={() => decide("keep")}
                  >
                    <Icon name="keep" size={16} /> Keep <kbd>K</kbd>
                  </button>
                  <button onClick={() => decide("later")}>
                    <Icon name="clock" size={16} /> Later <kbd>L</kbd>
                  </button>
                  <button className="action-bin" onClick={() => decide("bin")}>
                    <Icon name="bin" size={16} /> Bin <kbd>B</kbd>
                  </button>
                </div>
              </>
            ) : (
              <div className="demo-empty">
                <Creature mood="celebrate" size={110} />
                <strong>Room to breathe.</strong>
                <span>That’s one lighter pile.</span>
                <button
                  onClick={() => {
                    setQueues(seed);
                    setUndo(null);
                    react("idle", "A fresh little pile.");
                  }}
                >
                  Try it again <Icon name="refresh" size={16} />
                </button>
              </div>
            )}
            <div className="demo-bottom">
              <button disabled={!undo} onClick={restore}>
                <Icon name="undo" size={15} /> Undo <kbd>Z</kbd>
              </button>
              <span>Nothing here touches your files.</span>
            </div>
          </div>
          <aside className="demo-pal">
            <div className="demo-speech" aria-live="polite">
              {message}
            </div>
            <Creature mood={mood} size={122} />
            <span>munch, your pile pal</span>
          </aside>
        </div>
      </div>
      <div className="demo-caption">
        <span>↑ Go on. Feed the pile.</span>
        <span>Click a choice, or focus the demo and use your keyboard.</span>
      </div>
    </div>
  );
}
const faq = [
  [
    "What does it touch?",
    "Only regular files directly inside the Downloads and Screenshots folders you choose. Keep moves them to a separate folder you approve. It does not scan subfolders. Linked, hidden, incomplete, and cloud-placeholder files are excluded.",
  ],
  [
    "Can it permanently delete my files?",
    "Bin uses the Windows Recycle Bin. There is no permanent-delete or automatic-delete feature. Undo never overwrites an existing file. If you empty the Recycle Bin or change a file elsewhere, recovery may no longer be possible.",
  ],
  [
    "Does anything leave my computer?",
    "The desktop app has no accounts, AI, analytics, cloud service, or file uploads. Its folder settings and undo journal are stored locally. Copy puts an image on your Windows clipboard; your own clipboard history or sync settings still apply.",
  ],
  [
    "Why closed source if it’s free?",
    "Free is the price. Closed source is the distribution choice. The app is a small independently built tool; its source is not offered under an open-source license.",
  ],
  [
    "Which Windows versions?",
    "The beta targets Windows 10 and Windows 11 on x64 PCs, using Microsoft Edge WebView2. Real-device compatibility testing is still pending. ARM and older Windows versions are not part of this beta.",
  ],
  [
    "What happens when I choose Later?",
    "The card returns at the end of its pile. It stays in the same folder and still counts as pending. “Later” doesn’t quietly become “delete.”",
  ],
  [
    "Where do screenshots live?",
    "That depends on how you take them. Windows may use Pictures/Screenshots, a redirected folder, or just the clipboard. Choose the folder your screenshots actually land in.",
  ],
];
export default function Page() {
  const [downloadNote, setDownloadNote] = useState(false);
  const paragraph = useRef<HTMLParagraphElement>(null);
  const downloadTrigger = useRef<HTMLElement | null>(null);
  useEffect(() => {
    const node = paragraph.current;
    if (!node) return;
    const reveal = () => {
      const top = node.getBoundingClientRect().top;
      const progress = Math.max(
        0,
        Math.min(
          1,
          (window.innerHeight * 0.88 - top) / (window.innerHeight * 0.48),
        ),
      );
      node.style.setProperty("--reveal", String(progress));
      node
        .querySelectorAll<HTMLElement>(".reveal-word")
        .forEach(
          (word, i) =>
            (word.style.opacity = String(progress > i / 32 ? 1 : 0.18)),
        );
    };
    reveal();
    window.addEventListener("scroll", reveal, { passive: true });
    window.addEventListener("resize", reveal);
    return () => {
      window.removeEventListener("scroll", reveal);
      window.removeEventListener("resize", reveal);
    };
  }, []);
  useEffect(() => {
    if (!downloadNote) return;
    const previous = downloadTrigger.current;
    const previousOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setDownloadNote(false);
        return;
      }
      if (e.key === "Tab") {
        const items = Array.from(
          document.querySelectorAll<HTMLButtonElement>(
            ".download-dialog button",
          ),
        );
        const first = items[0],
          last = items[items.length - 1];
        if (e.shiftKey && document.activeElement === first) {
          e.preventDefault();
          last?.focus();
        } else if (!e.shiftKey && document.activeElement === last) {
          e.preventDefault();
          first?.focus();
        }
      }
    };
    document.addEventListener("keydown", handler);
    return () => {
      document.body.style.overflow = previousOverflow;
      document.removeEventListener("keydown", handler);
      previous?.focus({ preventScroll: true });
    };
  }, [downloadNote]);
  function download() {
    downloadTrigger.current = document.activeElement as HTMLElement | null;
    setDownloadNote(true);
  }
  return (
    <>
      <header className="site-header">
        <a className="site-logo" href="#">
          <Icon name="pile" size={27} /> feed the pile
        </a>
        <nav aria-label="Main navigation">
          <a href="#how">How it works</a>
          <a href="#pricing">The price</a>
          <a href="#questions">Questions</a>
        </nav>
        <button className="nav-download" onClick={download}>
          Windows beta <Icon name="arrow" size={17} />
        </button>
      </header>
      <main>
        <section className="hero">
          <div className="hero-note">
            <span /> A SMALL WINDOWS APP FOR YOUR VERY LARGE PILES
          </div>
          <h1>
            Your Downloads folder
            <br />
            is a drawer you <em>never open.</em>
          </h1>
          <p className="hero-copy">
            And your screenshots? A thousand little “I’ll need this.”
            <br />
            Make one small decision at a time. Your tiny pixel pal will help.
          </p>
          <div className="hero-ctas">
            <button className="download-button" onClick={download}>
              <Icon name="windows" size={17} /> Download for Windows{" "}
              <Icon name="download" size={17} />
            </button>
            <span>Free. Local. A little lighter.</span>
          </div>
          <Demo />
          <div className="hero-footnote">
            <span>
              <Icon name="shield" size={15} /> No auto-delete
            </span>
            <span>
              <Icon name="local" size={15} /> No AI or cloud
            </span>
            <span>
              <Icon name="cursor" size={15} /> Always your call
            </span>
          </div>
        </section>
        <section
          className="story-section"
          aria-label="The forgotten folder problem"
        >
          <span className="section-kicker">YOU KNOW THE PILE.</span>
          <p ref={paragraph}>
            {"Your Downloads folder is a drawer you never open."
              .split(" ")
              .map((word, i) => (
                <span className="reveal-word" key={`a${i}`}>
                  {word}{" "}
                </span>
              ))}
            <span className="inline-pixel" aria-hidden="true">
              <Icon name="pile" size={30} />
            </span>
            {" Your screenshots are notes you forgot to send yourself."
              .split(" ")
              .map((word, i) => (
                <span className="reveal-word" key={`b${i}`}>
                  {word}{" "}
                </span>
              ))}
            <span className="inline-pixel peach" aria-hidden="true">
              <Icon name="image" size={30} />
            </span>
          </p>
          <div className="story-under">
            <span>A LITTLE CLUTTER ADDS UP.</span>
            <span>Clearing it doesn’t have to be a whole thing.</span>
          </div>
        </section>
        <section className="process-section" id="how">
          <div className="section-intro">
            <span className="section-kicker">
              LESS ORGANIZING. MORE DECIDING.
            </span>
            <h2>
              Not a productivity system.
              <br />
              Just a smaller pile.
            </h2>
            <p>Two folders. Three choices. One very supportive creature.</p>
          </div>
          <div className="process-grid">
            <article>
              <span className="process-number">01 / THE PILE GROWS</span>
              <div className="process-visual grow">
                <span className="little-file lf1">pdf</span>
                <span className="little-file lf2">png</span>
                <span className="little-file lf3">zip</span>
                <span className="little-file lf4">jpg</span>
              </div>
              <h3>Life happens. Files happen.</h3>
              <p>
                Downloads, receipts, that screenshot of that thing. We’ve all
                got a pile.
              </p>
            </article>
            <article>
              <span className="process-number">02 / YOU MAKE THE CALL</span>
              <div className="process-visual choose">
                <div className="little-file">png</div>
                <div className="little-choices">
                  <Icon name="arrow" size={17} />
                  <Icon name="clock" size={17} />
                  <Icon name="bin" size={17} />
                </div>
              </div>
              <h3>Keep. Later. Bin.</h3>
              <p>
                One file at a time. Copy a screenshot first if you need it. No
                filing philosophy required.
              </p>
            </article>
            <article>
              <span className="process-number">
                03 / A LITTLE ROOM TO BREATHE
              </span>
              <div className="process-visual smile">
                <Creature mood="keep" size={110} />
                <span className="celebration-spark">
                  <Icon name="spark" size={23} />
                </span>
              </div>
              <h3>You feel lighter. Munch agrees.</h3>
              <p>
                A smaller pile and a small celebration. Sometimes that’s plenty.
              </p>
            </article>
          </div>
        </section>
        <section className="toggle-strip" aria-label="You remain in control">
          <div>
            <span>Silent auto-delete</span>
            <span className="display-switch off" aria-hidden="true">
              <i />
            </span>
            <strong>OFF</strong>
          </div>
          <span className="strip-cross">
            <Icon name="close" size={14} />
          </span>
          <div>
            <span>Your decision</span>
            <span className="display-switch on" aria-hidden="true">
              <i />
            </span>
            <strong>ON</strong>
          </div>
        </section>
        <section className="pricing-section" id="pricing">
          <div className="section-intro">
            <span className="section-kicker">
              A SMALL APP. AN EVEN SMALLER PRICE.
            </span>
            <h2>
              Less in your folders.
              <br />
              Nothing from your wallet.
            </h2>
            <p>No subscription to cancel. That would just be another pile.</p>
          </div>
          <div className="pricing-grid">
            <article className="price-card">
              <div className="price-top">
                <span>THE WHOLE LITTLE APP</span>
                <span className="price-tag">ACTUALLY FREE</span>
              </div>
              <strong className="price">
                $0<span>/ forever, for this version</span>
              </strong>
              <p>
                Both piles. Undo. The creature.
                <br />
                Everything you came here for.
              </p>
              <ul>
                <li>
                  <Icon name="check" size={15} />
                  Downloads + Screenshots
                </li>
                <li>
                  <Icon name="check" size={15} />
                  Copy, keep, later, and Recycle Bin
                </li>
                <li>
                  <Icon name="check" size={15} />A local undo journal
                </li>
                <li>
                  <Icon name="check" size={15} />A pixel pal with no business
                  model
                </li>
              </ul>
              <button className="download-button" onClick={download}>
                Download for Windows <Icon name="download" size={17} />
              </button>
              <span className="price-small">Free software. Closed source.</span>
            </article>
            <article className="price-card joke-card">
              <div className="price-top">
                <span>THE ABSURDLY PREMIUM PLAN</span>
                <span className="joke-star">
                  <Icon name="spark" size={23} />
                </span>
              </div>
              <strong className="price joke-price">$0000000</strong>
              <p>
                Same app. Plus the feeling
                <br />
                of having paid a lot.
              </p>
              <div className="joke-perks">
                <span>
                  <Icon name="check" size={15} /> Absolutely no extra features
                </span>
                <span>
                  <Icon name="check" size={15} /> The exact same creature
                </span>
                <span>
                  <Icon name="check" size={15} /> Unmatched imaginary value
                </span>
              </div>
              <button className="joke-button" onClick={download}>
                Pay nothing, again <Icon name="arrow" size={17} />
              </button>
              <span className="price-small">
                A joke tier. No payment collected.
              </span>
            </article>
          </div>
        </section>
        <section className="faq-section" id="questions">
          <div>
            <span className="section-kicker">
              A FEW THINGS, BEFORE YOU FEED.
            </span>
            <h2>Good questions.</h2>
            <Creature size={95} />
          </div>
          <div className="faq-list">
            {faq.map(([q, a]) => (
              <details key={q}>
                <summary>
                  {q}
                  <Icon name="plus" size={18} />
                </summary>
                <p>{a}</p>
              </details>
            ))}
          </div>
        </section>
        <section className="closing">
          <Creature mood="keep" size={110} />
          <span className="section-kicker">
            THE PILE CAN WAIT. BUT IT DOESN’T HAVE TO.
          </span>
          <h2>One file is a fine start.</h2>
          <button className="download-button" onClick={download}>
            Download for Windows <Icon name="download" size={17} />
          </button>
          <p>Free. Windows. Entirely on your side.</p>
        </section>
      </main>
      <footer className="site-footer">
        <a className="site-logo" href="#">
          <Icon name="pile" size={27} /> feed the pile
        </a>
        <span>A tiny tool by Sumit Kumar Sharma.</span>
        <span>© {new Date().getFullYear()} · Closed source.</span>
      </footer>
      {downloadNote && (
        <div className="modal-backdrop" onClick={() => setDownloadNote(false)}>
          <section
            className="download-dialog"
            role="dialog"
            aria-modal="true"
            aria-labelledby="download-title"
            onClick={(e) => e.stopPropagation()}
          >
            <button
              autoFocus
              className="dialog-close"
              onClick={() => setDownloadNote(false)}
              aria-label="Close download notice"
            >
              <Icon name="close" size={20} />
            </button>
            <Creature size={100} />
            <h2 id="download-title">A little more baking.</h2>
            <p>
              The Windows installer is not published yet. The sorting app is
              built in source, with real-device safety checks still to finish.
              There’s no pretend download here.
            </p>
            <p>
              For now, try the sample-file demo. It touches none of your files.
            </p>
            <button
              className="download-button"
              onClick={() => {
                setDownloadNote(false);
                document
                  .getElementById("demo")
                  ?.scrollIntoView({ behavior: "smooth" });
              }}
            >
              Try the little demo <Icon name="arrow" size={17} />
            </button>
          </section>
        </div>
      )}
    </>
  );
}
