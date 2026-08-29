import { render } from "solid-js/web";
import { App } from "./App";
import { initI18n } from "./i18n";
import { initTheme } from "./theme";
import "./styles.css";

// Vor dem ersten Zeichnen, sonst blitzt kurz das falsche Thema auf.
initTheme();
initI18n();

render(() => <App />, document.getElementById("root")!);
