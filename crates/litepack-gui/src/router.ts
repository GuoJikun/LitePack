import { createRouter, createWebHistory } from "vue-router";

const routes = [
  {
    path: "/",
    name: "home",
    component: () => import("./views/HomeView.vue"),
  },
  {
    path: "/open",
    name: "open",
    component: () => import("./views/OpenView.vue"),
  },
  {
    path: "/extract-here",
    name: "extract-here",
    component: () => import("./views/ExtractHereView.vue"),
  },
  {
    path: "/extract-to",
    name: "extract-to",
    component: () => import("./views/ExtractToView.vue"),
  },
  {
    path: "/extract-named",
    name: "extract-named",
    component: () => import("./views/ExtractNamedView.vue"),
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
