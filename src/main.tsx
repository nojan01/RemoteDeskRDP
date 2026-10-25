import { render } from "solid-js/web";
import { App } from "./App";
import { VmDesklet } from "./VmDesklet";
import { initI18n } from "./i18n";
import { initTheme } from "./theme";
import "./styles.css";

// Vor dem ersten Zeichnen, sonst blitzt kurz das falsche Thema auf.
initTheme();
initI18n();

render(() => new URLSearchParams(window.location.search).has("desklet") ? <VmDesklet /> : <App />, document.getElementById("root")!);
