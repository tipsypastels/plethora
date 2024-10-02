/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./themes/default/**/*.liquid", "./scripts/**/*.ts"],
  theme: {
    extend: {},
  },
  plugins: [
    require("@tailwindcss/typography"),
    require("@tailwindcss/forms"),
  ],
};
