<script>
import { useKeenSlider } from "keen-slider/vue.es";
import "keen-slider/keen-slider.min.css";

export default {
  setup() {
    const [container] = useKeenSlider({
      loop: true,
      mode: "free",
      slides: {
        perView: "auto",
        spacing: 10,
      },
    });
    return { container };
  },
};
</script>

<template>
  <div class="slider-wrapper">
    <div ref="container" class="keen-slider">
      <!-- generate 50 slides -->
      <div
        v-for="i in 50"
        :key="i"
        class="keen-slider__slide slide-items"
        :class="`number-slide${((i - 1) % 5) + 1}`"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="48"
          height="48"
          viewBox="0 0 32 32"
        >
          <path
            fill="#fafafa"
            d="M11.807 9.776c.011 0 .005 0 0 0M8.109 7.927c.011 0 .005 0 0 0m22.12 2.854c-.667-1.604-2.021-3.333-3.079-3.885c.865 1.692 1.365 3.396 1.552 4.661l.005.027c-1.739-4.329-4.681-6.073-7.088-9.871c-.12-.192-.24-.385-.36-.588c-.063-.104-.115-.208-.172-.319a2.8 2.8 0 0 1-.224-.609c0-.02-.015-.036-.036-.041h-.031l-.005.005c-.005 0-.011.005-.011.005s0-.005.005-.011c-3.417 2-4.828 5.505-5.193 7.729a8.3 8.3 0 0 0-3.041.776a.396.396 0 0 0-.197.489a.387.387 0 0 0 .525.224a7.4 7.4 0 0 1 2.651-.687l.089-.011c.125-.005.255-.011.38-.011a7.7 7.7 0 0 1 2.203.307l.125.037c.12.036.235.077.355.12c.083.031.172.063.255.099c.068.025.136.057.203.083q.157.073.313.152l.14.067q.155.08.303.167q.094.054.187.115a7.8 7.8 0 0 1 2.683 2.776c-.817-.572-2.287-1.145-3.697-.895c5.52 2.76 4.036 12.265-3.615 11.905a6.6 6.6 0 0 1-2.448-.568l-.26-.124c-1.876-.969-3.423-2.803-3.615-5.027c0 0 .708-2.64 5.072-2.64c.475 0 1.824-1.319 1.849-1.699c-.011-.125-2.683-1.187-3.724-2.213c-.557-.547-.817-.812-1.052-1.011a4 4 0 0 0-.401-.301a7.1 7.1 0 0 1-.041-3.751c-1.579.719-2.803 1.855-3.693 2.855h-.009c-.609-.771-.563-3.313-.532-3.844c-.005-.036-.453.229-.511.271c-.536.385-1.041.813-1.5 1.287a13.5 13.5 0 0 0-1.437 1.719a13 13 0 0 0-2.057 4.645a18 18 0 0 0-.249 1.417a8 8 0 0 0-.052.359a10 10 0 0 0-.089.881L.7 15.9c-.009.172-.02.339-.031.511v.077c0 8.48 6.875 15.355 15.355 15.355c7.593 0 13.9-5.516 15.135-12.756c.027-.197.047-.395.068-.593c.307-2.631-.031-5.401-.995-7.713z"
          />
        </svg>
      </div>
    </div>
  </div>
</template>

<style>
body {
  margin: 0;
  font-family: "Inter", sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

.slider-wrapper {
  position: relative;
  border-radius: 9999px;
  background-color: #2e2e33;
  padding-left: 16px;
  padding-right: 16px;
  padding-top: 8px;
  padding-bottom: 8px;
  overflow: hidden; /* <- clip the gradients into the rounded pill */
}

.slider-wrapper::before,
.slider-wrapper::after {
  content: "";
  position: absolute;
  top: 0;
  bottom: 0;
  width: 80px; /* wider = softer fade */
  pointer-events: none;
  z-index: 5;
}

.slider-wrapper::before {
  left: 0;
  background: linear-gradient(to right, #2e2e33 20%, rgba(240, 14, 14, 0) 100%);
}

.slider-wrapper::after {
  right: 0;
  background: linear-gradient(to left, #2e2e33 20%, rgba(240, 14, 14, 0) 100%);
}

[class^="number-slide"],
[class*=" number-slide"] {
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  color: #fff;
  font-weight: 500;

  max-height: 100vh;
}
.keen-slider__slide {
  height: 56px;
  min-width: 48px;
}

.keen-slider__slide:hover {
  cursor: pointer;
}

.slide-items {
  overflow: visible; /* allow child (icon) to move outside */
}

.slide-items svg,
.slide-items img {
  transition: transform 0.25s cubic-bezier(0.25, 1, 0.5, 1); /* easeOutBack-ish */
}

.slide-items:hover svg,
.slide-items:hover img {
  transform: translateY(-6px) scale(1);
}

/* 5 color variants */
/* .number-slide1 {
  background: linear-gradient(
    128deg,
    rgba(64, 175, 255, 1) 0%,
    rgba(63, 97, 255, 1) 100%
  );
}
.number-slide2 {
  background: linear-gradient(
    128deg,
    rgba(255, 154, 63, 1) 0%,
    rgba(255, 75, 64, 1) 100%
  );
}
.number-slide3 {
  background: linear-gradient(
    128deg,
    rgba(189, 255, 83, 1) 0%,
    rgba(43, 250, 82, 1) 100%
  );
}
.number-slide4 {
  background: linear-gradient(
    128deg,
    rgba(64, 255, 242, 1) 0%,
    rgba(63, 188, 255, 1) 100%
  );
}
.number-slide5 {
  background: linear-gradient(
    128deg,
    rgba(255, 64, 156, 1) 0%,
    rgba(255, 63, 63, 1) 100%
  );
} */
</style>
