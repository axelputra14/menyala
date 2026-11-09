<script setup lang="ts">
import { ref, computed, watch, nextTick, onUnmounted } from "vue";
import { useFetchStore } from "../stores/fetch";
import { AppEntry } from "../types/app";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useDraggable, type DraggableEvent } from "vue-draggable-plus";

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
// cards picked for a multi-drag, by app id
const selectedIds = ref(new Set<string>());
// card the last Ctrl/Shift+click landed on, where a Shift+click range starts
let selectionAnchor: string | null = null;

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
            selectedIds.value = new Set();
            selectionAnchor = null;
            window.addEventListener("keydown", onKeydown);
        } else {
            window.removeEventListener("keydown", onKeydown);
        }
    },
    { immediate: true },
);

onUnmounted(() => window.removeEventListener("keydown", onKeydown));

// ---- multi-select: Ctrl+click toggles a card, Shift+click selects a range ----
const selectedCount = computed(
    () => localApps.value.filter((a) => selectedIds.value.has(a.id!)).length,
);

function onCardClick(e: MouseEvent, app: AppEntry) {
    if (!e.ctrlKey && !e.shiftKey) return;
    // leave modifier-clicks inside form controls alone (Shift+click in a text field, etc.)
    if ((e.target as HTMLElement).closest("input, button, label")) return;
    e.preventDefault();
    const id = app.id!;
    const next = new Set(selectedIds.value);
    if (e.shiftKey && selectionAnchor) {
        const ids = localApps.value.map((a) => a.id!);
        const from = ids.indexOf(selectionAnchor);
        const to = ids.indexOf(id);
        if (from !== -1 && to !== -1) {
            const [lo, hi] = from < to ? [from, to] : [to, from];
            ids.slice(lo, hi + 1).forEach((i) => next.add(i));
        }
    } else if (next.has(id)) {
        next.delete(id);
    } else {
        next.add(id);
    }
    selectionAnchor = id;
    selectedIds.value = next;
}

// ---- drag reordering (vue-draggable-plus, pointer-based) ----
const listRef = ref<HTMLElement | null>(null);
const draggingId = ref<string | null>(null);
// true while a selected card is dragged with other selected cards along with it
const isGroupDrag = computed(
    () =>
        draggingId.value !== null &&
        selectedIds.value.has(draggingId.value) &&
        selectedCount.value > 1,
);

let dragStartPoint: { x: number; y: number } | null = null;

// Sortable attaches the triggering pointer event at runtime, but its typings leave it out
function pointOf(e: DraggableEvent<AppEntry>) {
    const p = (e as { originalEvent?: PointerEvent }).originalEvent;
    return p && typeof p.clientX === "number"
        ? { x: p.clientX, y: p.clientY }
        : null;
}

// a press-and-release on the handle can still fire start/end; only a real drag should regroup
function pointerTravelled(e: DraggableEvent<AppEntry>): boolean {
    const end = pointOf(e);
    if (!dragStartPoint || !end) return true;
    return Math.hypot(end.x - dragStartPoint.x, end.y - dragStartPoint.y) > 4;
}

const sortable = useDraggable<AppEntry>(listRef, {
    immediate: false,
    handle: ".drag-handle",
    draggable: ".app-edit-card",
    // pointer events instead of HTML5 drag, which Tauri's file-drop handling swallows on Windows
    forceFallback: true,
    animation: 250,
    ghostClass: "drag-placeholder",
    fallbackClass: "drag-floating",
    onStart: (e: DraggableEvent<AppEntry>) => {
        draggingId.value = e.item.dataset.id ?? null;
        dragStartPoint = pointOf(e);
    },
    onEnd: onDragEnd,
});

// the list only exists while the editor is open (v-if), so attach Sortable when it appears
watch(
    listRef,
    (el) => {
        if (el) sortable.start(el);
        else sortable.destroy();
    },
    { flush: "post" },
);

function cardRects(): Map<string, DOMRect> {
    const rects = new Map<string, DOMRect>();
    listRef.value
        ?.querySelectorAll<HTMLElement>(".app-edit-card")
        .forEach((el) => rects.set(el.dataset.id!, el.getBoundingClientRect()));
    return rects;
}

// slide each card from where it was on screen to where Vue just put it (FLIP)
function animateFrom(before: Map<string, DOMRect>) {
    listRef.value
        ?.querySelectorAll<HTMLElement>(".app-edit-card")
        .forEach((el) => {
            const prev = before.get(el.dataset.id!);
            if (!prev) return;
            const now = el.getBoundingClientRect();
            const dx = prev.left - now.left;
            const dy = prev.top - now.top;
            if (!dx && !dy) return;
            el.style.transition = "none";
            el.style.transform = `translate(${dx}px, ${dy}px)`;
            el.getBoundingClientRect(); // commit the start position before animating
            el.style.transition = "transform 0.25s ease";
            el.style.transform = "";
            el.addEventListener(
                "transitionend",
                () => (el.style.transition = ""),
                { once: true },
            );
        });
}

