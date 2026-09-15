<template>
  <!-- GuidedTourOverlay: Demo Day presentation overlay disabled for Core Banking Operations Workstation -->
  <div class="guided-tour-disabled hidden" aria-hidden="true"></div>
</template>

<script setup lang="ts">
import type { TourStep } from '../../types/treasury';

defineProps<{
  isActive?: boolean;
  currentStep?: TourStep | null;
  steps?: TourStep[];
  secondsRemaining?: number;
  isPaused?: boolean;
}>();

defineEmits<{
  (e: 'start'): void;
  (e: 'stop'): void;
  (e: 'next'): void;
  (e: 'prev'): void;
  (e: 'toggle-pause'): void;
  (e: 'go-to-step', index: number): void;
}>();
</script>
