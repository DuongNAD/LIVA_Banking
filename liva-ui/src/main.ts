import { createApp } from "vue";
import { createPinia } from "pinia";
import "./style.css";
import "./uno";
import BankingApp from "./BankingApp.vue";

import { detectPlatform } from "./platform";

const app = createApp(BankingApp);
const pinia = createPinia();
app.use(pinia);

// [Phase 5.1] Tự động detect môi trường (Tauri/Web) và inject adapter
const platform = detectPlatform();
app.provide('platform', platform);

app.mount("#app");

