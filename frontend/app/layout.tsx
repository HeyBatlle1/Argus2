import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Argus — Secure Agent Runtime. Built in Rust. Ferris stays locked in.",
  description:
    "Quantum-hardened, security-first autonomous AI agent runtime. The hundred eyes are open.",
  icons: {
    icon: "/favicon.svg",
    shortcut: "/favicon.svg",
  },
  openGraph: {
    title: "Argus — Secure Agent Runtime. Built in Rust. Ferris stays locked in.",
    description:
      "Quantum-hardened, security-first autonomous AI agent runtime. The hundred eyes are open.",
    images: [
      {
        url: "https://raw.githubusercontent.com/HeyBatlle1/Argus2/main/assets/logo.svg",
        width: 400,
        height: 400,
        alt: "Argus — Ferris the Rust crab locked behind vault bars",
      },
    ],
  },
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
