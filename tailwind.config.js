/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./packages/**/*.{rs,html}",
  ],
  theme: {
    extend: {},
  },
  plugins: [
    require('daisyui'),
  ],
  daisyui: {
    themes: ["light", "lofi"],
  },
}
