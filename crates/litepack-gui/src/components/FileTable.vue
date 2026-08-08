<script setup lang="ts">
import { computed } from "vue";
import { useAppStore } from "../stores/app";
import { formatDate, humanSize, iconKind, typeLabel } from "../utils/format";

const store = useAppStore();

const sortLabel = computed(() => {
  if (store.sortKey === "size") return store.sortAsc ? "↑" : "↓";
  if (store.sortKey === "modified") return store.sortAsc ? "↑" : "↓";
  return store.sortAsc ? "↑" : "↓";
});

function sortBy(key: "name" | "size" | "modified") {
  if (store.sortKey === key) store.sortAsc = !store.sortAsc;
  else {
    store.sortKey = key;
    store.sortAsc = key !== "name";
  }
}

function rowClick(path: string, isDir: boolean) {
  if (isDir) {
    store.navigate(path);
  } else {
    store.toggleSelect(path);
  }
}
</script>

<template>
  <div class="content">
    <div class="path-bar">
      <div class="path-breadcrumb">
        <template v-for="(crumb, i) in store.breadcrumbs" :key="crumb.path">
          <span
            v-if="i > 0"
            class="sep"
          >›</span>
          <span
            class="crumb"
            :class="{ current: i === store.breadcrumbs.length - 1 }"
            @click="store.navigate(crumb.path)"
          >{{ crumb.name }}</span>
        </template>
      </div>
      <div class="path-size">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
        {{ humanSize(store.dirTotalSize) }} · {{ store.children.length }} 个{{ store.currentDir ? "条目" : "文件" }}
      </div>
      <div class="view-toggle">
        <button class="active" title="列表视图">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><line x1="3" y1="6" x2="3.01" y2="6"/><line x1="3" y1="12" x2="3.01" y2="12"/><line x1="3" y1="18" x2="3.01" y2="18"/></svg>
        </button>
        <button title="详情视图">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/></svg>
        </button>
      </div>
    </div>

    <div class="file-table-wrap">
      <table class="file-table">
        <thead>
          <tr>
            <th style="width:45%" @click="sortBy('name')">名称 <span class="sort-icon" :class="{ active: store.sortKey === 'name' }">{{ store.sortKey === 'name' ? sortLabel : '↕' }}</span></th>
            <th style="width:12%;text-align:right" @click="sortBy('size')">压缩前 <span class="sort-icon" :class="{ active: store.sortKey === 'size' }">{{ store.sortKey === 'size' ? sortLabel : '↕' }}</span></th>
            <th style="width:12%;text-align:right">压缩后</th>
            <th style="width:12%">类型</th>
            <th style="width:19%" @click="sortBy('modified')">修改日期 <span class="sort-icon" :class="{ active: store.sortKey === 'modified' }">{{ store.sortKey === 'modified' ? sortLabel : '↕' }}</span></th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="e in store.children"
            :key="e.path"
            :class="{ selected: store.selected.includes(e.path) }"
            @click="rowClick(e.path, e.is_dir)"
          >
            <td>
              <div class="col-name">
                <div class="file-icon" :class="iconKind(e)">
                  <svg v-if="e.is_dir" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
                  <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
                </div>
                <span class="filename" :title="e.path">{{ e.is_dir ? e.path.slice(store.currentDir.length).replace(/\/$/, '') : e.path.slice(store.currentDir.length) }}</span>
                <svg
                  v-if="e.encrypted"
                  class="lock-icon"
                  width="12"
                  height="12"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  title="加密"
                ><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
              </div>
            </td>
            <td class="col-num">{{ e.is_dir ? "—" : humanSize(e.size) }}</td>
            <td class="col-num">{{ e.is_dir ? "—" : humanSize(e.compressed_size) }}</td>
            <td class="col-type">{{ typeLabel(e) }}</td>
            <td class="col-date">{{ formatDate(e.modified) }}</td>
          </tr>
          <tr v-if="store.children.length === 0">
            <td colspan="5" style="text-align:center;color:var(--text-3);padding:24px">
              {{ store.search ? "没有匹配的条目" : "（空目录）" }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
