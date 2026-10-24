import type { CSSProperties, ReactNode } from "react";
type Name =
  | "pile"
  | "download"
  | "image"
  | "keep"
  | "clock"
  | "bin"
  | "copy"
  | "undo"
  | "arrow"
  | "close"
  | "check"
  | "plus"
  | "refresh"
  | "shield"
  | "local"
  | "cursor"
  | "windows"
  | "spark";
const paths: Record<Exclude<Name, "windows">, ReactNode> = {
  pile: (
    <>
      <path d="m4 7 8-4 8 4-8 4z" />
      <path d="m4 12 8 4 8-4M4 17l8 4 8-4" />
    </>
  ),
  download: (
    <>
      <path d="M12 3v12m-5-5 5 5 5-5M4 17v4h16v-4" />
    </>
  ),
  image: (
    <>
      <rect x="3" y="3" width="18" height="18" rx="3" />
      <circle cx="8" cy="8" r="1" />
      <path d="m3 16 5-5 4 4 3-3 6 6" />
    </>
  ),
  keep: (
    <>
      <path d="M8 4h12v12M20 4 9 15M4 8v12h12" />
    </>
  ),
  clock: (
    <>
      <circle cx="12" cy="12" r="9" />
      <path d="M12 7v5l3 2" />
    </>
  ),
  bin: (
    <>
      <path d="M3 6h18M9 6V3h6v3M6 6l1 15h10l1-15M10 10v7m4-7v7" />
    </>
  ),
  copy: (
    <>
      <rect x="8" y="8" width="13" height="13" rx="2" />
      <path d="M16 8V3H3v13h5" />
    </>
  ),
  undo: (
    <>
      <path d="M8 4 3 9l5 5M3 9h10a7 7 0 0 1 0 14" />
    </>
  ),
  arrow: (
    <>
      <path d="M5 12h14m-6-6 6 6-6 6" />
    </>
  ),
  close: <path d="m6 6 12 12M6 18 18 6" />,
  check: <path d="m5 12 4 4L19 6" />,
  plus: <path d="M12 5v14M5 12h14" />,
  refresh: (
    <>
      <path d="M20 7v5h-5M4 17v-5h5" />
      <path d="M6.1 6.1A8 8 0 0 1 20 12M4 12a8 8 0 0 0 13.9 5.9" />
    </>
  ),
  shield: (
    <>
      <path d="m12 3 8 3v6c0 5-8 9-8 9s-8-4-8-9V6z" />
      <path d="m8 12 3 3 5-6" />
    </>
  ),
  local: (
    <>
      <rect x="4" y="4" width="16" height="12" rx="2" />
      <path d="M8 21h8m-4-5v5" />
    </>
  ),
  cursor: <path d="m5 3 15 9-7 2-3 7z" />,
  spark: (
    <>
      <path d="m12 2 2.5 7.5L22 12l-7.5 2.5L12 22l-2.5-7.5L2 12l7.5-2.5z" />
    </>
  ),
};
export function Icon({
  name,
  size = 18,
  className = "",
  style,
}: {
  name: Name;
  size?: number;
  className?: string;
  style?: CSSProperties;
}) {
  return (
    <svg
      className={`ui-icon ${className}`}
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill={name === "windows" ? "currentColor" : "none"}
      stroke={name === "windows" ? "none" : "currentColor"}
      strokeWidth="1.7"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      focusable="false"
      style={style}
    >
      {name === "windows" ? (
        <path d="M3 3h8v8H3zm10 0h8v8h-8zM3 13h8v8H3zm10 0h8v8h-8z" />
      ) : (
        paths[name]
      )}
    </svg>
  );
}
