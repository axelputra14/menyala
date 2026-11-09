<script setup lang="ts">
import { ref, computed, watch, nextTick, onUnmounted } from "vue";
import { useFetchStore } from "../stores/fetch";
import { AppEntry } from "../types/app";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

const props = defineProps<{
    visible: boolean;
}>();

const emit = defineEmits(["close"]);

const fetchStore = useFetchStore();
const localApps = ref<AppEntry[]>([]);
const isSaving = ref(false);
const errorMsg = ref("");
const confirmingDiscard = ref(false);
// serialized list as it was when the editor opened, to detect unsaved edits
const snapshot = ref("");

const isDirty = computed(
    () => JSON.stringify(localApps.value) !== snapshot.value,
);

// Initialize local list from store when modal opens
watch(
    () => props.visible,
    (newVal) => {
        if (newVal) {
            localApps.value = fetchStore.appList.map((app) => ({
                ...app,
                id: app.id ?? crypto.randomUUID(),
            }));
            snapshot.value = JSON.stringify(localApps.value);
            errorMsg.value = "";
            confirmingDiscard.value = false;
            window.addEventListener("keydown", onKeydown);
        } else {
            window.removeEventListener("keydown", onKeydown);
        }
    },
    { immediate: true },
);

onUnmounted(() => window.removeEventListener("keydown", onKeydown));

// Esc backs out of the discard prompt or asks to close; Ctrl+S saves
function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
        e.preventDefault();
        if (confirmingDiscard.value) {
            confirmingDiscard.value = false;
        } else {
            requestClose();
        }
    } else if (e.ctrlKey && e.key.toLowerCase() === "s") {
        e.preventDefault();
        if (isDirty.value && !isSaving.value) {
            saveChanges();
        }
    }
}

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
async function resetPos() {
    await fetchStore.refreshApps();
}

// every way of dismissing the editor must also shrink/reposition the window
function closeEditor() {
    confirmingDiscard.value = false;
    emit("close");
    resetPos();
}

// dismiss requests (overlay click, ×, Cancel) ask first when there are unsaved edits
function requestClose() {
    if (isDirty.value) {
        confirmingDiscard.value = true;
        return;
    }
    closeEditor();
}

async function selectExeFile(app: AppEntry) {
    try {
        const file = await open({
            multiple: false,
            directory: false,
            title: "Select executable file",
            filters: [
                {
                    name: "Filters",
                    extensions: ["exe", "bat"],
                },
            ],
        });
        // null means the user cancelled the dialog, so keep the existing path
        if (file) {
            app.exe = file;
            // old icon belongs to the previous exe; refresh_apps resolves the new one on save
            app.icon = null;
        }
    } catch (error) {
        console.error("Error selecting file:", error);
    }
}
</script>

