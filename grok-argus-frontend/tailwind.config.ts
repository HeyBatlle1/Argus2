import type { Config } from "tailwindcss";

const config: Config = {
  darkMode: "class",
  content: ["./app/**/*.{ts,tsx}", "./components/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        argus: {
          bg: "#0d0d16", bg2: "#12121f", amber: "#f5b800", amberDim: "#c48a20",
          amberBright: "#ffe066", greenLight: "#39d353", red: "#ff4444", text: "#e8e5dc",
          textDim: "#b8b5ac", border: "#32325a", surface: "#16162a", surfaceHi: "#1e1e38",
        },
      },
    },
  },
  plugins: [],
};
export default config;
