import { ref, watch } from "vue";

type ThemeChoice = "auto" | "light" | "dark";

const theme = ref<ThemeChoice>(
  (localStorage.getItem("ccgate.theme") as ThemeChoice) || "dark"
);
const systemPrefersDark = ref(
  window.matchMedia?.("(prefers-color-scheme: dark)").matches ?? true
);

const mql = window.matchMedia?.("(prefers-color-scheme: dark)");
mql?.addEventListener?.("change", (e) => {
  systemPrefersDark.value = e.matches;
  apply();
});

function effectiveTheme(): "light" | "dark" {
  if (theme.value === "auto") return systemPrefersDark.value ? "dark" : "light";
  return theme.value;
}

function apply() {
  const root = document.documentElement;
  root.setAttribute("data-theme", effectiveTheme());
}

let initialized = false;

export function useTheme() {
  if (!initialized) {
    initialized = true;
    apply();
    watch(theme, () => {
      localStorage.setItem("ccgate.theme", theme.value);
      apply();
    });
  }
  return { theme, systemPrefersDark, effectiveTheme };
}
