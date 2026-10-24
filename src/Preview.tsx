import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Card } from "./types";
import type { Pile } from "../shared/queue";
const cache = new Map<string, string>();
async function renderPdf(base64: string, signal: AbortSignal): Promise<string> {
  if (signal.aborted) throw new Error("Preview cancelled");
  const pdfjs = await import("pdfjs-dist");
  const worker = await import("pdfjs-dist/build/pdf.worker.min.mjs?url");
  pdfjs.GlobalWorkerOptions.workerSrc = worker.default;
  const bytes = Uint8Array.from(atob(base64), (x) => x.charCodeAt(0));
  const task = pdfjs.getDocument({
    data: bytes,
    useSystemFonts: true,
    disableFontFace: true,
  });
  const cancel = () => {
    void task.destroy();
  };
  signal.addEventListener("abort", cancel, { once: true });
  if (signal.aborted) cancel();
  try {
    const doc = await task.promise;
    const page = await doc.getPage(1);
    const original = page.getViewport({ scale: 1 });
    const viewport = page.getViewport({
      scale: Math.min(720 / original.width, 520 / original.height, 2),
    });
    const canvas = document.createElement("canvas");
    canvas.width = viewport.width;
    canvas.height = viewport.height;
    await page.render({ canvas, viewport }).promise;
    return canvas.toDataURL("image/png");
  } finally {
    signal.removeEventListener("abort", cancel);
    await task.destroy();
  }
}
export function FilePreview({ card, pile }: { card: Card; pile: Pile }) {
  const [preview, setPreview] = useState<{
    id: string;
    url?: string;
    error?: string;
  }>({ id: card.id });
  useEffect(() => {
    let active = true;
    const abort = new AbortController();
    const hit = cache.get(card.id);
    if (hit) {
      setPreview({ id: card.id, url: hit });
      return;
    }
    setPreview({ id: card.id });
    invoke<{ kind: string; data: string }>("preview", {
      pile,
      path: card.path,
      id: card.id,
    })
      .then(async (p) => {
        if (!active) return;
        const url =
          p.kind === "pdf"
            ? await renderPdf(p.data, abort.signal)
            : `data:image/png;base64,${p.data}`;
        if (!active) return;
        cache.set(card.id, url);
        if (cache.size > 16) cache.delete(cache.keys().next().value!);
        setPreview({ id: card.id, url });
      })
      .catch((error) => {
        if (active) setPreview({ id: card.id, error: String(error) });
      });
    return () => {
      active = false;
      abort.abort();
    };
  }, [card.id, card.path, pile]);
  const current = preview.id === card.id ? preview : undefined;
  return (
    <div className="file-preview">
      {current?.url ? (
        <img src={current.url} alt={`Preview of ${card.name}`} />
      ) : (
        <div className="file-fallback">
          <span className="file-type">{card.extension || "FILE"}</span>
          <span>{current?.error || "Making a little preview…"}</span>
        </div>
      )}
    </div>
  );
}
