<script lang="ts">
    // 分区选择组件，负责分区和子分区的选择
    import {createEventDispatcher} from 'svelte';
    import {invoke} from "@tauri-apps/api/core";
    import {createPop, partition} from "./common";

    const dispatch = createEventDispatcher();

    export let currentChildren;
    let scrollParent;
    let scrollChildren;

    export let current;
    let parentName;
    let currentTypelist = [];
    if ($partition === null) {
        $partition = [];
        invoke('archive_pre',)
            .then((res) => {
                $partition = res.data.typelist;
                console.log('加载分区表', res);
            }).catch((e) => {
            createPop(e, 5000);
            console.log(e);
        });
    }

    $: {
        //     console.log('!!', $partition);
        if (current) {
            currentTypelist = $partition.find(value => value.id === current).children;
        } else {
            current = $partition[0]?.id;
            parentName = $partition[0]?.name;
            currentTypelist = $partition[0]?.children ?? [];
        }
    }

    function archive_pre(type, event) {
        if (type.parent === 0) {
            current = type.id;
            currentTypelist = type.children;
            parentName = type.name;
            scrollParent = event.target;
            return;
        }
        current = type.parent;
        currentChildren = type.id;
        dispatch('tid', {
            tid: type.id,
            parent: parentName,
            children: type.name,
            scroll: [scrollParent, event.target],
        });
    }
</script>
<div class="max-h-52 grid grid-flow-col divide-x-2">
    <div class="container overflow-y-scroll max-h-52 items-center justify-center bg-white dark:bg-gray-800">
        {#each $partition as tid}
            <div class:selected="{current === tid.id}" on:click={(event)=>archive_pre(tid, event)}
                 class="py-2 pr-0 flex text-gray-600 justify-between items-center cursor-pointer hover:bg-gray-200 hover:text-gray-700">
                <div class="ml-3.5 font-medium dark:text-gray-200 text-base">
                    {tid.name}
                </div>
                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24"
                     stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
                </svg>
            </div>
        {/each}
    </div>
    <div class="overflow-y-auto max-h-52 py-1.5">
        {#each currentTypelist as tid}
            <div class:selected="{currentChildren === tid.id}" on:click={(event)=>archive_pre(tid, event)}
                 class="p-2.5 cursor-pointer hover:bg-gray-200 hover:text-gray-700">
                <span class="font-weight">{tid.name}</span>
                <span class="ml-2.5 text-xs text-black text-opacity-50">{tid.desc}</span>
            </div>
        {/each}
    </div>
</div>
<style>
    .selected {
        color: #687ef2 !important;
        /*@apply text-gray-700 bg-gray-200;*/
    }
</style>

