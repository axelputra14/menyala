import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { AppEntry } from "../types/app";

export const useFetchStore = defineStore("fetch", () => {
  const appList = ref<AppEntry[]>([]);

  async function getApps() {
    appList.value = await invoke("get_apps");
    console.log("ini applist: ", appList.value);
  }

  async function refreshApps() {
    appList.value = await invoke<AppEntry[]>("refresh_apps");
  }

  return { appList, getApps, refreshApps };
});
