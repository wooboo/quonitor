/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'quota-green': '#10b981',
        'quota-yellow': '#f59e0b',
        'quota-orange': '#f97316',
        'quota-red': '#ef4444',
      }
    },
  },
  plugins: [],
}
