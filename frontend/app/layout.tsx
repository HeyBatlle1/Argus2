import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "ARGUS — The Hundred-Eyed Agent",
  description:
    "Quantum-hardened, security-first autonomous AI agent runtime. The hundred eyes are open.",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className="dark">
      <body className="noise-overlay">{children}</body>
    </html>
  );
}