async function onDragEnd(e: DraggableEvent<AppEntry>) {
    const draggedId = draggingId.value;
    const group = isGroupDrag.value && pointerTravelled(e);
    draggingId.value = null;
    dragStartPoint = null;
    const oldIndex = e.oldDraggableIndex;
    const newIndex = e.newDraggableIndex;
    if (!draggedId || oldIndex === undefined || newIndex === undefined) return;
    if (oldIndex === newIndex && !group) return;

    const before = cardRects();

    // Sortable moved the element itself; put it back so Vue stays the one that owns DOM order
    if (oldIndex !== newIndex) {
        const parent = e.from;
        parent.removeChild(e.item);
        const cards = parent.querySelectorAll(".app-edit-card");
        parent.insertBefore(e.item, cards[oldIndex] ?? null);
    }

    // order with only the dragged card moved, as Sortable showed it
    const order = [...localApps.value];
    const [dragged] = order.splice(oldIndex, 1);
    order.splice(newIndex, 0, dragged);

    if (group) {
        // gather the other selected cards around the dragged one, keeping their relative order
        const block = localApps.value.filter((a) => selectedIds.value.has(a.id!));
        const rest = order.filter(
            (a) => a.id === draggedId || !selectedIds.value.has(a.id!),
        );
        rest.splice(rest.indexOf(dragged), 1, ...block);
        localApps.value = rest;
    } else {
        localApps.value = order;
    }

    await nextTick();
    animateFrom(before);
}

// Esc backs out of the discard prompt, then clears the selection, then asks to close; Ctrl+S saves
function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
        e.preventDefault();
        if (confirmingDiscard.value) {
            confirmingDiscard.value = false;
        } else if (selectedCount.value > 0) {
            selectedIds.value = new Set();
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
        if (listRef.value) {
            listRef.value.scrollTop = listRef.value.scrollHeight;
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
                    <span v-if="selectedCount > 0" class="selection-pill">
                        {{ selectedCount }} selected · Esc to clear
                    </span>
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

                <div ref="listRef" class="editor-list hide-scrollbar">
                    <div
                        v-for="(app, index) in localApps"
                        :key="app.id"
                        :data-id="app.id"
                        class="app-edit-card"
                        :class="{
                            'is-selected': selectedIds.has(app.id!),
                            'is-companion':
                                isGroupDrag &&
                                selectedIds.has(app.id!) &&
                                app.id !== draggingId,
                        }"
                        @click="onCardClick($event, app)"
                    >
                        <span
                            v-if="isGroupDrag && app.id === draggingId"
                            class="group-badge"
                        >
                            {{ selectedCount }}
                        </span>
                        <div class="card-left">
                            <div
                                class="drag-handle"
                                title="Drag to reorder · Ctrl/Shift+click to select several"
                            >
                                <svg
                                    xmlns="http://www.w3.org/2000/svg"
                                    width="10"
                                    height="16"
                                    viewBox="0 0 10 16"
                                    aria-hidden="true"
                                >
                                    <g fill="currentColor">
                                        <circle cx="2" cy="2" r="1.5" />
                                        <circle cx="8" cy="2" r="1.5" />
                                        <circle cx="2" cy="8" r="1.5" />
                                        <circle cx="8" cy="8" r="1.5" />
                                        <circle cx="2" cy="14" r="1.5" />
                                        <circle cx="8" cy="14" r="1.5" />
                                    </g>
                                </svg>
                            </div>
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

.app-edit-card {
    position: relative; /* anchors the group badge */
    user-select: none; /* Shift+click selects cards, not text */
}

.app-edit-card input {
    user-select: text;
}

.app-edit-card.is-selected {
    background: rgba(99, 102, 241, 0.12);
    border-color: rgba(99, 102, 241, 0.6);
}

/* other selected cards while one of them is dragged; they join it on drop */
.app-edit-card.is-companion {
    opacity: 0.35;
}

/* the card's slot in the list while it is being dragged */
.app-edit-card.drag-placeholder {
    opacity: 0.3;
    border-style: dashed;
    border-color: rgba(99, 102, 241, 0.7);
}

/* the copy that follows the cursor */
.app-edit-card.drag-floating {
    opacity: 0.95 !important;
    background: rgba(30, 30, 45, 0.95);
    border-color: rgba(99, 102, 241, 0.7);
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.5);
}

.group-badge {
    position: absolute;
    top: -8px;
    right: -8px;
    min-width: 22px;
    height: 22px;
    padding: 0 6px;
    border-radius: 11px;
    background: #6366f1;
    color: #fff;
    font-size: 0.75rem;
    font-weight: 600;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 2px 8px rgba(99, 102, 241, 0.5);
}

.drag-handle {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 32px;
    border-radius: 4px;
    color: #6b7280;
    cursor: grab;
    transition:
        background 0.25s,
        color 0.25s;
}

.drag-handle:hover {
    background: rgba(255, 255, 255, 0.08);
    color: #f3f4f6;
}

.drag-handle:active {
    cursor: grabbing;
}

.selection-pill {
    margin-left: auto;
    margin-right: 0.75rem;
    padding: 0.2rem 0.6rem;
    border-radius: 999px;
    background: rgba(99, 102, 241, 0.18);
    color: #c7d2fe;
    font-size: 0.75rem;
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
