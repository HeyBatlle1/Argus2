import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "ARGUS — Grok Build 2",
  description:
    "The hundred eyes are open. Grok Build 2 edition — truth-seeking interface for the persistent Argus runtime.",
  icons: {
    icon: "/favicon.ico",
  },
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className="dark">
      <body className="noise-overlay bg-[#0d0d16] text-[#e8e5dc] h-screen overflow-hidden">
        {children}
      </body>
    </html>
  );
}
