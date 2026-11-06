<script setup>
import { onMounted, onUnmounted, nextTick, watch } from "vue";
import { useKeenSlider } from "keen-slider/vue.es";
import "keen-slider/keen-slider.min.css";
import { useFetchStore } from "../stores/fetch";

import { convertFileSrc } from "@tauri-apps/api/core";
import { invoke } from "@tauri-apps/api/core";

const fetchStore = useFetchStore();

// const loading = ref(true);

const emit = defineEmits(["hover", "leave", "error"]);

const SLIDE_SPACING = 32;
const MAX_SCALE = 1.5;
const MAX_LIFT = 14; // px
const REACH = 2.5; // how far the lift spreads, in icon widths including spacing
const EASE = 0.2; // share of the remaining distance covered per frame on enter/leave

let pointerX = 0;
let pointerInside = false;
let strength = 0; // 0 = at rest, 1 = fully lifted
let nearestIndex = null;
let frame = null;

function slideCenter(slide) {
  const rect = slide.getBoundingClientRect();
  return rect.left + rect.width / 2;
}

function updateTooltip(centers) {
  let nearest = null;
  centers.forEach((center, index) => {
    if (
      nearest === null ||
      Math.abs(center - pointerX) < Math.abs(centers[nearest] - pointerX)
    ) {
      nearest = index;
    }
  });

  const app = fetchStore.appList[nearest];
  if (nearest !== nearestIndex && app) {
    nearestIndex = nearest;
    emit("hover", app.name);
  }
}

// Runs every frame while hovered, so the lift also follows icons scrolling under a still cursor
function renderLift() {
  frame = null;
  const slides = slider.value?.slides ?? [];

  strength += ((pointerInside ? 1 : 0) - strength) * EASE;
  if (!pointerInside && strength < 0.01) strength = 0;

  if (slides.length > 0) {
    const centers = slides.map(slideCenter);
    const reach =
      REACH * (slides[0].getBoundingClientRect().width + SLIDE_SPACING);

    slides.forEach((slide, index) => {
      const img = slide.querySelector("img");
      if (!img) return;

      const distance = Math.abs(centers[index] - pointerX);
      const lift =
        distance < reach ? (1 + Math.cos((Math.PI * distance) / reach)) / 2 : 0;
      const amount = lift * strength;

      img.style.transform = amount
        ? `translateY(${-MAX_LIFT * amount}px) scale(${1 + (MAX_SCALE - 1) * amount})`
        : "";
      img.style.zIndex = amount ? Math.round(lift * 10) : "";
    });

    if (pointerInside) updateTooltip(centers);
  }

  if (pointerInside || strength > 0) requestLift();
}

function requestLift() {
  if (frame === null) frame = requestAnimationFrame(renderLift);
}

function onPointerMove(event) {
  pointerX = event.clientX;
  pointerInside = true;
  requestLift();
}

function onPointerLeave() {
  pointerInside = false;
  nearestIndex = null;
  emit("leave");
  requestLift();
}

function launchApp(app) {
  // launch slightly after the click
  setTimeout(() => {
    invoke("launch_app", { exe: app.exe }).catch((err) => {
      // user dismissed the UAC prompt
      if (err === "cancelled") return;
      emit("error", `Couldn't launch ${app.name}: ${err}`);
    });
  }, 40);
}
const wheelControls = (slider) => {
  let touchTimeout;
  let position = { x: 0 };
  let wheelActive = true;

  const dispatch = (e, name) => {
    position.x -= e.deltaY;
    slider.container.dispatchEvent(
      new CustomEvent(name, { detail: { x: position.x, y: 0 } }),
    );
  };

  const eventWheel = (e) => {
    e.preventDefault();
    if (!wheelActive) {
      position = { x: 0 };
      dispatch(e, "ksDragStart");
      wheelActive = true;
    }
    dispatch(e, "ksDrag");
    clearTimeout(touchTimeout);
    touchTimeout = setTimeout(() => {
      wheelActive = false;
      dispatch(e, "ksDragEnd");
    }, 50);
  };

  slider.on("created", () => {
    slider.container.addEventListener("wheel", eventWheel, { passive: false });
  });
};

const sliderOptions = {
  loop: true,
  mode: "free-snap",
  rubberband: true,
  slides: { perView: 8, spacing: SLIDE_SPACING },
};

const [container, slider] = useKeenSlider(sliderOptions, [wheelControls]);

// Passing options (instead of a bare update()) fires "optionsChanged", which makes
// keen-slider's renderer re-read the slide elements in their new DOM order.
// A bare update() keeps rendering against the stale element list, so reordered
// apps stay in their old positions.
watch(
  () => fetchStore.appList,
  async () => {
    await nextTick();
    slider.value?.update(sliderOptions);
  },
  { deep: true, flush: "post" },
);

onMounted(async () => {
  await fetchStore.getApps();
  await fetchStore.refreshApps();
});

onUnmounted(() => {
  if (frame !== null) cancelAnimationFrame(frame);
});
</script>

<template>
  <div
    class="slider-wrapper"
    @pointerenter="onPointerMove"
    @pointermove="onPointerMove"
    @pointerleave="onPointerLeave"
  >
    <div class="hover-zone"></div>
    <div ref="container" class="keen-slider">
      <div
        v-for="app in fetchStore.appList"
        :key="app.id ?? app.exe"
        class="keen-slider__slide slide-items"
        @click="launchApp(app)"
      >
        <img
          :src="app.icon ? convertFileSrc(app.icon) : '/tauri.svg'"
          v-bind:alt="app.name"
          width="64"
          height="64"
        />
      </div>
    </div>
  </div>
</template>

<style>
/* .tooltip-container {

} */

body {
  margin: 0;
  font-family: "Inter", sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

.slider-wrapper {
  position: relative; /* needed for overlays */
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(115, 115, 155, 0.4);
  border-radius: 1.5rem;
  padding: 8px 16px;
  margin-bottom: 0.25rem;
  margin-top: 3vh;
  /* macOS dock-style: bottom-only shadow via negative spread */
  box-shadow: 0 12px 24px -20px rgba(0, 0, 0, 0.6);
}

/* gradient overlays on left + right */
.slider-wrapper::before,
.slider-wrapper::after {
  content: "";
  position: absolute;
  top: 0;
  bottom: 0;
  width: 48px;
  pointer-events: none;
  z-index: 5;
  border-radius: 1.5rem;
}

.slider-wrapper::before {
  left: 0;
  background: linear-gradient(
    to right,
    rgba(0, 0, 0, 0.3),
    rgba(255, 255, 255, 0.01)
  );
  border-radius: 1.5rem;
}

.slider-wrapper::after {
  right: 0;
  background: linear-gradient(
    to left,
    rgba(0, 0, 0, 0.3),
    rgba(255, 255, 255, 0.01)
  );
  border-radius: 1.5rem;
}

/* keeps the dock hovered while the cursor is between lifted icons, above the wrapper */
.hover-zone {
  position: absolute;
  bottom: 100%;
  left: 0;
  right: 0;
  height: 30px;
}

.keen-slider {
  overflow: visible !important; /* no mask anymore */
}

.keen-slider__slide {
  max-height: 64px;
  max-width: 64px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.keen-slider__slide img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.keen-slider__slide:hover {
  cursor: pointer;
}

.slide-items {
  overflow: visible !important;
  position: relative; /* tooltip positions relative to icon */
}

.slide-items svg,
.slide-items img {
  transform-origin: 50% 100%;
  will-change: transform;
}

.slide-items svg.loading,
.slide-items img.loading {
  animation: 1s loading ease-in infinite;
}
</style>
