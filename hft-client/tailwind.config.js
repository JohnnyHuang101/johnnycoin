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
          black: '#0c0c0c',
          dark: '#161616',
          green: '#22c55e',
          red: '#ef4444',
          text: '#e5e5e5',
        }
      }
    },
  },
  plugins: [],
}