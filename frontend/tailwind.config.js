/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        terminal: {
          bg: '#0a0e1a',
          surface: '#0d1117',
          border: '#1f2937',
          text: '#e6edf3',
          muted: '#8b949e',
          neon: '#00ff9f',
          'neon-dim': '#00d084',
          success: '#00ff9f',
          error: '#ff4757',
          warning: '#ffa502',
        },
      },
      fontFamily: {
        mono: ['JetBrains Mono', 'Fira Code', 'Consolas', 'Monaco', 'monospace'],
        display: ['JetBrains Mono', 'monospace'],
      },
      animation: {
        'scan': 'scan 8s linear infinite',
        'glow': 'glow 2s ease-in-out infinite alternate',
        'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        'dots': 'dots 1.4s infinite',
      },
      keyframes: {
        scan: {
          '0%, 100%': { transform: 'translateY(-100%)' },
          '50%': { transform: 'translateY(100%)' },
        },
        glow: {
          '0%': { boxShadow: '0 0 5px rgba(0, 255, 159, 0.5), 0 0 10px rgba(0, 255, 159, 0.3)' },
          '100%': { boxShadow: '0 0 10px rgba(0, 255, 159, 0.8), 0 0 20px rgba(0, 255, 159, 0.5)' },
        },
        dots: {
          '0%, 20%': { content: '"."' },
          '40%': { content: '".."' },
          '60%, 100%': { content: '"..."' },
        },
      },
    },
  },
  plugins: [],
}
