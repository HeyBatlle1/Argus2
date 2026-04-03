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
          bg:          "#080810",
          bg2:         "#0c0c18",
          amber:       "#f0a500",
          amberDim:    "#a87020",
          amberBright: "#ffcc44",
          green:       "#1a4a20",
          greenLight:  "#39d353",
          red:         "#ff4444",
          redDim:      "#8b1a1a",
          text:        "#f0ede4",
          textDim:     "#9d9a91",
          textBright:  "#ffffff",
          border:      "#2a2a44",
          borderBright:"#4a4a6a",
          surface:     "#12121e",
          surfaceHi:   "#1a1a2e",
          blue:        "#1a3a5c",
          blueLight:   "#5aafef",
          input:       "#1e1e30",
          inputBorder: "#3a3a5a",
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
