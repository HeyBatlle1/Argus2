import type { Config } from "tailwindcss";

const config: Config = {
  darkMode: "class",
  content: [
    "./app/**/*.{ts,tsx}",
    "./components/**/*.{ts,tsx}",
    "./hooks/**/*.{ts,tsx}",
    "./lib/**/*.{ts,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        argus: {
          bg: "#0a0a0f",
          bg2: "#0d0d14",
          amber: "#c9a84c",
          amberDim: "#8a6f2e",
          green: "#2d5016",
          greenLight: "#4a7c59",
          red: "#8b1a1a",
          text: "#d4d0c8",
          textDim: "#8a877f",
          border: "#1a1a2e",
          surface: "#111118",
          blue: "#1a3a5c",
          blueLight: "#4a8fc4",
        },
      },
      fontFamily: {
        mono: ["JetBrains Mono", "Fira Code", "monospace"],
        sans: ["Instrument Sans", "Inter", "sans-serif"],
      },
      keyframes: {
        "pulse-soft": {
          "0%, 100%": { opacity: "1" },
          "50%": { opacity: "0.6" },
        },
        "pulse-rapid": {
          "0%, 100%": { opacity: "1" },
          "50%": { opacity: "0.3" },
        },
        "spin-slow": {
          from: { transform: "rotate(0deg)" },
          to: { transform: "rotate(360deg)" },
        },
        "flash-complete": {
          "0%": { opacity: "0", filter: "brightness(3)" },
          "30%": { opacity: "1", filter: "brightness(2)" },
          "100%": { opacity: "0.9", filter: "brightness(1)" },
        },
        "glow-amber": {
          "0%, 100%": { boxShadow: "0 0 4px rgba(201,168,76,0.3)" },
          "50%": { boxShadow: "0 0 12px rgba(201,168,76,0.7)" },
        },
        "glow-blue": {
          "0%, 100%": { boxShadow: "0 0 4px rgba(74,143,196,0.3)" },
          "50%": { boxShadow: "0 0 12px rgba(74,143,196,0.7)" },
        },
        "fade-in": {
          from: { opacity: "0", transform: "translateY(4px)" },
          to: { opacity: "1", transform: "translateY(0)" },
        },
      },
      animation: {
        "pulse-soft": "pulse-soft 3s ease-in-out infinite",
        "pulse-rapid": "pulse-rapid 0.8s ease-in-out infinite",
        "spin-slow": "spin-slow 3s linear infinite",
        "flash-complete": "flash-complete 0.5s ease-out forwards",
        "glow-amber": "glow-amber 3s ease-in-out infinite",
        "glow-blue": "glow-blue 0.8s ease-in-out infinite",
        "fade-in": "fade-in 0.2s ease-out forwards",
      },
    },
  },
  plugins: [],
};

export default config;
