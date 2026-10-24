import type { CSSProperties } from "react";
export type Mood = "idle" | "keep" | "later" | "bin" | "copy" | "celebrate";
export function Creature({
  mood = "idle",
  size = 90,
}: {
  mood?: Mood;
  size?: number;
}) {
  return (
    <div
      className={`creature creature-${mood}`}
      style={{ "--creature-size": `${size}px` } as CSSProperties}
      role="img"
      aria-label={
        mood === "celebrate"
          ? "Pixel creature celebrating an empty pile"
          : `Pixel creature, ${mood}`
      }
    >
      <svg
        width={size}
        height={size}
        viewBox="0 0 32 32"
        shapeRendering="crispEdges"
        aria-hidden="true"
      >
        <path
          fill="#173c52"
          d="M6 10h4V6h4v4h6V6h4v4h2v4h2v10h-4v4h-6v-2h-4v2H8v-4H4V14h2z"
        />
        <path fill="#7db9ed" d="M8 12h16v4h2v6h-4v2H10v-2H6v-6h2z" />
        <path fill="#a9d8fc" d="M10 12h12v4H10z" />
        {mood === "celebrate" || mood === "keep" ? (
          <path
            stroke="#173c52"
            strokeWidth="2"
            fill="none"
            d="m10 18 2-2 2 2m4 0 2-2 2 2"
          />
        ) : (
          <path fill="#173c52" d="M10 16h3v4h-3zm9 0h3v4h-3z" />
        )}
        <path fill="#f7ab9a" d="M8 20h3v2H8zm13 0h3v2h-3z" />
        <path
          fill="#173c52"
          d={mood === "later" ? "M15 21h3v1h-3z" : "M14 21h2v1h2v-1h2v2h-6z"}
        />
        {mood === "celebrate" && (
          <path
            fill="#f4b850"
            d="M1 3h2v2H1zm26-2h2v2h-2zM29 8h2v2h-2zM3 29h2v2H3z"
          />
        )}
      </svg>
    </div>
  );
}