<template>
    <Transition name="fade">
        <div v-if="visible" class="editor-overlay" @click.self="requestClose">
            <div class="editor-modal">
                <div class="editor-header">
                    <h2>Manage Applications</h2>
                    <button
                        class="btn-close"
                        title="Close"
                        aria-label="Close"
                        @click="requestClose"
                    >
                        &times;
                    </button>
                </div>

                <div v-if="confirmingDiscard" class="discard-banner">
                    <span>You have unsaved changes. Discard them?</span>
                    <div class="discard-actions">
                        <button
                            class="btn-ghost"
                            @click="confirmingDiscard = false"
                        >
                            Keep editing
                        </button>
                        <button class="btn-danger" @click="closeEditor">
                            Discard
                        </button>
                    </div>
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
                            <img
                                class="card-icon"
                                :src="
                                    app.icon
                                        ? convertFileSrc(app.icon)
                                        : '/tauri.svg'
                                "
                                alt=""
                            />
                        </div>

                        <div class="card-inputs">
                            <div class="input-row">
                                <div class="input-group">
                                    <label :for="`${app.id}-name`">Name</label>
                                    <input
                                        :id="`${app.id}-name`"
                                        type="text"
                                        v-model="app.name"
                                        placeholder="e.g. VS Code"
                                    />
                                </div>
                                <div class="input-group">
                                    <label :for="`${app.id}-publisher`"
                                        >Publisher</label
                                    >
                                    <input
                                        :id="`${app.id}-publisher`"
                                        type="text"
                                        v-model="app.publisher"
                                        placeholder="e.g. Microsoft"
                                    />
                                </div>
                            </div>
                            <div class="input-group full-width">
                                <label :for="`${app.id}-exe`"
                                    >Executable Path</label
                                >
                                <div class="path-row">
                                    <input
                                        :id="`${app.id}-exe`"
                                        type="text"
                                        v-model="app.exe"
                                        placeholder="C:\Path\to\app.exe"
                                    />
                                    <button
                                        type="button"
                                        class="btn-browse"
                                        title="Browse for executable"
                                        aria-label="Browse for executable"
                                        @click="selectExeFile(app)"
                                    >
                                        <svg
                                            xmlns="http://www.w3.org/2000/svg"
                                            width="1em"
                                            height="1em"
                                            viewBox="0 0 36 36"
                                        >
                                            <path d="M0 0h36v36H0z" fill="none" />
                                            <path
                                                fill="currentColor"
                                                d="M35.32 13.74a1.71 1.71 0 0 0-1.45-.74h-22.7a2.59 2.59 0 0 0-2.25 1.52a1 1 0 0 0 0 .14L6 25V7h6.49l2.61 3.59a1 1 0 0 0 .81.41H32a2 2 0 0 0-2-2H16.42l-2.31-3.18A2 2 0 0 0 12.49 5H6a2 2 0 0 0-2 2v22.69A1.37 1.37 0 0 0 5.41 31h24.93a1 1 0 0 0 1-.72l4.19-15.1a1.64 1.64 0 0 0-.21-1.44"
                                                class="clr-i-solid clr-i-solid-path-1"
                                            />
                                            <path fill="none" d="M0 0h36v36H0z" />
                                        </svg>
                                    </button>
                                </div>
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
                        <button class="btn-ghost" @click="requestClose">
                            Cancel
                        </button>
                        <button
                            class="btn-primary"
                            :disabled="isSaving || !isDirty"
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

<style scoped>
/* open/close fade for the overlay */
.fade-enter-active,
.fade-leave-active {
    transition: opacity 0.5s cubic-bezier(0.55, 0, 0.1, 1);
}

.fade-enter-from,
.fade-leave-to {
    opacity: 0;
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
}

.editor-modal {
    width: 90%;
    max-width: 650px;
    height: 90vh;
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
    transition: color 0.25s;
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

.discard-banner {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    background: rgba(234, 179, 8, 0.15);
    border-left: 4px solid #eab308;
    color: #fde68a;
    padding: 0.5rem 0.75rem 0.5rem 1.25rem;
    margin: 0.5rem 1.5rem 0;
    border-radius: 0.375rem;
    font-size: 0.875rem;
}

.discard-actions {
    display: flex;
    gap: 0.5rem;
    flex-shrink: 0;
}

.btn-danger {
    font-family: inherit;
    font-size: 0.875rem;
    font-weight: 500;
    border-radius: 0.5rem;
    padding: 0.4rem 0.9rem;
    cursor: pointer;
    background: #ef4444;
    border: 1px solid #ef4444;
    color: #fff;
    transition: background 0.25s;
}

.btn-danger:hover {
    background: #dc2626;
    border-color: #dc2626;
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
        background 0.25s,
        border-color 0.25s;
}

.app-edit-card:hover {
    background: rgba(255, 255, 255, 0.07);
    border-color: rgba(255, 255, 255, 0.15);
}

.card-left {
    display: flex;
    align-items: center;
    gap: 0.75rem;
}

.card-icon {
    width: 40px;
    height: 40px;
    object-fit: contain;
    flex-shrink: 0;
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
    transition: all 0.25s;
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
        border-color 0.25s,
        box-shadow 0.25s;
}

.input-group input:focus {
    outline: none;
    border-color: rgba(99, 102, 241, 0.6);
    box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.2);
}

.path-row {
    display: flex;
    gap: 0.5rem;
}

.path-row input {
    flex: 1;
    min-width: 0;
}

.btn-browse {
    flex-shrink: 0;
    width: 2.25rem;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 0.375rem;
    color: #9ca3af;
    font-size: 1rem;
    cursor: pointer;
    transition:
        background 0.25s,
        color 0.25s;
}

.btn-browse:hover {
    background: rgba(255, 255, 255, 0.12);
    color: #fff;
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
    transition: all 0.25s;
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
    transition: all 0.25s;
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
