<script setup lang="ts">
import { ref, watch, nextTick } from "vue";
import { useFetchStore } from "../stores/fetch";
import { AppEntry } from "../types/app";
import { invoke } from "@tauri-apps/api/core";

const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits(["close"]);

const fetchStore = useFetchStore();
const localApps = ref<AppEntry[]>([]);
const isSaving = ref(false);
const errorMsg = ref("");

// Initialize local list from store when modal opens
watch(
  () => props.visible,
  (newVal) => {
    if (newVal) {
      localApps.value = fetchStore.appList.map((app) => ({
        ...app,
        id: crypto.randomUUID(),
      }));
      errorMsg.value = "";
    }
  },
  { immediate: true },
);

function addApp() {
  localApps.value.push({
    id: crypto.randomUUID(),
    name: "New App",
    exe: "",
    publisher: "Custom",
    icon: null,
  });
  // Auto-scroll list to bottom after adding
  nextTick(() => {
    const listContainer = document.querySelector(".editor-list");
    if (listContainer) {
      listContainer.scrollTop = listContainer.scrollHeight;
    }
  });
}

function removeApp(index: number) {
  localApps.value.splice(index, 1);
}

function moveApp(index: number, direction: -1 | 1) {
  const targetIndex = index + direction;
  if (targetIndex < 0 || targetIndex >= localApps.value.length) return;
  const temp = localApps.value[index];
  localApps.value[index] = localApps.value[targetIndex];
  localApps.value[targetIndex] = temp;
}

async function saveChanges() {
  errorMsg.value = "";
  // Simple validation
  for (let i = 0; i < localApps.value.length; i++) {
    const app = localApps.value[i];
    if (!app.name.trim()) {
      errorMsg.value = `App #${i + 1} must have a name.`;
      return;
    }
    if (!app.exe.trim()) {
      errorMsg.value = `App "${app.name}" must have an executable path.`;
      return;
    }
  }

  isSaving.value = true;
  try {
    // Send to backend
    await invoke("update_apps", { apps: localApps.value });
    // Trigger refresh (which extracts icons for new apps)
    await fetchStore.refreshApps();
    emit("close");
  } catch (err: any) {
    errorMsg.value = err.toString() || "Failed to save changes.";
  } finally {
    isSaving.value = false;
  }
}
</script>

<template>
  <Transition name="fade">
    <div v-if="visible" class="editor-overlay" @click.self="emit('close')">
      <div class="editor-modal">
        <div class="editor-header">
          <h2>Manage Applications</h2>
          <button class="btn-close" @click="emit('close')">&times;</button>
        </div>

        <div v-if="errorMsg" class="error-banner">
          {{ errorMsg }}
        </div>

        <div class="editor-list hide-scrollbar">
          <div
            v-for="(app, index) in localApps"
            :key="app.id"
            class="app-edit-card"
          >
            <div class="card-left">
              <div class="reorder-controls">
                <button
                  class="btn-arrow"
                  :disabled="index === 0"
                  @click="moveApp(index, -1)"
                  title="Move Up"
                >
                  ▲
                </button>
                <button
                  class="btn-arrow"
                  :disabled="index === localApps.length - 1"
                  @click="moveApp(index, 1)"
                  title="Move Down"
                >
                  ▼
                </button>
              </div>
            </div>

            <div class="card-inputs">
              <div class="input-row">
                <div class="input-group">
                  <label>Name</label>
                  <input
                    type="text"
                    v-model="app.name"
                    placeholder="e.g. VS Code"
                  />
                </div>
                <div class="input-group">
                  <label>Publisher</label>
                  <input
                    type="text"
                    v-model="app.publisher"
                    placeholder="e.g. Microsoft"
                  />
                </div>
              </div>
              <div class="input-group full-width">
                <label>Executable Path</label>
                <input
                  type="text"
                  v-model="app.exe"
                  placeholder="C:\Path\to\app.exe"
                />
              </div>
            </div>

            <div class="card-actions">
              <button
                class="btn-delete"
                @click="removeApp(index)"
                title="Remove application"
              >
                Delete
              </button>
            </div>
          </div>

          <div v-if="localApps.length === 0" class="empty-state">
            No applications configured. Click below to add one.
          </div>
        </div>

        <div class="editor-footer">
          <button class="btn-secondary" @click="addApp">
            + Add Application
          </button>
          <div class="footer-right">
            <button class="btn-ghost" @click="emit('close')">Cancel</button>
            <button
              class="btn-primary"
              :disabled="isSaving"
              @click="saveChanges"
            >
              {{ isSaving ? "Saving..." : "Save Changes" }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style>
.list-move, /* apply transition to moving elements */
.list-enter-active,
.list-leave-active {
  transition: all 0.5s cubic-bezier(0.55, 0, 0.1, 1);
}

.list-enter-from,
.list-leave-to {
  opacity: 0;
  transform: scaleY(0.01) translate(30px, 0);
}

/* ensure leaving items are taken out of layout flow so that moving
   animations can be calculated correctly. */
.list-leave-active {
  position: absolute;
}

/* Animations */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.5s cubic-bezier(0.55, 0, 0.1, 1);
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: scaleY(0.01) translate(30px, 0);
}
</style>

