import type { Metadata } from "next";
import "@fontsource-variable/dm-sans";
import "@fontsource-variable/manrope";
import "./globals.css";
export const metadata: Metadata = {
  metadataBase: new URL(
    "https://feed-the-pile-sumit.agilemars259.chatgpt.site",
  ),
  title: "Feed the Pile — A little less digital clutter",
  description:
    "Sort Downloads and Screenshots, one file at a time. Keep, Later, or Recycle Bin. A free Windows app with a tiny pixel pal.",
  openGraph: {
    title: "Feed the Pile",
    description: "Big piles. Small bites. A little less digital clutter.",
    images: [{ url: "/og.png", width: 1200, height: 630 }],
  },
  twitter: {
    card: "summary_large_image",
    title: "Feed the Pile",
    description: "One file. One little decision.",
    images: ["/og.png"],
  },
};
export default function Layout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
