import type { Config } from "tailwindcss";

export default {
  darkMode: "class",
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        ciab: {
          copper: "#C4693D",
          "copper-light": "#D4845E",
          "copper-dark": "#A85530",
          "copper-glow": "#E8A878",
          "steel-blue": "#5B8CA8",
          "steel-blue-light": "#6FA0BE",
          silver: "#9EAAB8",
          bg: {
            primary: "#09090B",
            secondary: "#111114",
            card: "#18181B",
            hover: "#222225",
            elevated: "#27272A",
          },
          text: {
            primary: "#EAEAED",
            secondary: "#A1A1AA",
            muted: "#52525B",
          },
          border: {
            DEFAULT: "#27272A",
            light: "#3F3F46",
            focus: "#C4693D",
          },
        },
        state: {
          running: "#22C55E",
          paused: "#EAB308",
          failed: "#EF4444",
          stopped: "#71717A",
          creating: "#3B82F6",
          pending: "#A78BFA",
        },
        provider: {
          claude: "#D97757",
          openai: "#10A37F",
          gemini: "#4285F4",
          cursor: "#7C3AED",
        },
      },
      fontFamily: {
        sans: ["DM Sans", "-apple-system", "BlinkMacSystemFont", "sans-serif"],
        mono: ["IBM Plex Mono", "JetBrains Mono", "monospace"],
        display: ["Instrument Serif", "Georgia", "serif"],
      },
      borderRadius: {
        lg: "0.5rem",
        md: "0.375rem",
        sm: "0.25rem",
      },
      animation: {
        "pulse-slow": "pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite",
        "fade-in": "fadeIn 0.15s ease-out",
        "slide-up": "slideUp 0.2s ease-out",
        "slide-down": "slideDown 0.15s ease-out",
        "scale-in": "scaleIn 0.15s ease-out",
        "glow-pulse": "glowPulse 2s ease-in-out infinite",
        "shimmer": "shimmer 2s linear infinite",
        "slide-in-right": "slideInRight 0.2s ease-out",
      },
      keyframes: {
        fadeIn: {
          "0%": { opacity: "0" },
          "100%": { opacity: "1" },
        },
        slideUp: {
          "0%": { opacity: "0", transform: "translateY(8px)" },
          "100%": { opacity: "1", transform: "translateY(0)" },
        },
        slideDown: {
          "0%": { opacity: "0", transform: "translateY(-4px)" },
          "100%": { opacity: "1", transform: "translateY(0)" },
        },
        scaleIn: {
          "0%": { opacity: "0", transform: "scale(0.95)" },
          "100%": { opacity: "1", transform: "scale(1)" },
        },
        glowPulse: {
          "0%, 100%": { boxShadow: "0 0 0 0 rgba(196, 105, 61, 0)" },
          "50%": { boxShadow: "0 0 12px 2px rgba(196, 105, 61, 0.15)" },
        },
        shimmer: {
          "0%": { backgroundPosition: "-200% 0" },
          "100%": { backgroundPosition: "200% 0" },
        },
        slideInRight: {
          "0%": { opacity: "0", transform: "translateX(100%)" },
          "100%": { opacity: "1", transform: "translateX(0)" },
        },
      },
      backgroundImage: {
        "gradient-radial": "radial-gradient(var(--tw-gradient-stops))",
        "noise": "url(\"data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noise'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.65' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noise)' opacity='0.03'/%3E%3C/svg%3E\")",
      },
    },
  },
  plugins: [],
} satisfies Config;
