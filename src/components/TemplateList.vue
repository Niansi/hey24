<script setup lang="ts">
import { computed } from 'vue';
import { useProjectStore } from '../stores/project';

const store = useProjectStore();

interface TemplateItem {
  id: string;
  name: string;
  disabled: boolean;
}

const templates: TemplateItem[] = [
  { id: 'silent-grow', name: '悄悄长大', disabled: false },
  { id: 'birthday', name: '生日快剪', disabled: true },
  { id: 'travel', name: '旅行日记', disabled: true },
];

const selectedTemplate = computed(() => store.currentTemplate?.id ?? null);

function handleSelect(id: string) {
  if (id === 'silent-grow') {
    store.loadTemplate(id);
  }
}

const resolutionOptions = [
  { label: '1080 x 1920', width: 1080, height: 1920 },
  { label: '720 x 1280', width: 720, height: 1280 },
];

const currentResolution = computed(() =>
  resolutionOptions.find(
    (r) => r.width === store.outputSettings.width && r.height === store.outputSettings.height
  )?.label ?? '1080 x 1920'
);

function handleResolutionChange(event: Event) {
  const select = event.target as HTMLSelectElement;
  const option = resolutionOptions.find((r) => r.label === select.value);
  if (option) {
    store.outputSettings.width = option.width;
    store.outputSettings.height = option.height;
  }
}
</script>

<template>
  <aside class="sidebar sidebar-left">
    <div class="section">
      <h3 class="section-title">模板</h3>
      <ul class="template-list">
        <li
          v-for="tpl in templates"
          :key="tpl.id"
          class="template-item"
          :class="{ active: selectedTemplate === tpl.id, disabled: tpl.disabled }"
          @click="handleSelect(tpl.id)"
        >
          {{ tpl.name }}
        </li>
      </ul>
    </div>

    <div class="section output-section">
      <h3 class="section-title">输出设置</h3>
      <div class="setting-row">
        <label class="setting-label">分辨率</label>
        <select :value="currentResolution" @change="handleResolutionChange">
          <option
            v-for="opt in resolutionOptions"
            :key="opt.label"
            :value="opt.label"
          >
            {{ opt.label }}
          </option>
        </select>
      </div>
      <div class="setting-row">
        <label class="setting-label">每张时长 (秒)</label>
        <input
          type="number"
          min="0.1"
          max="2.0"
          step="0.1"
          :value="store.outputSettings.durationPerPhoto"
          @input="store.outputSettings.durationPerPhoto = Number(($event.target as HTMLInputElement).value)"
        />
      </div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 200px;
  min-width: 200px;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  padding: 16px;
  gap: 24px;
}

.section-title {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  color: var(--text-secondary);
  letter-spacing: 0.5px;
  margin-bottom: 8px;
}

.template-list {
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.template-item {
  padding: 8px 12px;
  border-radius: var(--radius);
  font-size: 14px;
  cursor: pointer;
  border-left: 3px solid transparent;
  transition: background 0.2s, border-color 0.2s;
}

.template-item:hover {
  background: var(--bg-card);
}

.template-item.active {
  border-left-color: var(--accent);
  background: var(--bg-card);
  color: white;
}

.template-item.disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.output-section {
  margin-top: auto;
}

.setting-row {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 12px;
}

.setting-label {
  font-size: 12px;
  color: var(--text-secondary);
}
</style>
