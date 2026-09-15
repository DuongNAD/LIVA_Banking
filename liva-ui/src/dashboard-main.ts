import { createApp } from "vue";
import { createPinia } from "pinia";
import "./dashboard.css";
import "./uno";
import BankingApp from "./BankingApp.vue";

import { detectPlatform } from "./platform";

const app = createApp(BankingApp);
const pinia = createPinia();
app.use(pinia);

app.provide('platform', detectPlatform());
app.mount("#app");

