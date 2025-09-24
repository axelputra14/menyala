import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

export const useFetchStore = defineStore("fetch", () => {
  const appList = ref<any[]>([]);

  async function getApps() {
    appList.value = await invoke("get_apps");
    console.log("ini applist: ", appList.value);
  }

  return { appList, getApps };
});
