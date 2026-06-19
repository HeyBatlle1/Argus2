import type { Metadata } from "next";
import "./globals.css";
import { Toaster } from "sonner";

export const metadata: Metadata = {
  title: "ARGUS NEXUS — The Living Observatory",
  description: "Grok's free-reign vision of Argus Home. A spatial, intuitive interface for the persistent hundred-eyed agent runtime.",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" className="dark">
      <body className="nexus-bg min-h-screen overflow-hidden text-[#e8e8f0]">
        {children}
        <Toaster position="top-center" richColors closeButton />
      </body>
    </html>
  );
}
