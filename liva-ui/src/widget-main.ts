import { createApp } from "vue";
import { createPinia } from "pinia";
import "./style.css";
import "./uno";
import TreasuryMiniDock from "./components/banking/TreasuryMiniDock.vue";
import { detectPlatform } from "./platform";

const app = createApp(TreasuryMiniDock);
const pinia = createPinia();
app.use(pinia);
app.provide('platform', detectPlatform());
app.mount("#app");
