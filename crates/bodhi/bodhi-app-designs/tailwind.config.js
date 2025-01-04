const colors = require("tailwindcss/colors");
const defaultTheme = require("tailwindcss/defaultTheme");

module.exports = {
  future: {
    hoverOnlyWhenSupported: true,
  },
  content: ["./build/**/*.html"],
  theme: {
    extend: {
      fontFamily: {
        sans: ["Inter", ...defaultTheme.fontFamily.sans],
      },
      colors: {
        gray: {
          900: "#0A0D14",
          800: "#161922",
          700: "#20232D",
          600: "#31353F",
          500: "#525866",
          400: "#868C98",
          300: "#CDD0D5",
          200: "#E2E4E9",
          100: "#F6F8FA",
        },
        primary: colors.violet,
      },
      boxShadow: {
        xs: "0px 1px 2px rgba(16, 24, 40, 0.05)",
      },
      letterSpacing: {
        tightest: "-0.096px",
      },
    },
  },
  plugins: [require("@tailwindcss/forms")],
};
