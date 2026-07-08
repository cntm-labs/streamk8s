# Implementation Plan: OpenLens-like Logical Grouping

## Task 1: Refactor ResourceTree.vue
- Define the `CATEGORY_MAP` for K8s standard resources.
- Create a reactive state `logicalGroups` with predefined keys: `Workloads`, `Network`, `Config`, `Storage`, `Access Control`, `Custom Resources`.
- In `fetchApiResources`, after receiving the `apiGroups` from Tauri, process them:
  - Iterate through all API groups and resources.
  - Lookup each resource's `kind` in `CATEGORY_MAP`.
  - If a match is found, push it to the corresponding logical group.
  - If no match is found, place it inside `Custom Resources` (optionally retaining its original API group as a sub-category, but a flat list under Custom Resources is fine for MVP).
- Update the Vue `<template>` to loop through `logicalGroups` (omitting empty ones) instead of the raw `apiGroups`.
- Assign specific icons for the categories (e.g. 📦 for Workloads, 🌐 for Network, ⚙️ for Config, 💾 for Storage, 🔐 for Access Control, 🧩 for CRDs).

## Task 2: Testing and Validation
- Verify UI renders correctly without duplicate resources.
- Check that selecting a resource from the tree correctly triggers the resource view (DynamicResourceTable).
- Ensure unknown CRDs fallback to the "Custom Resources" section successfully.
