export type Card = {
  id: string;
  path: string;
  name: string;
  size: number;
  modified: number;
  extension: string;
};
export type Operation = {
  id: number;
  kind: "keep" | "later" | "bin";
  source: string;
  destination: string | null;
  hash: string;
  status: string;
  message: string;
};
export type Snapshot = {
  config: {
    downloads: string | null;
    screenshots: string | null;
    keep: string | null;
  };
  downloads: Card[];
  screenshots: Card[];
  history: Operation[];
  skipped: number;
  warning: string | null;
};