<style scoped>
.apps-group {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  position: relative;
}
.editor-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;

  backdrop-filter: blur(8px);
  z-index: 100;
  display: flex;
  justify-content: center;
  align-items: center;
  animation: fadeIn 0.5s ease-out;
}

.editor-modal {
  width: 90%;
  max-width: 650px;
  height: 80vh;
  background: rgba(20, 20, 30, 0.75);
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 1rem;

  display: flex;
  flex-direction: column;
  color: #f3f4f6;
  overflow: hidden;
}

.editor-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1.25rem 1.5rem;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.editor-header h2 {
  margin: 0;
  font-size: 1.25rem;
  font-weight: 600;
  letter-spacing: -0.025em;
  color: #fff;
}

.btn-close {
  background: transparent;
  border: none;
  color: #9ca3af;
  font-size: 1.5rem;
  cursor: pointer;
  padding: 4px;
  line-height: 1;
  transition: color 0.5s;
}

.btn-close:hover {
  color: #f3f4f6;
}

.error-banner {
  background: rgba(239, 68, 68, 0.2);
  border-left: 4px solid #ef4444;
  color: #fca5a5;
  padding: 0.75rem 1.25rem;
  margin: 0.5rem 1.5rem 0;
  border-radius: 0.375rem;
  font-size: 0.875rem;
}

.editor-list {
  flex: 1;
  padding: 1.5rem;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.app-edit-card {
  display: flex;
  align-items: stretch;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 0.75rem;
  padding: 1rem;
  gap: 1rem;
  transition:
    background 0.5s,
    border-color 0.5s;
}

.app-edit-card:hover {
  background: rgba(255, 255, 255, 0.07);
  border-color: rgba(255, 255, 255, 0.15);
}

.card-left {
  display: flex;
  align-items: center;
}

.reorder-controls {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.btn-arrow {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 4px;
  color: #9ca3af;
  font-size: 0.65rem;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.5s;
}

.btn-arrow:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.15);
  color: #fff;
}

.btn-arrow:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.card-inputs {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.input-row {
  display: flex;
  gap: 0.75rem;
}

.input-group {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.input-group.full-width {
  flex: none;
  width: 100%;
}

.input-group label {
  font-size: 0.75rem;
  font-weight: 500;
  color: #9ca3af;
}

.input-group input {
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 0.375rem;
  padding: 0.5rem 0.75rem;
  color: #fff;
  font-family: inherit;
  font-size: 0.875rem;
  transition:
    border-color 0.5s,
    box-shadow 0.5s;
}

.input-group input:focus {
  outline: none;
  border-color: rgba(99, 102, 241, 0.6);
  box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.2);
}

.card-actions {
  display: flex;
  align-items: center;
}

.btn-delete {
  background: rgba(239, 68, 68, 0.15);
  border: 1px solid rgba(239, 68, 68, 0.3);
  color: #fca5a5;
  border-radius: 0.375rem;
  padding: 0.5rem 0.75rem;
  font-size: 0.825rem;
  cursor: pointer;
  transition: all 0.5s;
}

.btn-delete:hover {
  background: #ef4444;
  color: #fff;
  border-color: #ef4444;
}

.empty-state {
  text-align: center;
  color: #6b7280;
  padding: 3rem 1rem;
  font-size: 0.875rem;
}

.editor-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1.25rem 1.5rem;
  border-top: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(0, 0, 0, 0.15);
}

.footer-right {
  display: flex;
  gap: 0.75rem;
}

.btn-primary,
.btn-secondary,
.btn-ghost {
  font-family: inherit;
  font-size: 0.875rem;
  font-weight: 500;
  border-radius: 0.5rem;
  padding: 0.5rem 1rem;
  cursor: pointer;
  transition: all 0.5s;
}

.btn-primary {
  background: #6366f1;
  border: 1px solid #6366f1;
  color: #fff;
  box-shadow: 0 4px 12px rgba(99, 102, 241, 0.3);
}

.btn-primary:hover:not(:disabled) {
  background: #4f46e5;
  border-color: #4f46e5;
  box-shadow: 0 4px 16px rgba(99, 102, 241, 0.45);
}

.btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.12);
  color: #fff;
}

.btn-secondary:hover {
  background: rgba(255, 255, 255, 0.12);
}

.btn-ghost {
  background: transparent;
  border: 1px solid transparent;
  color: #9ca3af;
}

.btn-ghost:hover {
  color: #fff;
}

.hide-scrollbar::-webkit-scrollbar {
  width: 6px;
}
.hide-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.hide-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 3px;
}
.hide-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.2);
}
</style>
