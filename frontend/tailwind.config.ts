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
          bg:          "#0d0d16",
          bg2:         "#12121f",
          amber:       "#f5b800",
          amberDim:    "#c48a20",
          amberBright: "#ffe066",
          green:       "#1a4a20",
          greenLight:  "#39d353",
          red:         "#ff4444",
          redDim:      "#8b1a1a",
          text:        "#e8e5dc",
          textDim:     "#b8b5ac",
          textBright:  "#ffffff",
          border:      "#32325a",
          borderBright:"#5a5a8a",
          surface:     "#16162a",
          surfaceHi:   "#1e1e38",
          blue:        "#1a3a5c",
          blueLight:   "#5aafef",
          input:       "#22223a",
          inputBorder: "#4a4a72",
        },
      },
      fontFamily: {
        mono: ["JetBrains Mono", "Fira Code", "monospace"],
        sans: ["Instrument Sans", "Inter", "sans-serif"],
      },
      keyframes: {
        "pulse-soft":     { "0%, 100%": { opacity: "1" }, "50%": { opacity: "0.6" } },
        "pulse-rapid":    { "0%, 100%": { opacity: "1" }, "50%": { opacity: "0.3" } },
        "spin-slow":      { from: { transform: "rotate(0deg)" }, to: { transform: "rotate(360deg)" } },
        "flash-complete": { "0%": { opacity: "0", filter: "brightness(3)" }, "30%": { opacity: "1", filter: "brightness(2)" }, "100%": { opacity: "0.9", filter: "brightness(1)" } },
        "glow-amber":     { "0%, 100%": { boxShadow: "0 0 6px rgba(240,165,0,0.4)" }, "50%": { boxShadow: "0 0 16px rgba(240,165,0,0.8)" } },
        "glow-green":     { "0%, 100%": { boxShadow: "0 0 4px rgba(57,211,83,0.4)" }, "50%": { boxShadow: "0 0 12px rgba(57,211,83,0.7)" } },
        "glow-blue":      { "0%, 100%": { boxShadow: "0 0 4px rgba(90,175,239,0.3)" }, "50%": { boxShadow: "0 0 12px rgba(90,175,239,0.7)" } },
        "fade-in":        { from: { opacity: "0", transform: "translateY(4px)" }, to: { opacity: "1", transform: "translateY(0)" } },
      },
      animation: {
        "pulse-soft":     "pulse-soft 3s ease-in-out infinite",
        "pulse-rapid":    "pulse-rapid 0.8s ease-in-out infinite",
        "spin-slow":      "spin-slow 3s linear infinite",
        "flash-complete": "flash-complete 0.5s ease-out forwards",
        "glow-amber":     "glow-amber 3s ease-in-out infinite",
        "glow-green":     "glow-green 2s ease-in-out infinite",
        "glow-blue":      "glow-blue 0.8s ease-in-out infinite",
        "fade-in":        "fade-in 0.2s ease-out forwards",
      },
    },
  },
  plugins: [],
};

export default config;
