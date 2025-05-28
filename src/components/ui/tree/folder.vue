<template>
  <div :id ref="folderRef" data-nested-draggable="" :value class="">
    <div
isDir :data-tree-view-item="JSON.stringify({ value })"
      class="accordion-item active motion-preset-slide-left motion-ease-spring-bouncier" :class="{
        'selected': isActive?.value,
        'text-base-content': !color,
      }" role="treeitem" :style="{ color: color || '' }">
      <div
        class="accordion-heading tree-view-selected:bg-primary/80 flex items-center gap-x-0.5 rounded-md hover:bg-primary/20 group">
        <button class="accordion-toggle btn btn-sm btn-circle btn-text shrink-0" :aria-controls="controlId">
          <Icon name="tabler:plus" class="accordion-item-active:rotate-45 size-4 transition-all duration-300" />
        </button>
        <button class="cursor-pointer rounded-md px-1.5 w-full" @click.stop="$emit('click', value)">
          <div class="flex items-center gap-x-3">
            <Icon v-if="icon" :name="icon || 'mdi:folder-outline'" class="shrink-0" />

            <div class="flex whitespace-nowrap">
              {{ value }}
            </div>
          </div>
        </button>
        <button
class="sticky right-2 btn btn-sm btn-circle btn-text shrink-0 group-hover:flex hidden ml-auto"
          @click.stop="$emit('edit', value)">
          <Icon name="mdi:pencil-outline" class="size-4 transition-all duration-300" />
        </button>
      </div>

      <div
:id="controlId" class="accordion-content w-full transition-[height] duration-300" role="group"
        :aria-labelledby="id">
        <div ref="childRef" class="tree-view-space min-h-1" data-nested-draggable="">
          <slot>
            <template
v-for="(item, index) in children?.sort(
              (a, b) => a.order ?? 0 - (b.order ?? 0)
            )" :key="item.id!" :data-tree-view-item="JSON.stringify({ value: item.value })">
              <UiTreeFolder
v-if="item.type === 'folder'" :icon="item.icon || 'tabler:folder'"
                v-bind="item" @click="(value) => $emit('click', value)" @edit="(value) => $emit('edit', value)" />

              <UiTreeFile v-if="item.type === 'file'" v-bind="item" />
            </template>
          </slot>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { HSAccordion } from 'flyonui/flyonui';
import Sortable from 'sortablejs';

const props = defineProps({
  value: String,
  icon: {
    type: [String, null],
    default: 'tabler:folder',
  },
  children: {
    type: Array as PropType<ITreeItem[] | null>,
    default: () => [],
  },
  name: String,
  color: [String, null],
  isActive: Object as PropType<ComputedRef<boolean>>,
});

const id = useId();
const controlId = useId();
const folderRef = ref<HTMLElement>();
const childRef = ref<HTMLElement>();

defineEmits<{
  click: [value: string | undefined];
  edit: [value: string | undefined];
}>();

const { groups } = storeToRefs(useVaultGroupStore());
const sorty = ref([]);
onMounted(() => {
  if (folderRef.value && childRef.value)
    [folderRef.value, childRef.value].forEach((element) => {
      const create = Sortable.create(element, {
        animation: 150,
        ghostClass: 'bg-opacity-20',
        group: 'vault',
        swapThreshold: 0.65,
        fallbackOnBody: true,
        fallbackTolerance: 3,

        onEnd: (evt) => {
          const { item } = evt;

          /* if (item.classList.contains('accordion')) {
            let existingInstance = HSAccordion.getInstance(item, true);
            let updatedInstance;

            existingInstance.element.update();
            updatedInstance = HSAccordion.getInstance(item, true);
            window.$hsAccordionCollection.map((el) => {
              if (
                el.element.el !== existingInstance.element.el &&
                el.element.group === existingInstance.element.group &&
                el.element.el.closest('.accordion') &&
                el.element.el.classList.contains('active') &&
                existingInstance.element.el.classList.contains('active')
              )
                el.element.hide();

              return el;
            });
          }

          if (!!item.hasAttribute('data-tree-view-item')) {
            const treeViewItem = HSTreeView.getInstance(
              item.closest('[data-tree-view]'),
              true
            );

            treeViewItem.element.update();
          } */
        },
        onUpdate: (evt) => {
          console.log('update', evt.item, props.value, sorty.value);
        },
      });
      /* const sortable = new Sortable(element, {
        animation: 150,
        ghostClass: 'bg-opacity-20',
        group: 'vault',
        swapThreshold: 0.65,
        fallbackOnBody: true,
        fallbackTolerance: 3,

        onEnd: (evt) => {
          console.log(
            'end',
            evt.item,
            props.value,
            sorty.value.at(0).toArray(),
            sorty.value.at(1).toArray()
          );
        },
        onUpdate: (evt) => {
          console.log('update', evt.item, props.value, sorty.value);
        },
      });
      sorty.value.push(sortable); */
    });
});

onMounted(() => {
  const draggable = document.querySelectorAll('[data-nested-draggable]');

  draggable.forEach((el) => {
    const options = {
      group: 'nested',
      animation: 150,
      fallbackOnBody: true,
      swapThreshold: 0.65,
      ghostClass: 'dragged',
      onEnd: (evt) => {
        const { item, items } = evt;
        console.log('standard', item, evt);
        if (item.classList.contains('accordion')) {
          const existingInstance = HSAccordion.getInstance(item, true);
          let updatedInstance;

          existingInstance.element.update();
          updatedInstance = HSAccordion.getInstance(item, true);
          window.$hsAccordionCollection.map((el) => {
            if (
              el.element.el !== existingInstance.element.el &&
              el.element.group === existingInstance.element.group &&
              el.element.el.closest('.accordion') &&
              el.element.el.classList.contains('active') &&
              existingInstance.element.el.classList.contains('active')
            )
              el.element.hide();

            return el;
          });
        }

        if (item.hasAttribute('data-tree-view-item')) {
          const treeViewItem = HSTreeView.getInstance(
            item.closest('[data-tree-view]'),
            true
          );

          treeViewItem.element.update();
        }
      },
    };
    const data = el.getAttribute('data-nested-draggable');
    const dataOptions = data ? JSON.parse(data) : {};
    const sortable = new Sortable(el, options);
    console.log('stand', sortable.toArray());
  });
});
</script>