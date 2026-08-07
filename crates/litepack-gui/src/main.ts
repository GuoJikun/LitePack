import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import { useAppStore } from "./stores/app";
import "./theme/variables.css";
import "./style.css";

const app = createApp(App);
app.use(createPinia());

const store = useAppStore();
store.setTheme(store.theme);

app.mount("#app");
