/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./themes/notebook/**/*.liquid", "./scripts/**/*.ts"],
  theme: {
    extend: {},
  },
  plugins: [
    require("@tailwindcss/typography"),
    require("@tailwindcss/forms"),
  ],
};
